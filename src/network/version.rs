use std::collections::HashMap;
use std::sync::{ Arc, Mutex, RwLock };

use serde::{ Deserialize, Serialize };

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VersionVector {
  versions: Arc<RwLock<HashMap<String, Arc<Mutex<u64>>>>>,
}

impl VersionVector {
  pub fn new() -> Self {
    Self {
      versions: Arc::new(RwLock::new(HashMap::new())),
    }
  }
  pub fn increment(&self, node_id: &str) -> u64 {
    let versions = self.versions.read().unwrap();

    if let Some(counter) = versions.get(node_id) {
      let mut value = counter.lock().unwrap();
      *value += 1;
      return *value;
    }

    drop(versions);

    let mut versions = self.versions.write().unwrap();
    let counter = Arc::new(Mutex::new(1));
    versions.insert(node_id.to_string(), counter.clone());
    *counter.clone().lock().unwrap()
  }

  pub fn get_version(&self, node_id: &str) -> u64 {
    let versions = self.versions.read().unwrap();
    if let Some(counter) = versions.get(node_id) {
      return *counter.lock().unwrap();
    }
    0
  }

  pub fn merge(&self, other: &VersionVector) {
    let mut versions = self.versions.write().unwrap();

    for (node, other_counter) in other.versions.read().unwrap().iter() {
      let counter = versions.entry(node.clone()).or_insert_with(|| Arc::new(Mutex::new(0)));

      let mut local_value = counter.lock().unwrap();
      let other_value = other_counter.lock().unwrap();
      *local_value = (*local_value).max(*other_value);
    }
  }

  pub fn to_string(&self) -> String {
    let versions = self.versions.read().unwrap();
    let map: HashMap<String, u64> = versions
      .iter()
      .map(|(node, counter)| (node.clone(), *counter.lock().unwrap()))
      .collect();
    serde_json::to_string(&map).unwrap()
  }

  pub fn from_string(s: &str) -> Result<Self, serde_json::Error> {
    let map: HashMap<String, u64> = serde_json::from_str(s).unwrap();
    let mut versions = HashMap::new();

    for (node, value) in map {
      versions.insert(node, Arc::new(Mutex::new(value)));
    }

    Ok(Self {
      versions: Arc::new(RwLock::new(versions)),
    })
  }
}
