use std::collections::{ HashMap, HashSet };
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{ Duration, Instant };
use crate::config::{ Config, PeerConfig };
use crate::map::NodeMap;
use crate::proto::{ Ping, Pong, SuspicionMessage };
use tokio::net::TcpStream;
use tokio::time::timeout;
use prost::Message;
use tokio::io::{ AsyncReadExt, AsyncWriteExt };
use std::time::{ SystemTime, UNIX_EPOCH };

#[derive(Debug, Clone, PartialEq)]
pub enum NodeStatus {
  Alive,
  Suspect {
    suspicion_start: Instant,
    suspicion_timeout: Duration,
  },
  Dead {
    since: Instant,
  },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
  pub id: String,
  pub host: String,
  pub sync_port: u16,
  pub discovery_port: u16,
  pub last_seen: Instant,
  pub status: NodeStatus,
  pub incarnation: u64, // For conflict resolution
}

#[derive(Clone)]
pub struct Discovery {
  config: Arc<Config>,
  pub nodes: Arc<NodeMap<String, Node>>,
  failed_nodes: Arc<RwLock<HashSet<String>>>,
}

impl Discovery {
  pub fn new(config: Arc<Config>) -> Self {
    Self {
      config,
      nodes: Arc::new(NodeMap::<String, Node>::new()),
      failed_nodes: Arc::new(RwLock::new(HashSet::new())),
    }
  }

  pub async fn add_node(&self, peer: PeerConfig) {
    let node = Node {
      id: peer.node_id,
      host: peer.host,
      sync_port: peer.sync_port,
      discovery_port: peer.discovery_port,
      last_seen: Instant::now(),
      status: NodeStatus::Alive,
      incarnation: 0,
    };
    println!("Adding node: {:?}", node);
    // let mut nodes = self.nodes.write().await;
    // nodes.insert(node.id.clone(), node);
    self.nodes.set(node.id.clone(), node).await;
  }

  pub async fn start(&self) {
    let discovery_config = self.config.cluster.discovery.clone();
    // Initial peer discovery
    if let Some(node) = self.nodes.get_all().await.values().next() {
      println!("Starting discovery with initial nodes: {:?}", node);
    }

    let probe_interval = Duration::from_millis(discovery_config.probe_interval);
    let probe_timeout = Duration::from_millis(discovery_config.probe_timeout);
    let indirect_probes = discovery_config.indirect_probes;
    let nodes = Arc::clone(&self.nodes);
    let failed_nodes = Arc::clone(&self.failed_nodes);
    let config = self.config.clone();
    let discovery = Arc::new(self.clone());
    let node_id = discovery.get_node_id().clone(); // Get our node ID

    // Start peer discovery loop
    let loop_discovery = Arc::clone(&discovery);
    tokio::spawn(async move {
      loop {
        tokio::time::sleep(Duration::from_secs(30)).await;
        let peers = loop_discovery.get_peers().await;
        if let Err(e) = loop_discovery.discover_peers(peers).await {
          eprintln!("Peer discovery error: {}", e);
        }
      }
    });

    // Start failure detection loop
    tokio::spawn(async move {
      loop {
        tokio::time::sleep(probe_interval).await;

        // Get a snapshot of nodes to check
        let nodes_to_check = {
          let nodes_read = nodes.get_all().await;
          nodes_read
            .iter()
            .filter(|(id, node)| {
              // Don't probe ourselves and don't probe dead nodes
              *id != &node_id && !matches!(node.status, NodeStatus::Dead { .. })
            })
            .map(|(id, node)| (id.clone(), node.clone()))
            .collect::<Vec<_>>()
        };

        if !nodes_to_check.is_empty() {
          // println!("Found {} nodes to probe", nodes_to_check.len());
        }

        for (node_id, node) in nodes_to_check {
          println!("Probing node: {} at {}:{}", node_id, node.host, node.discovery_port);
          let probe_result = Self::probe_node(&node).await;
          if let Some(mut node) = nodes.get_item(&node_id).await {
            if probe_result.is_err() {
              node.status = NodeStatus::Suspect {
                suspicion_start: Instant::now(),
                suspicion_timeout: Duration::from_millis(discovery_config.suspicion_timeout),
              };
              nodes.update(node_id.clone(), node.clone()).await;

              let mut indirect_success = false;
              for _ in 0..indirect_probes {
                if Self::probe_node(&node).await.is_ok() {
                  indirect_success = true;
                  break;
                }
              }

              // Re-acquire write lock for final status update
              let mut dead_node = None;
              if let Some(mut node) = nodes.get_item(&node_id).await {
                if !indirect_success {
                  node.status = NodeStatus::Dead { since: Instant::now() };
                  dead_node = Some(node_id.clone());
                } else {
                  node.last_seen = Instant::now();
                  node.status = NodeStatus::Alive;
                }
                nodes.update(node_id.clone(), node.clone()).await;
              }

              if let Some(dead_node_id) = dead_node {
                failed_nodes.write().await.insert(dead_node_id.clone());
                discovery.broadcast_suspicion(&dead_node_id, "self").await;
              }
            } else {
              node.last_seen = Instant::now();
              node.status = NodeStatus::Alive;
              nodes.update(node_id.clone(), node.clone()).await;
            }
          }
        }
      }
    });
  }

