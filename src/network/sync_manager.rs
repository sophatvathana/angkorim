use std::sync::Arc;
use tokio::{
  io::{ AsyncReadExt, AsyncWriteExt },
  net::TcpStream,
  sync::{ broadcast, Mutex, RwLock },
};
use crate::{
  proto::{ ChatMessage, EncryptionType, GossipMessage, MessageType, SyncRequest, SyncResponse },
  storage::{ Message, Storage },
  utils::message_byte::{ bytes_to_message, message_to_bytes },
};
use prost::Message as ProstMessage;
use std::time::Duration;
use rand::{ rngs::StdRng, seq::{ SliceRandom }, SeedableRng };
use super::{
  message_cache::{ MessageCache, MessageCacheData },
  discovery::{ NodeStatus, Discovery },
};
use colored::Colorize;
use super::version::VersionVector;

pub struct SyncManager {
  node_id: String,
  version_vector: Arc<VersionVector>,
  storage: Arc<dyn Storage>,
  discovery: Arc<Discovery>,
  message_cache: Arc<MessageCache>,
  broadcast_tx: Option<broadcast::Sender<Message>>,
}

impl SyncManager {
  pub fn new(
    node_id: String,
    storage: Arc<dyn Storage>,
    discovery: Arc<Discovery>,
    broadcast_tx: broadcast::Sender<Message>
  ) -> Self {
    Self {
      node_id,
      version_vector: Arc::new(VersionVector::new()),
      storage,
      discovery,
      message_cache: Arc::new(MessageCache::new()),
      broadcast_tx: Some(broadcast_tx),
    }
  }

  pub async fn publish_message(&self, message: Message) -> Result<(), Box<dyn std::error::Error>> {
    println!("SyncManager: Starting publish_message for: {:?}", message);
    let version = self.version_vector.increment(&message.sender);
    // Create message with version info
    let versioned_message = Message {
      id: message.id.clone(),
      sender: message.sender.clone(),
      receiver: message.receiver.clone(),
      content: message.content.clone(),
      timestamp: message.timestamp,
      message_type: message.message_type,
      version,
      origin_node: message.sender.clone(),
    };

    // Add to cache with version info with timeout
    let cache = self.message_cache.add_message(
      versioned_message.id.clone(),
      versioned_message.clone()
    ).await;
    // Save to storage
    let _ = self.storage.save_message(versioned_message.clone()).await;
    println!("{} {:?}", "Saved message to storage:".green(), versioned_message.clone());
    // Trigger immediate sync with some peers
    println!("Triggering immediate sync with peers");
    self.sync_with_random_peers(3).await?;
    Ok(())
  }

  pub async fn sync_with_random_peers(
    &self,
    count: usize
  ) -> Result<(), Box<dyn std::error::Error>> {
    println!("Syncing with random peers - attempting to acquire discovery nodes lock...");
    let nodes = self.discovery.nodes.get_all().await;
    println!("Successfully acquired discovery nodes lock");
    let node_keys = nodes.keys().collect::<Vec<_>>();
    println!("Known nodes for sync: {:?}", node_keys);

    let alive_nodes: Vec<_> = nodes
      .values()
      .filter(|n| matches!(n.status, NodeStatus::Alive))
      .collect();

    println!("Found {} alive nodes for sync", alive_nodes.len());

    if alive_nodes.is_empty() {
      println!("No alive nodes found for sync");
      return Ok(());
    }

    let mut rng = StdRng::from_rng(&mut rand::thread_rng()).unwrap();
    let selected = alive_nodes.choose_multiple(
      &mut rng,
      std::cmp::min(count, alive_nodes.len())
    );

    let sync_targets: Vec<(String, String, u16)> = selected
      .map(|node| (node.id.clone(), node.host.clone(), node.sync_port))
      .collect();
    // Now perform the sync with the cloned data
    for (id, host, port) in sync_targets {
      let addr = format!("{}:{}", host, port);
      println!("Attempting to sync with node at {} (id: {})", addr, id);
      if let Err(e) = self.sync_with_peer(addr).await {
        eprintln!("Error syncing with {}: {}", id, e);
      }
    }
    Ok(())
  }

