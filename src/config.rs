use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
  pub server: ServerConfig,
  pub cluster: ClusterConfig,
  pub storage: StorageConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
  pub host: String,
  pub port: u16,
  pub node_id: String,
  pub cluster_port_sync: u16,
  pub cluster_port_discovery: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ClusterConfig {
  pub peers: Vec<PeerConfig>,
  pub discovery: DiscoveryConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PeerConfig {
  pub host: String,
  pub sync_port: u16,
  pub discovery_port: u16,
  pub node_id: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DiscoveryConfig {
  pub probe_interval: u64,
  pub probe_timeout: u64,
  pub indirect_probes: u32,
  pub suspicion_timeout: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StorageConfig {
  pub r#type: String,
  pub sqlite: SqliteConfig,
  pub postgres: PostgresConfig,
  pub dynamodb: DynamoDbConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SqliteConfig {
  pub path: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PostgresConfig {
  pub host: String,
  pub port: u16,
  pub database: String,
  pub user: String,
  pub password: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DynamoDbConfig {
  pub region: String,
  pub table: String,
}

impl Config {
  pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let config = serde_yaml::from_str(&content)?;
    Ok(config)
  }
}
