use ed25519_dalek::{ SigningKey, VerifyingKey, Signature, Signer, Verifier };
use jsonwebtoken::Header;
use rand::rngs::OsRng;
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;
use crate::proto::ChatMessage;
use crate::proto::HandshakeMessage;
use crate::utils::message_byte::bytes_to_message;
use jsonwebtoken::{encode, decode, decode_header, DecodingKey, EncodingKey, Validation};

pub struct AuthManager {
  pub keys: Arc<RwLock<HashMap<String, VerifyingKey>>>,
}
pub struct AuthToken {
  pub user_id: String,
  pub expires_at: i64,
  pub token: String,
  pub refresh_token: String,
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
    if let Some(key) = self.keys.read().await.get(&msg.sender) {
      let message = format!("{}{}{:?}", msg.timestamp, msg.sender, msg.content);
      let signature = match Signature::try_from(&msg.signature[..]) {
        Ok(sig) => sig,
        Err(_) => {
          return false;
        }
      };

      key.verify(message.as_bytes(), &signature).is_ok()
    } else {
      false
    }
  }

  pub async fn generate_jwt(&self, user_id: &str) -> AuthToken {
    let now = chrono::Utc::now();
    let expires_at = now + chrono::Duration::hours(24);
    let refresh_expires_at = now + chrono::Duration::days(30);

    #[derive(serde::Serialize, serde::Deserialize)]
    struct Claims {
      sub: String,
      exp: i64,
      iat: i64,
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    struct RefreshClaims {
      sub: String,
      exp: i64,
      iat: i64,
      refresh: bool,
    }

    let claims = Claims {
      sub: user_id.to_string(),
      exp: expires_at.timestamp(),
      iat: now.timestamp(),
    };

    let refresh_claims = RefreshClaims {
      sub: user_id.to_string(),
      exp: refresh_expires_at.timestamp(),
      iat: now.timestamp(),
      refresh: true,
    };

    // In production, use a proper secret key
    let secret = b"your-256-bit-secret";
    let encoding_key = EncodingKey::from_secret(secret);

    let token = encode(
      &Header::default(),
      &claims,
      &encoding_key,
    ).unwrap();

    let refresh_token = encode(
      &Header::default(),
      &refresh_claims,
      &encoding_key,
    ).unwrap();

    AuthToken {
      user_id: user_id.to_string(),
      expires_at: expires_at.timestamp(),
      token,
      refresh_token,
    }
  }
}
