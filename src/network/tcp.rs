use colored::Colorize;
use tokio::net::{ TcpListener, TcpStream };
use std::sync::Arc;
use crate::{
  network::sync_manager::SyncManager,
  proto::{ ChatMessage, EncryptionType, GossipMessage },
  utils::message_byte::message_to_bytes,
};
use prost::Message;
use tokio::io::{ AsyncReadExt, AsyncWriteExt };

pub struct TcpServer {
  sync_manager: Arc<SyncManager>,
  port: u16,
}

impl TcpServer {
  pub fn new(sync_manager: Arc<SyncManager>, port: u16) -> Self {
    Self { sync_manager, port }
  }

  pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = format!("0.0.0.0:{}", self.port);
    let listener = TcpListener::bind(&addr).await?;
    println!("Gossip server listening on {}", addr);

    loop {
      let (mut socket, addr) = listener.accept().await?;
      println!("TCP Server received connection from: {}", addr);

      let sync_manager = Arc::clone(&self.sync_manager);

      tokio::spawn(async move {
        let mut buf = vec![0; 1024 * 1024];
        match socket.read(&mut buf).await {
          Ok(n) if n > 0 => {
            buf.truncate(n);
            println!("TCP Server received {} bytes", n);

            if let Ok(gossip) = GossipMessage::decode(&buf[..]) {
              if !gossip.node_id.is_empty() {
                println!(
                  "{} {} {} {}",
                  "TCP Server received gossip from:".red(),
                  gossip.node_id,
                  "with".green(),
                  gossip.messages.len()
                );

                if let Err(e) = sync_manager.process_gossip(gossip.clone()).await {
                  eprintln!("Error processing gossip: {}", e);
                }
                // Send response back with our messages
                let our_cache = sync_manager.get_message_cache().await;
                let our_vv = sync_manager.get_version_vector().await;

                let mut response_messages = Vec::new();
                for msg in our_cache.values() {
                  let msg_proto = ChatMessage {
                    id: msg.id.clone(),
                    sender: msg.sender.clone(),
                    receiver: msg.receiver.clone(),
                    content: message_to_bytes(&msg.content),
                    timestamp: msg.timestamp as u64,
                    message_type: msg.message_type.clone().into(),
                    version_vector: our_vv.to_string(),
                    signature: Vec::new(),
                    encryption_type: EncryptionType::None as i32,
                  };
                  response_messages.push(msg_proto.encode_to_vec());
                }

                let response = GossipMessage {
                  node_id: sync_manager.get_node_id(),
                  version_vector: our_vv.to_string(),
                  messages: response_messages,
                };

                if let Err(e) = socket.write_all(&response.encode_to_vec()).await {
                  eprintln!("Error sending response: {}", e);
                }
              } else {
                println!("Received invalid gossip message with empty node_id");
              }
            }
          }
          Ok(n) => {
            println!("TCP Server received empty message ({} bytes)", n);
          }
          Err(e) => eprintln!("Error reading from socket: {}", e),
        }
      });
    }
  }
}
