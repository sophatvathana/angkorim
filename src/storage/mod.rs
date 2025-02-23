use async_trait::async_trait;
use serde::{ Deserialize, Serialize };

use crate::proto::ChatMessage;
use crate::proto::MessageType as ProtoMessageType;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
  pub id: String,
  pub sender: String,
  pub receiver: String,
  pub content: String,
  pub timestamp: i64,
  pub message_type: MessageType,
  pub version: u64,
  pub origin_node: String,
}

impl From<MessageType> for ProtoMessageType {
  fn from(message_type: MessageType) -> Self {
    match message_type {
      MessageType::Direct => ProtoMessageType::Direct,
      MessageType::Group => ProtoMessageType::Group,
      MessageType::Channel => ProtoMessageType::Channel,
    }
  }
}

impl From<ProtoMessageType> for MessageType {
  fn from(message_type: ProtoMessageType) -> Self {
    match message_type {
      ProtoMessageType::Direct => MessageType::Direct,
      ProtoMessageType::Group => MessageType::Group,
      ProtoMessageType::Channel => MessageType::Channel,
    }
  }
}

impl From<i32> for MessageType {
  fn from(message_type: i32) -> Self {
    match message_type {
      0 => MessageType::Direct,
      1 => MessageType::Group,
      2 => MessageType::Channel,
      _ => panic!("Invalid message type"),
    }
  }
}

impl From<MessageType> for i32 {
  fn from(message_type: MessageType) -> Self {
    match message_type {
      MessageType::Direct => 0,
      MessageType::Group => 1,
      MessageType::Channel => 2,
      _ => panic!("Invalid message type"),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
  Direct,
  Group,
  Channel,
}

impl PartialEq for MessageType {
  fn eq(&self, other: &Self) -> bool {
    self == other
  }
}

#[async_trait]
pub trait Storage: Send + Sync {
  async fn save_message(
    &self,
    message: Message
  ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
  async fn get_messages(
    &self,
    user: &str
  ) -> Result<Vec<Message>, Box<dyn std::error::Error + Send + Sync>>;
}

pub mod sqlite;
