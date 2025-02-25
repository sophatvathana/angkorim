use std::hash::{BuildHasher, Hash, Hasher};
use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{RwLock, RwLockWriteGuard};

pub const PARTITION: usize = 8;

pub struct NodeMap<K, V, S = std::collections::hash_map::RandomState>
where K: Hash + Eq + Clone, S: BuildHasher + Default {
  pub map: Vec<Arc<RwLock<HashMap<K, V>>>>,
  hash_builder: S,
}

impl<K, V, S> NodeMap<K, V, S> where K: Hash + Eq + Clone, V: Clone, S: BuildHasher + Default {
  pub fn new() -> Self {
    Self {
      map: (0..PARTITION).map(|_| Arc::new(RwLock::new(HashMap::new()))).collect(),
      hash_builder: S::default(),
    }
  }

  pub fn get(&self, key: &K) -> &Arc<RwLock<HashMap<K, V>>> {
    let mut hasher = self.hash_builder.build_hasher();
    key.hash(&mut hasher);
    let index = hasher.finish() % PARTITION as u64;
    &self.map[index as usize]
  }

  pub async fn get_item(&self, key: &K) -> Option<V> {
    let mut hasher: <S as BuildHasher>::Hasher = self.hash_builder.build_hasher();
    key.hash(&mut hasher);
    let index = hasher.finish() % PARTITION as u64;
    let map = self.map[index as usize].read().await;
    map.get(key).map(|v| v.clone())
  }

  pub async fn update(&self, key: K, value: V) {
    let mut hasher = self.hash_builder.build_hasher();
    key.hash(&mut hasher);
    let index = hasher.finish() as usize % PARTITION;
    let mut map = self.map[index].write().await;

    if let Some(existing) = map.get_mut(&key) {
      *existing = value;
    } else {
      map.insert(key, value);
    }
  }

  pub async fn set(&self, key: K, value: V) {
    let mut hasher = self.hash_builder.build_hasher();
    key.hash(&mut hasher);
    let index = hasher.finish() % PARTITION as u64;
    let mut map = self.map[index as usize].write().await;
    map.insert(key, value);
  }

  pub async fn remove(&self, key: &K) {
    let mut hasher = self.hash_builder.build_hasher();
    key.hash(&mut hasher);
    let index = hasher.finish() % PARTITION as u64;
    let mut map = self.map[index as usize].write().await;
    map.remove(key);
  }

  pub async fn get_all(&self) -> HashMap<K, V> {
    let mut result = HashMap::new();
    for map in self.map.iter() {
      let map = map.read().await;
      result.extend(map.iter().map(|(k, v)| (k.clone(), v.clone())));
    }
    result
  }
}

#[cfg(test)]
mod tests {
  use tokio::time::Instant;

use crate::network::discovery::{Node, NodeStatus};

use super::*;

  #[tokio::test]
  async fn test_node_map() {
    let node_map = NodeMap::<&str, &str>::new();
    node_map.set("key1", "value1").await;
    assert_eq!(node_map.get(&"key1").read().await.get("key1"), Some(&"value1"));
  }

  #[tokio::test]
  async fn test_node_map_get_all() {
    let node_map = NodeMap::<&str, &str>::new();
    node_map.set("key1", "value1").await;
    node_map.set("key2", "value2").await;
    let mut expected = HashMap::new();
    expected.insert("key1", "value1");
    expected.insert("key2", "value2");
    assert_eq!(node_map.get_all().await, expected);
  }

  #[tokio::test]
  async fn test_node_map_remove() {
    let node_map = NodeMap::<&str, &str>::new();
    node_map.set("key1", "value1").await;
    node_map.remove(&"key1").await;
    assert_eq!(node_map.get(&"key1").read().await.get("key1"), None);
  }

  #[tokio::test]
  async fn test_node_map_set() {
    let last_seen = Instant::now();
    let node_map = NodeMap::<&str, Node>::new();
    node_map.set("key1", Node { id: "key1".to_string(), host: "localhost".to_string(), sync_port: 8080, discovery_port: 8080, last_seen: last_seen, status: NodeStatus::Alive, incarnation: 0 }).await;
    node_map.set("key1", Node { id: "key1".to_string(), host: "localhost".to_string(), sync_port: 8081, discovery_port: 8081, last_seen: last_seen, status: NodeStatus::Alive, incarnation: 0 }).await;
    assert_eq!(node_map.get(&"key1").read().await.get("key1"),
    Some(&Node { id: "key1".to_string(), host: "localhost".to_string(),
    sync_port: 8081, discovery_port: 8081, last_seen: last_seen, status: NodeStatus::Alive, incarnation: 0 }));
  }

  #[tokio::test]
  async fn test_node_map_read_write_concurrent() {
    let node_map = Arc::new(NodeMap::<&str, Node>::new());
    let node_map_clone = node_map.clone();
    let node_map_clone2 = node_map.clone();
    let last_seen = Instant::now();
    let handle = tokio::spawn(async move {
      node_map_clone.set("key1", Node { id: "key1".to_string(), host: "localhost".to_string(), sync_port: 8080, discovery_port: 8080, last_seen: last_seen, status: NodeStatus::Alive, incarnation: 0 }).await;
    });

    let handle2 = tokio::spawn(async move {
      node_map_clone2.set("key1", Node { id: "key1".to_string(), host: "localhost".to_string(), sync_port: 8081, discovery_port: 8081, last_seen: last_seen, status: NodeStatus::Alive, incarnation: 0 }).await;
    });

    let _ = tokio::join!(handle, handle2);
    assert_eq!(node_map.get(&"key1").read().await.get("key1"), Some(&Node { id: "key1".to_string(), host: "localhost".to_string(), sync_port: 8081, discovery_port: 8081, last_seen: last_seen, status: NodeStatus::Alive, incarnation: 0 }));
  }
}