  async fn probe_node(node: &Node) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match
      timeout(
        Duration::from_millis(100),
        TcpStream::connect((node.host.as_str(), node.discovery_port))
      ).await
    {
      Ok(Ok(mut stream)) => {
        let ping = Ping {
          node_id: node.id.clone(),
          timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
          cluster_port_sync: node.sync_port.clone() as u32,
          cluster_port_discovery: node.discovery_port.clone() as u32,
        };

        stream.write_all(&ping.encode_to_vec()).await?;

        let mut buf = vec![0; 1024];
        match timeout(Duration::from_millis(100), stream.read(&mut buf)).await {
          Ok(Ok(_)) => Ok(()),
          _ => Err("No response received".into()),
        }
      }
      _ => Err("Connection failed".into()),
    }
  }

  pub async fn get_peers(&self) -> Vec<PeerConfig> {
    let nodes = self.nodes.get_all().await;
    nodes
      .values()
      .map(|node| PeerConfig {
        host: node.host.clone(),
        sync_port: node.sync_port,
        node_id: node.id.clone(),
        discovery_port: node.discovery_port,
      })
      .collect()
  }

  pub async fn suspect_node(&self, node_id: &str, from_node: &str) {
    let mut nodes = self.nodes.get_all().await;
    if let Some(node) = nodes.get_mut(node_id) {
      match node.status {
        NodeStatus::Alive => {
          node.status = NodeStatus::Suspect {
            suspicion_start: Instant::now(),
            suspicion_timeout: Duration::from_millis(self.config.cluster.discovery.suspicion_timeout),
          };
          self.broadcast_suspicion(node_id, from_node).await;
        }
        _ => {}
      }
    }
  }

  async fn broadcast_suspicion(&self, suspect_id: &str, from_node: &str) {
    let nodes = self.nodes.get_all().await;
    for node in nodes.values() {
      if node.id != suspect_id && matches!(node.status, NodeStatus::Alive) {
        if let Ok(mut stream) = TcpStream::connect((node.host.as_str(), node.discovery_port)).await {
          let msg = SuspicionMessage {
            suspect_id: suspect_id.to_string(),
            from_node: from_node.to_string(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
          };
          let _ = stream.write_all(&msg.encode_to_vec()).await;
        }
      }
    }
  }

  pub async fn discover_peers(
    &self,
    seed_nodes: Vec<PeerConfig>
  ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let our_node_id = self.get_node_id();

    for peer in seed_nodes {
      println!("Discovering peer: {}", peer.node_id);
      if let Ok(mut stream) = TcpStream::connect((peer.host.as_str(), peer.discovery_port)).await {
        let ping = Ping {
          node_id: our_node_id.clone(),
          timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
          cluster_port_sync: self.config.server.cluster_port_sync as u32,
          cluster_port_discovery: self.config.server.cluster_port_discovery as u32,
        };

        let ping_bytes = ping.encode_to_vec();
        if stream.write_all(&ping_bytes).await.is_ok() {
          // Wait for pong response
          let mut buf = vec![0; 1024];
          if let Ok(n) = stream.read(&mut buf).await {
            buf.truncate(n);
            if let Ok(_pong) = Pong::decode(&buf[..]) {
              self.add_node(peer.clone()).await;
              println!("Successfully added peer: {}", peer.node_id);
            }
          }
        }
      }
    }
    Ok(())
  }

  pub fn get_node_id(&self) -> String {
    self.config.clone().server.node_id.clone()
  }
}

impl Node {
  pub fn update_status(&mut self) {
    self.last_seen = Instant::now();
    self.status = NodeStatus::Alive;
  }
}
