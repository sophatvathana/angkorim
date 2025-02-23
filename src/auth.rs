use ed25519_dalek::{ SigningKey, VerifyingKey, Signature, Signer, Verifier };
use rand::rngs::OsRng;
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;
use crate::proto::ChatMessage;
use crate::proto::HandshakeMessage;
use crate::utils::message_byte::bytes_to_message;
pub struct AuthManager {
  pub keys: Arc<RwLock<HashMap<String, VerifyingKey>>>,
}

impl AuthManager {
  pub fn new() -> Self {
    Self {
      keys: Arc::new(RwLock::new(HashMap::new())),
    }
  }

  pub async fn verify_handshake(&self, msg: &HandshakeMessage) -> bool {
    let public_key = match
      VerifyingKey::from_bytes(
        &msg.public_key
          .as_slice()
          .try_into()
          .unwrap_or([0; 32])
      )
    {
      Ok(key) => key,
      Err(_) => {
        return false;
      }
    };

    let message = format!("{}{}", msg.timestamp, msg.user_id);
    let signature = match Signature::try_from(&msg.signature[..]) {
      Ok(sig) => sig,
      Err(_) => {
        return false;
      }
    };

    if public_key.verify(message.as_bytes(), &signature).is_ok() {
      self.keys.write().await.insert(msg.user_id.clone(), public_key);
      true
    } else {
      false
    }
  }

  pub async fn verify_message(&self, msg: &ChatMessage) -> bool {
    let keys = self.keys.read().await;
    let string_content = bytes_to_message(&msg.content).unwrap();
    if let Some(public_key) = keys.get(&msg.sender) {
      let message = format!(
        "{}{}{}{}{}",
        msg.id,
        msg.sender,
        msg.receiver,
        string_content,
        msg.timestamp
      );
      let signature = match Signature::try_from(&msg.signature[..]) {
        Ok(sig) => sig,
        Err(_) => {
          return false;
        }
      };
      public_key.verify(message.as_bytes(), &signature).is_ok()
    } else {
      false
    }
  }
}
