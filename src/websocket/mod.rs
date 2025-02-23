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
  proto::{ ChatMessage, EncryptionType, HandshakeMessage },
  storage::{ Message as StorageMessage, MessageType },
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

        let conn = ConnectionHandler::new(Arc::new(private_tx), user_id.clone());
        state.connections.write().await.insert(user_id.clone(), Arc::new(conn));
      }
    }
  }

  if user_id.is_empty() {
    eprintln!("No handshake received, closing connection");
    return;
  }

  let user_id_clone = user_id.clone();

  let incoming = {
    let state = Arc::clone(&state);
    let user_id = user_id.clone();

    tokio::spawn(async move {
      while let Some(Ok(msg)) = receiver.next().await {
        if let WebSocketMessage::Binary(bytes) = msg {
          if let Ok(proto_msg) = ChatMessage::decode(bytes) {
            if !state.auth.verify_message(&proto_msg).await {
              eprintln!("Invalid message signature");
              continue;
            }

            if proto_msg.sender != user_id {
              eprintln!("Message sender doesn't match authenticated user");
              continue;
            }

            let storage_msg = StorageMessage {
              id: uuid::Uuid::new_v4().to_string(),
              sender: user_id.clone(),
              receiver: proto_msg.receiver,
              content: bytes_to_message(&proto_msg.content).unwrap(),
              timestamp: proto_msg.timestamp as i64,
              message_type: proto_msg.message_type.into(),
              version: 0,
              origin_node: user_id.clone(),
            };

            if let Err(e) = state.sync_manager.publish_message(storage_msg.clone()).await {
              eprintln!("Error publishing message: {}", e);
              continue;
            }

            if let Err(e) = state.tx.send(storage_msg) {
              eprintln!("Error broadcasting message: {}", e);
            }

            if let Err(e) = state.sync_manager.sync_with_random_peers(3).await {
              eprintln!("Error syncing after broadcast: {}", e);
            }
          }
        }
      }
    })
  };
  // let connections = state.connections.clone();
  let outgoing = tokio::spawn(async move {
    while let Ok(storage_msg) = rx.recv().await {
      println!("{} {:?}", colored::Colorize::green("Received outgoing message:"), storage_msg);
      // let connections = connections.read().await;
      // if storage_msg.message_type == MessageType::Direct {
      //     if let Some(conn) = connections.get(&storage_msg.receiver) {
      //         conn.send_message(storage_msg.clone()).await;
      //     }
      // }else
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

  let _ = state.connections.write().await.remove(&user_id);
  println!("User disconnected: {}", user_id);
}
