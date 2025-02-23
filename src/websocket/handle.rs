use std::{ collections::HashMap, sync::Arc };

use prost::Message as _;
use tokio::sync::{ mpsc, RwLock };

use crate::{
  proto::{ ChatMessage, EncryptionType },
  storage::Message,
  utils::message_byte::message_to_bytes,
};

type Tx = mpsc::UnboundedSender<Vec<u8>>;
pub type Clients = Arc<RwLock<HashMap<String, Arc<ConnectionHandler>>>>;

pub struct ConnectionHandler {
  pub sender: Arc<Tx>,
  pub user_id: String,
}

impl ConnectionHandler {
  pub fn new(sender: Arc<Tx>, user_id: String) -> Self {
    Self { sender, user_id }
  }

  pub async fn send_message(&self, msg: Message) {
    let proto_msg = ChatMessage {
      id: msg.id,
      sender: msg.sender,
      receiver: msg.receiver,
      content: message_to_bytes(&msg.content),
      timestamp: msg.timestamp as u64,
      message_type: msg.message_type as i32,
      version_vector: msg.version.to_string(),
      signature: Vec::new(),
      encryption_type: EncryptionType::None as i32,
    };
    let bytes = proto_msg.encode_to_vec();
    let _ = self.sender.send(bytes);
  }
}
