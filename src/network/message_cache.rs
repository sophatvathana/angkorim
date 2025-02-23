use std::{ collections::HashMap, sync::Arc };

use tokio::sync::RwLock;

use crate::storage::Message;

pub type MessageCacheData = HashMap<String, Message>;
pub struct MessageCache {
  messages: Arc<RwLock<MessageCacheData>>,
}

impl MessageCache {
  pub fn new() -> Self {
    Self { messages: Arc::new(RwLock::new(HashMap::new())) }
  }

  pub async fn add_message(&self, key: String, message: Message) -> MessageCacheData {
    let mut messages = self.messages.write().await;
    messages.insert(key, message);
    messages.clone()
  }

  pub async fn get_messages(&self) -> MessageCacheData {
    self.messages.read().await.clone()
  }

  pub async fn retain_messages(&self) {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    let mut messages = self.messages.write().await;
    messages.retain(|_, msg| now.saturating_sub(msg.timestamp.try_into().unwrap()) < 3600);
  }
}
