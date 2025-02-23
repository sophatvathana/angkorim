use tokio::{ io::{ AsyncReadExt, AsyncWriteExt }, net::TcpListener };
use crate::network::{
  protocol::{ AliveMessage, Ping, Pong, SuspicionMessage },
  discovery::NodeStatus,
};
use prost::Message;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use super::discovery::{ Node, Discovery };
use std::time::{ Instant };
use crate::config::Config;

pub struct DiscoveryServer {
  nodes: Arc<RwLock<HashMap<String, Node>>>,
  discovery: Arc<Discovery>,
  port: u16,
  config: Arc<Config>,
}

impl DiscoveryServer {
  pub fn new(discovery: Arc<Discovery>, port: u16, config: Arc<Config>) -> Self {
    Self {
      nodes: Arc::clone(&discovery.nodes),
      discovery,
      port,
      config,
    }
  }

  pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = format!("0.0.0.0:{}", self.port);
    let listener = TcpListener::bind(&addr).await?;

    loop {
      let (mut socket, addr) = listener.accept().await?;
      let nodes = Arc::clone(&self.nodes);
      let discovery = Arc::clone(&self.discovery);
      let config = Arc::clone(&self.config);

      tokio::spawn(async move {
        let mut buf = vec![0; 1024];
        if let Ok(n) = socket.read(&mut buf).await {
          buf.truncate(n);

          if let Ok(ping) = Ping::decode(buf.as_slice()) {
            let pong = Pong {
              node_id: ping.node_id.clone(),
              timestamp: std::time::SystemTime
                ::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            };
            let _ = socket.write_all(&pong.encode_to_vec()).await;

            {
              let mut nodes_write = nodes.write().await;
              if !nodes_write.contains_key(&ping.node_id) {
                let node = Node {
                  id: ping.node_id.clone(),
                  host: addr.ip().to_string(),
                  sync_port: ping.cluster_port_sync as u16,
                  discovery_port: ping.cluster_port_discovery as u16,
                  last_seen: Instant::now().into(),
                  status: NodeStatus::Alive,
                  incarnation: 0,
                };
                nodes_write.insert(ping.node_id.clone(), node);
                println!("Added new node from ping: {}", ping.node_id);
              }
            }
          } else if let Ok(suspicion) = SuspicionMessage::decode(buf.as_slice()) {
            discovery.suspect_node(&suspicion.suspect_id, &suspicion.from_node).await;
          } else if let Ok(alive) = AliveMessage::decode(buf.as_slice()) {
            // Handle alive message
            if let Some(node) = nodes.write().await.get_mut(&alive.node_id) {
              if alive.incarnation > node.incarnation {
                node.update_status();
                node.incarnation = alive.incarnation;
              }
            }
          }
        }
      });
    }
  }
}