  pub async fn sync_with_peer(&self, peer_addr: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = match TcpStream::connect(&peer_addr).await {
      Ok(stream) => {
        println!("Successfully connected to peer {}", peer_addr);
        stream
      }
      Err(e) => {
        println!("Failed to connect to peer {}: {}", peer_addr, e);
        return Ok(());
      }
    };

    let our_vv = self.version_vector.clone();
    let caches = self.message_cache.get_messages().await;

    // Send all messages in initial sync
    let mut messages = Vec::new();
    for (id, msg) in caches {
      println!(
        "Preparing message for sync: id={}, sender={}, content={}",
        msg.id,
        msg.sender,
        msg.content
      );
      let msg_proto = ChatMessage {
        id: msg.id.clone(),
        sender: msg.sender.clone(),
        receiver: msg.receiver.clone(),
        content: message_to_bytes(&msg.content.clone()),
        timestamp: msg.timestamp as u64,
        message_type: msg.message_type.clone().into(),
        version_vector: our_vv.to_string(),
        signature: Vec::new(),
        encryption_type: EncryptionType::None as i32,
      };
      messages.push(msg_proto.encode_to_vec());
    }

    println!("Sending {} messages to peer {}", messages.len(), peer_addr);

    let gossip = GossipMessage {
      node_id: self.node_id.clone(),
      version_vector: our_vv.to_string(),
      messages,
    };

    // Send gossip message
    if let Err(e) = stream.write_all(&gossip.encode_to_vec()).await {
      println!("Failed to send gossip to peer {}: {}", peer_addr, e);
      return Ok(());
    }
    println!("Successfully sent gossip to peer {}", peer_addr);

    // Read and process response
    let mut buf = vec![0; 1024 * 1024];
    match stream.read(&mut buf).await {
      Ok(n) => {
        buf.truncate(n);
        if let Ok(response) = GossipMessage::decode(buf.as_slice()) {
          if let Err(e) = self.process_gossip(response).await {
            println!("Failed to process gossip from peer {}: {}", peer_addr, e);
          }
        } else {
          println!("Failed to decode response from peer {}", peer_addr);
        }
      }
      Err(e) => {
        println!("Failed to read response from peer {}: {}", peer_addr, e);
      }
    }
    Ok(())
  }

  pub async fn process_gossip(
    &self,
    gossip: GossipMessage
  ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("{} {}", "Processing gossip from:".purple(), gossip.node_id);
    let peer_vv = VersionVector::from_string(&gossip.version_vector).map_err(|e|
      format!("Failed to parse version vector: {}", e)
    )?;

    let caches = self.message_cache.get_messages().await;

    // First merge the peer's version vector with ours
    let our_vv = self.version_vector.clone();
    our_vv.merge(&peer_vv);

    for msg_bytes in gossip.messages {
      if let Ok(msg) = ChatMessage::decode(msg_bytes.as_slice()) {
        // Process all received messages that we don't have
        if !caches.contains_key(&msg.id) {
          let storage_msg = Message {
            id: msg.id.clone(),
            sender: msg.sender.clone(),
            receiver: msg.receiver.clone(),
            content: bytes_to_message(&msg.content).unwrap(),
            timestamp: msg.timestamp as i64,
            message_type: msg.message_type.into(),
            version: peer_vv.get_version(&msg.sender),
            origin_node: msg.sender.clone(),
          };

          if let Err(e) = self.storage.save_message(storage_msg.clone()).await {
            println!("Note: Message {} already exists in storage", msg.id);
          } else {
            println!("Saved new message: {} from {}", msg.id, msg.sender);
            self.message_cache.add_message(msg.id, storage_msg.clone()).await;
          }
          println!("{}", "Broadcasting message".green());
          if let Some(tx) = &self.broadcast_tx {
            println!("{} {:?}", "Broadcasting message".green(), storage_msg);
            let _ = tx.send(storage_msg);
          }
        }
      }
    }
    Ok(())
  }

  pub fn start_sync(self: Arc<Self>) {
    // Start periodic sync
    tokio::spawn(async move {
      loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
        self.clean_message_cache().await;
        let sync_targets = match
          tokio::time::timeout(Duration::from_secs(1), async {
            let nodes = self.discovery.nodes.get_all().await;
            let targets: Vec<(String, String, u16)> = nodes
              .values()
              .filter(|n| matches!(n.status, NodeStatus::Alive))
              .map(|n| (n.id.clone(), n.host.clone(), n.sync_port))
              .collect();
            targets
          }).await
        {
          Ok(targets) => targets,
          Err(_) => {
            println!("Timeout getting sync targets");
            continue;
          }
        };

        for (id, host, port) in sync_targets {
          let addr = format!("{}:{}", host, port);
          if
            let Err(e) = tokio::time::timeout(
              Duration::from_secs(5),
              self.sync_with_peer(addr.clone())
            ).await
          {
            println!("Timeout syncing with {}: {:?}", addr, e);
          }
        }
      }
    });
  }

  async fn clean_message_cache(&self) {
    let caches = self.message_cache.clone();
    caches.retain_messages().await;
  }

  pub fn get_node_id(&self) -> String {
    self.node_id.clone()
  }

  pub async fn get_version_vector(&self) -> Arc<VersionVector> {
    self.version_vector.clone()
  }

  pub async fn get_message_cache(&self) -> MessageCacheData {
    self.message_cache.get_messages().await
  }
}
