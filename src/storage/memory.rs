use async_trait::async_trait;

use crate::storage::Storage;
use crate::storage::Message;

pub struct MemoryStorage {
    
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Storage for MemoryStorage {
    async fn save_message(
        &self,
        message: Message
      ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
       Ok(())
    }

    async fn get_messages(
        &self,
        user: &str
      ) -> Result<Vec<Message>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(vec![])
    }
}

