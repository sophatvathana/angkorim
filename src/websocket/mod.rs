pub mod handle;
use aws_sdk_dynamodb::Client;
use axum::{
  extract::ws::{ Message as WebSocketMessage, WebSocket, WebSocketUpgrade },
  response::IntoResponse,
  extract::State,
};
use bytes::Bytes;
use futures::{ SinkExt, StreamExt };
use handle::{ Clients, ConnectionHandler };
use prost::Message;
use std::sync::Arc;
use tokio::sync::{ broadcast, mpsc };
use crate::{
  proto::{ ChatMessage, EncryptionType, HandshakeMessage, WsMessage, ws_message },
  storage::{ Message as StorageMessage, MessageType, user::{ Message as UserMessage, UserStorage } },
  utils::message_byte::{ bytes_to_message, message_to_bytes },
};
use crate::network::sync_manager::SyncManager;
use std::collections::HashMap;
use crate::auth::AuthManager;
use tokio::sync::RwLock;

pub struct ChatState {
  pub sync_manager: Arc<SyncManager>,
  pub tx: broadcast::Sender<StorageMessage>,
  pub connections: Clients,
  pub auth: Arc<AuthManager>,
  pub user_storage: Arc<dyn UserStorage>,
}

pub async fn ws_handler(
  ws: WebSocketUpgrade,
  State(state): State<Arc<ChatState>>
) -> impl IntoResponse {
  ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<ChatState>) {
  let (mut sender, mut receiver) = socket.split();
  let (private_tx, mut private_rx) = mpsc::unbounded_channel();
  let mut rx: broadcast::Receiver<StorageMessage> = state.tx.subscribe();
  let mut user_id = String::new();
  // Wait for handshake message
  if let Some(Ok(msg)) = receiver.next().await {
    if let WebSocketMessage::Binary(bytes) = msg {
      if let Ok(handshake) = HandshakeMessage::decode(bytes.as_ref()) {
        // Verify handshake signature
        if !state.auth.verify_handshake(&handshake).await {
          eprintln!("Invalid handshake signature");
          return;
        }
        user_id = handshake.user_id;
        println!("User connected: {}", user_id);

        let conn = ConnectionHandler::new(
          Arc::new(private_tx),
          user_id.clone(),
          Arc::clone(&state.auth),
          Arc::clone(&state.user_storage)
        );
        state.connections.write().await.insert(user_id.clone(), Arc::new(conn));
      }
    }
  }

  if user_id.is_empty() {
    eprintln!("No handshake received, closing connection");
    return;
  }

  let user_id_clone = user_id.clone();
  let state_clone = Arc::clone(&state);

  let incoming = {
    let state = Arc::clone(&state);
    let user_id = user_id.clone();

    tokio::spawn(async move {
      while let Some(Ok(msg)) = receiver.next().await {
        if let WebSocketMessage::Binary(bytes) = msg {
          if let Ok(proto_msg) = WsMessage::decode(bytes) {
            // For chat messages, verify signature
            if let Some(ws_message::Message::ChatMessage(chat_msg)) = &proto_msg.message {
              println!("DEBUG: Received chat message from {}", chat_msg.sender);
              if !state.auth.verify_message(chat_msg).await {
                eprintln!("Invalid message signature");
                continue;
              }

              if chat_msg.sender != user_id {
                eprintln!("Message sender doesn't match authenticated user");
                continue;
              }

              let storage_msg = StorageMessage {
                id: uuid::Uuid::new_v4().to_string(),
                sender: user_id.clone(),
                receiver: chat_msg.receiver.clone(),
                content: bytes_to_message(&chat_msg.content).unwrap(),
                timestamp: chat_msg.timestamp as i64,
                message_type: chat_msg.message_type.into(),
                version: 0,
                origin_node: user_id.clone(),
              };
              let storage_msg_clone = storage_msg.clone();
              let msg_type = storage_msg.message_type.clone();
              let broadcast_msg = storage_msg_clone.clone();
              let direct_msg = storage_msg_clone.clone();
              let publish_msg = storage_msg_clone;
              println!("DEBUG: Created storage message");

              if let Some(conn) = state.connections.read().await.get(&user_id) {
                if let Err(e) = conn.user_storage.save_message(&UserMessage {
                    id: storage_msg.id.clone(),
                    sender: storage_msg.sender.clone(),
                    receiver: storage_msg.receiver.clone(),
                    content: storage_msg.content.clone(),
                    timestamp: storage_msg.timestamp,
                    message_type: msg_type as i32,
                    version: storage_msg.version as i64,
                    origin_node: storage_msg.origin_node.clone(),
                    delivered: false,
                    read: false,
                    delivered_at: None,
                    read_at: None,
                }).await {
                    eprintln!("Error saving message: {}", e);
                    continue;
                }
              }

              if direct_msg.message_type == MessageType::Direct {
                println!("DEBUG: Handling direct message to {}", direct_msg.receiver);
                let connections = state.connections.read().await;
                if let Some(conn) = connections.get(&direct_msg.receiver) {
                    let proto_msg = ChatMessage {
                        id: direct_msg.id.clone(),
                        sender: direct_msg.sender.clone(),
                        receiver: direct_msg.receiver.clone(),
                        content: message_to_bytes(&direct_msg.content),
                        timestamp: direct_msg.timestamp as u64,
                        message_type: direct_msg.message_type as i32,
                        version_vector: direct_msg.version.to_string(),
                        signature: chat_msg.signature.clone(),
                        encryption_type: chat_msg.encryption_type,
                        metadata: None,
                    };

                    let ws_msg = WsMessage {
                        message: Some(ws_message::Message::ChatMessage(proto_msg)),
                    };

                    let bytes = ws_msg.encode_to_vec();
                    if let Err(e) = conn.sender.send(bytes) {
                        eprintln!("Error sending direct message: {}", e);
                    }
                }
              } else {
                println!("DEBUG: Broadcasting non-direct message");
                if let Err(e) = state.tx.send(broadcast_msg) {
                    eprintln!("Error broadcasting message: {}", e);
                }
              }

              println!("DEBUG: Publishing message to sync manager");
              if let Err(e) = state.sync_manager.publish_message(publish_msg).await {
                eprintln!("Error publishing message: {}", e);
                continue;
              }

              println!("DEBUG: Syncing with peers");
              if let Err(e) = state.sync_manager.sync_with_random_peers(3).await {
                eprintln!("Error syncing after broadcast: {}", e);
              }
            }
          }
        }
      }
    })
  };

  // Update outgoing message handling to use connections map for direct messages
  let outgoing = tokio::spawn(async move {
    while let Ok(storage_msg) = rx.recv().await {
      println!("{} {:?}", colored::Colorize::green("Received outgoing message:"), storage_msg);

      // Skip direct messages as they are handled in the incoming flow
      if storage_msg.message_type == MessageType::Direct {
        continue;
      }

      // For non-direct messages, send to all relevant users
      if storage_msg.sender == user_id_clone || storage_msg.receiver == user_id_clone {
        println!("Sending message to {}: {:?}", user_id_clone, storage_msg);
        let proto_msg = ChatMessage {
          id: storage_msg.id,
          sender: storage_msg.sender,
          receiver: storage_msg.receiver,
          content: message_to_bytes(&storage_msg.content),
          timestamp: storage_msg.timestamp as u64,
          message_type: storage_msg.message_type.into(),
          version_vector: String::new(),
          signature: Vec::new(),
          encryption_type: EncryptionType::None as i32,
          metadata: None,
        };

        let bytes = proto_msg.encode_to_vec();
        if let Err(e) = sender.send(WebSocketMessage::Binary(Bytes::from(bytes))).await {
          eprintln!("Error sending message: {}", e);
          break;
        }
      }
    }
  });

  tokio::select! {
    _ = incoming => {}
    _ = outgoing => {}
  }

  let _ = state_clone.connections.write().await.remove(&user_id);
  println!("User disconnected: {}", user_id);
}
