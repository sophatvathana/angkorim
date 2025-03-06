mod config;
mod map;
mod network;
mod storage;
mod websocket;
mod auth;

pub mod proto {
  include!(concat!(env!("OUT_DIR"), "/chat.rs"));
  include!(concat!(env!("OUT_DIR"), "/protocol.rs"));
}
mod utils;

use axum::{ routing::get, Router };
use storage::{memory::MemoryStorage, Storage};
use websocket::ChatState;
use std::sync::Arc;
use tokio::{ net::TcpListener, sync::broadcast };
use crate::{
  config::Config,
  network::{ sync_manager::SyncManager, discovery::Discovery, tcp::TcpServer },
  storage::{
    sqlite::SqliteStorage,
    user::{ SqliteUserStorage, UserStorage },
  },
  websocket::{ ws_handler, handle::ConnectionHandler },
  network::discovery_server::DiscoveryServer,
};
use sqlx::sqlite::SqlitePool;
use std::time::Duration;
use std::collections::HashMap;
use tokio::sync::RwLock;
use crate::auth::AuthManager;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let config_path = std::env::args().nth(1).unwrap_or("config/config.yml".to_string());
  // Load configuration
  let config = Config::load(&config_path)?;

  let config_clone = config.clone();

  // Initialize storage
  let mut storage: Arc<dyn Storage> = Arc::new(MemoryStorage::new());
  let mut user_storage = None;

  if config.storage.r#type == "sqlite" {
    let pool = SqlitePool::connect(&format!("sqlite:{}", config.storage.sqlite.path)).await?;
    storage = Arc::new(SqliteStorage::new(&config.storage.sqlite.path).await.unwrap());
    user_storage = Some(Arc::new(SqliteUserStorage::new(pool)) as Arc<dyn UserStorage>);
  }

  let user_storage = user_storage.unwrap_or_else(|| {
    panic!("No user storage configured")
  });

  let discovery: Arc<Discovery> = Arc::new(Discovery::new(Arc::new(config_clone.clone())));

  let discovery_server = DiscoveryServer::new(
    Arc::clone(&discovery),
    config.server.cluster_port_discovery,
    Arc::new(config_clone)
  );
  tokio::spawn(async move {
    if let Err(e) = discovery_server.start().await {
      eprintln!("Discovery server error: {}", e);
    }
  });

  tokio::time::sleep(Duration::from_secs(2)).await;
  if !config.cluster.peers.is_empty() {
    if let Err(e) = discovery.discover_peers(config.cluster.peers).await {
      eprintln!("Peer discovery error: {}", e);
    }
  }

  discovery.start().await;
  // Wait for discovery server to start
  tokio::time::sleep(Duration::from_secs(3)).await;
  let (tx, _) = broadcast::channel(100);
  let sync_manager = Arc::new(
    SyncManager::new(
      config.server.node_id.clone(),
      storage as Arc<dyn Storage>,
      Arc::clone(&discovery),
      tx.clone()
    )
  );

  let connections = Arc::new(RwLock::new(HashMap::new()));
  let auth = Arc::new(AuthManager::new());

  let chat_state = Arc::new(ChatState {
    sync_manager: Arc::clone(&sync_manager),
    tx: tx.clone(),
    connections: Arc::clone(&connections),
    auth: Arc::clone(&auth),
    user_storage: Arc::clone(&user_storage),
  });

  let app = Router::new().route("/ws", get(ws_handler)).with_state(chat_state);

  let addr = (config.server.host.as_str(), config.server.port);
  println!("HTTP server listening on {}:{}", addr.0, addr.1);

  Arc::clone(&sync_manager).start_sync();

  let tcp_server = TcpServer::new(Arc::clone(&sync_manager), config.server.cluster_port_sync);

  tokio::spawn(async move {
    if let Err(e) = tcp_server.start().await {
      eprintln!("TCP server error: {}", e);
    }
  });

  axum::serve(TcpListener::bind(addr).await?, app.into_make_service()).await?;

  Ok(())
}
