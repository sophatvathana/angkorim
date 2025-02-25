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
    let mut config = serde_yaml::from_str::<Self>(&content)?;
    // Allow for environment variables to override the config
    for (key, value) in std::env::vars() {
      if let Some(config_key) = key.strip_prefix("ANGKORIM_") {
        let config_key = config_key.replace('_', ".");
        let config_value = value.parse::<String>().unwrap();
        config.set(&config_key, config_value);
      }
    }
    Ok(config)
  }

  pub fn set(&mut self, key: &str, value: String) {
    match key {
      "server.host" => self.server.host = value,
      "server.port" => self.server.port = value.parse::<u16>().unwrap(),
      "server.node_id" => self.server.node_id = value,
      "cluster.peers.host" => self.cluster.peers[0].host = value,
      "cluster.peers.sync_port" => self.cluster.peers[0].sync_port = value.parse::<u16>().unwrap(),
      "cluster.peers.discovery_port" => self.cluster.peers[0].discovery_port = value.parse::<u16>().unwrap(),
      "cluster.peers.node_id" => self.cluster.peers[0].node_id = value,
      "cluster.discovery.probe_interval" => self.cluster.discovery.probe_interval = value.parse::<u64>().unwrap(),
      "cluster.discovery.probe_timeout" => self.cluster.discovery.probe_timeout = value.parse::<u64>().unwrap(),
      "cluster.discovery.indirect_probes" => self.cluster.discovery.indirect_probes = value.parse::<u32>().unwrap(),
      "cluster.discovery.suspicion_timeout" => self.cluster.discovery.suspicion_timeout = value.parse::<u64>().unwrap(),
      "storage.type" => self.storage.r#type = value,
      "storage.sqlite.path" => self.storage.sqlite.path = value,
      "storage.postgres.host" => self.storage.postgres.host = value,
      "storage.postgres.port" => self.storage.postgres.port = value.parse::<u16>().unwrap(),
      "storage.postgres.database" => self.storage.postgres.database = value,
      "storage.postgres.user" => self.storage.postgres.user = value,
      "storage.postgres.password" => self.storage.postgres.password = value,
      "storage.dynamodb.region" => self.storage.dynamodb.region = value,
      "storage.dynamodb.table" => self.storage.dynamodb.table = value,
      _ => {}
    }
  }
}
