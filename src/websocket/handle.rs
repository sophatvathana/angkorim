use std::{ collections::HashMap, sync::Arc };

use prost::Message as _;
use tokio::sync::{ mpsc, RwLock };

use crate::{
  proto::{
    ws_message, AuthInitiate, AuthMethod, AuthResponse, AuthSendCodeResponse,
    AuthStatus, ChatMessage, EncryptionType, User, WsMessage,
    auth_initiate,
  },
  storage::user::{ Message, UserStorage, SqliteUserStorage, VerificationCode },
  utils::message_byte::message_to_bytes,
  auth::AuthManager,
};

type Tx = mpsc::UnboundedSender<Vec<u8>>;
pub type Clients = Arc<RwLock<HashMap<String, Arc<ConnectionHandler>>>>;

pub struct ConnectionHandler {
  pub sender: Arc<Tx>,
  pub user_id: String,
  pub auth_manager: Arc<AuthManager>,
  pub user_storage: Arc<dyn UserStorage>,
}

impl ConnectionHandler {
  pub fn new(
    sender: Arc<Tx>,
    user_id: String,
    auth_manager: Arc<AuthManager>,
    user_storage: Arc<dyn UserStorage>
  ) -> Self {
    Self {
      sender,
      user_id,
      auth_manager,
      user_storage,
    }
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
      metadata: None,
    };
    let bytes = proto_msg.encode_to_vec();
    let _ = self.sender.send(bytes);
  }

  pub async fn handle_incoming_message(&self, msg: WsMessage) {
    match msg.message {
      Some(ws_message::Message::ChatMessage(chat_msg)) => {
        // Verify message signature and sender
        if !self.auth_manager.verify_message(&chat_msg).await {
          eprintln!("Invalid message signature");
          return;
        }

        if chat_msg.sender != self.user_id {
          eprintln!("Message sender doesn't match authenticated user");
          return;
        }

        // Create storage message
        let storage_msg = Message {
          id: uuid::Uuid::new_v4().to_string(),
          sender: self.user_id.clone(),
          receiver: chat_msg.receiver.clone(),
          content: String::from_utf8_lossy(&chat_msg.content).to_string(),
          timestamp: chrono::Utc::now().timestamp(),
          message_type: chat_msg.message_type,
          version: 0,
          origin_node: self.user_id.clone(),
          delivered: false,
          read: false,
          delivered_at: None,
          read_at: None,
        };
        println!("Saving message: {:?}", storage_msg);
        // Store message
        if let Err(e) = self.user_storage.save_message(&storage_msg).await {
          eprintln!("Error saving message: {}", e);
          return;
        }

        let proto_msg = ChatMessage {
          id: storage_msg.id,
          sender: storage_msg.sender,
          receiver: storage_msg.receiver,
          content: chat_msg.content,
          timestamp: storage_msg.timestamp as u64,
          message_type: storage_msg.message_type,
          version_vector: storage_msg.version.to_string(),
          signature: chat_msg.signature,
          encryption_type: chat_msg.encryption_type,
          metadata: Some(crate::proto::MessageMetadata {
            data: Some(crate::proto::message_metadata::Data::DeliveryStatus(
              crate::proto::DeliveryStatus {
                delivered: storage_msg.delivered,
                read: storage_msg.read,
                delivered_at: storage_msg.delivered_at.unwrap_or(0) as u64,
                read_at: storage_msg.read_at.unwrap_or(0) as u64,
              }
            ))
          }),
        };

        // Send to recipient if online
        let ws_msg = WsMessage {
          message: Some(ws_message::Message::ChatMessage(proto_msg)),
        };
        let bytes = ws_msg.encode_to_vec();
        let _ = self.sender.send(bytes);

        // Handle read receipts and typing indicators through metadata
        if let Some(metadata) = chat_msg.metadata {
          match metadata.data {
            Some(crate::proto::message_metadata::Data::ReadReceipt(receipt)) => {
              let now = chrono::Utc::now().timestamp();
              // Create read receipt message
              let read_msg = Message {
                id: uuid::Uuid::new_v4().to_string(),
                sender: self.user_id.clone(),
                receiver: chat_msg.sender.clone(),
                content: format!("read:{}", receipt.message_id),
                timestamp: now,
                message_type: crate::proto::MessageType::ReadReceipt as i32,
                version: 0,
                origin_node: self.user_id.clone(),
                delivered: true,
                read: true,
                delivered_at: Some(now),
                read_at: Some(now),
              };

              if let Err(e) = self.user_storage.save_message(&read_msg).await {
                eprintln!("Error saving read receipt: {}", e);
                return;
              }

              // Forward read receipt to original sender
              let proto_msg = ChatMessage {
                id: read_msg.id,
                sender: read_msg.sender,
                receiver: read_msg.receiver,
                content: read_msg.content.into_bytes(),
                timestamp: read_msg.timestamp as u64,
                message_type: read_msg.message_type,
                version_vector: read_msg.version.to_string(),
                signature: Vec::new(),
                encryption_type: chat_msg.encryption_type,
                metadata: Some(crate::proto::MessageMetadata {
                  data: Some(crate::proto::message_metadata::Data::ReadReceipt(receipt))
                }),
              };

              let ws_msg = WsMessage {
                message: Some(ws_message::Message::ChatMessage(proto_msg)),
              };
              let bytes = ws_msg.encode_to_vec();
              let _ = self.sender.send(bytes);
            }
            Some(crate::proto::message_metadata::Data::Typing(typing)) => {
              // Forward typing indicator to recipient
              let typing_msg = Message {
                id: uuid::Uuid::new_v4().to_string(),
                sender: self.user_id.clone(),
                receiver: typing.chat_id.clone(),
                content: format!("typing:{}", typing.is_typing),
                timestamp: chrono::Utc::now().timestamp(),
                message_type: crate::proto::MessageType::Typing as i32,
                version: 0,
                origin_node: self.user_id.clone(),
                delivered: true,
                read: true,
                delivered_at: Some(chrono::Utc::now().timestamp()),
                read_at: Some(chrono::Utc::now().timestamp()),
              };

              let proto_msg = ChatMessage {
                id: typing_msg.id,
                sender: typing_msg.sender,
                receiver: typing_msg.receiver,
                content: typing_msg.content.into_bytes(),
                timestamp: typing_msg.timestamp as u64,
                message_type: typing_msg.message_type,
                version_vector: typing_msg.version.to_string(),
                signature: Vec::new(),
                encryption_type: chat_msg.encryption_type,
                metadata: Some(crate::proto::MessageMetadata {
                  data: Some(crate::proto::message_metadata::Data::Typing(typing))
                }),
              };

              let ws_msg = WsMessage {
                message: Some(ws_message::Message::ChatMessage(proto_msg)),
              };
              let bytes = ws_msg.encode_to_vec();
              let _ = self.sender.send(bytes);
            }
            _ => {}
          }
        }
      }
      Some(ws_message::Message::AuthInitiate(auth_initiate)) => {
        println!("Received auth initiate message: {:?}", auth_initiate);

        // Handle authentication based on method
        match auth_initiate.auth_method() {
          AuthMethod::Username => {
            if let Some(username) = auth_initiate.identifier.as_ref() {
              match username {
                auth_initiate::Identifier::Username(username) => {
                  // Check if user exists
                  match self.user_storage.get_user_by_username(username).await {
                    Ok(Some(_)) => {
                      // User already exists - send error
                      let auth_response = AuthResponse {
                        user: None,
                        session_token: String::new(),
                        refresh_token: String::new(),
                        status: AuthStatus::UsernameTaken as i32,
                      };

                      let ws_msg = WsMessage {
                        message: Some(ws_message::Message::AuthResponse(auth_response)),
                      };

                      let bytes = ws_msg.encode_to_vec();
                      let _ = self.sender.send(bytes);
                    }
                    Ok(None) => {
                      // Create new user
                      let user = crate::storage::user::User {
                        id: uuid::Uuid::new_v4().to_string(),
                        username: Some(username.clone()),
                        phone_number: None,
                        password_hash: SqliteUserStorage::hash_password(&auth_initiate.password),
                        full_name: None,
                        avatar: None,
                        created_at: chrono::Utc::now().timestamp(),
                        updated_at: chrono::Utc::now().timestamp(),
                      };

                      match self.user_storage.create_user(&user).await {
                        Ok(_) => {
                          let auth_token = self.auth_manager.generate_jwt(&user.id).await;
                          let auth_response = AuthResponse {
                            user: Some(User {
                              id: user.id,
                              name: username.clone(),
                              avatar: String::new(),
                            }),
                            session_token: auth_token.token,
                            refresh_token: auth_token.refresh_token,
                            status: AuthStatus::Success as i32,
                          };

                          let ws_msg = WsMessage {
                            message: Some(ws_message::Message::AuthResponse(auth_response)),
                          };

                          let bytes = ws_msg.encode_to_vec();
                          let _ = self.sender.send(bytes);
                        }
                        Err(e) => {
                          eprintln!("Error creating user: {}", e);
                          let auth_response = AuthResponse {
                            user: None,
                            session_token: String::new(),
                            refresh_token: String::new(),
                            status: AuthStatus::InvalidCredentials as i32,
                          };

                          let ws_msg = WsMessage {
                            message: Some(ws_message::Message::AuthResponse(auth_response)),
                          };

                          let bytes = ws_msg.encode_to_vec();
                          let _ = self.sender.send(bytes);
                        }
                      }
                    }
                    Err(e) => {
                      eprintln!("Database error: {}", e);
                      let auth_response = AuthResponse {
                        user: None,
                        session_token: String::new(),
                        refresh_token: String::new(),
                        status: AuthStatus::InvalidCredentials as i32,
                      };

                      let ws_msg = WsMessage {
                        message: Some(ws_message::Message::AuthResponse(auth_response)),
                      };

                      let bytes = ws_msg.encode_to_vec();
                      let _ = self.sender.send(bytes);
                    }
                  }
                }
                _ => {
                  // Send error response for invalid username
                  let auth_response = AuthResponse {
                    user: None,
                    session_token: String::new(),
                    refresh_token: String::new(),
                    status: AuthStatus::InvalidCredentials as i32,
                  };

                  let ws_msg = WsMessage {
                    message: Some(ws_message::Message::AuthResponse(auth_response)),
                  };

                  let bytes = ws_msg.encode_to_vec();
                  let _ = self.sender.send(bytes);
                }
              }
            }
          }
          AuthMethod::Phone => {
            if let Some(phone_number) = auth_initiate.identifier.as_ref() {
              match phone_number {
                auth_initiate::Identifier::PhoneNumber(phone) => {
                  // Check if user exists
                  match self.user_storage.get_user_by_phone(phone).await {
                    Ok(Some(_)) => {
                      // User already exists - send error
                      let auth_response = AuthResponse {
                        user: None,
                        session_token: String::new(),
                        refresh_token: String::new(),
                        status: AuthStatus::PhoneAlreadyUsed as i32,
                      };

                      let ws_msg = WsMessage {
                        message: Some(ws_message::Message::AuthResponse(auth_response)),
                      };

                      let bytes = ws_msg.encode_to_vec();
                      let _ = self.sender.send(bytes);
                    }
                    Ok(None) => {
                      // Generate verification code
                      let code = format!("{:06}", rand::random::<u32>() % 1000000);
                      let verification = VerificationCode {
                        id: uuid::Uuid::new_v4().to_string(),
                        phone_number: phone.clone(),
                        code,
                        expires_at: chrono::Utc::now().timestamp() + 300, // 5 minutes
                        attempts: 0,
                        verified: false,
                        created_at: chrono::Utc::now().timestamp(),
                      };

                      if let Err(e) = self.user_storage.save_verification_code(&verification).await {
                        eprintln!("Error saving verification code: {}", e);
                        let auth_response = AuthResponse {
                          user: None,
                          session_token: String::new(),
                          refresh_token: String::new(),
                          status: AuthStatus::InvalidCredentials as i32,
                        };

                        let ws_msg = WsMessage {
                          message: Some(ws_message::Message::AuthResponse(auth_response)),
                        };

                        let bytes = ws_msg.encode_to_vec();
                        let _ = self.sender.send(bytes);
                        return;
                      }

                      // TODO: Send actual SMS with verification code
                      println!("Verification code for {}: {}", phone, verification.code);

                      let response = AuthSendCodeResponse {
                        verification_id: verification.id,
                        timeout: 300,
                        is_new_user: true,
                      };

                      let ws_msg = WsMessage {
                        message: Some(ws_message::Message::AuthSendCodeResponse(response)),
                      };

                      let bytes = ws_msg.encode_to_vec();
                      let _ = self.sender.send(bytes);
                    }
                    Err(e) => {
                      eprintln!("Database error: {}", e);
                      let auth_response = AuthResponse {
                        user: None,
                        session_token: String::new(),
                        refresh_token: String::new(),
                        status: AuthStatus::InvalidCredentials as i32,
                      };

                      let ws_msg = WsMessage {
                        message: Some(ws_message::Message::AuthResponse(auth_response)),
                      };

                      let bytes = ws_msg.encode_to_vec();
                      let _ = self.sender.send(bytes);
                    }
                  }
                }
                _ => {
                  // Send error response for invalid phone number
                  let auth_response = AuthResponse {
                    user: None,
                    session_token: String::new(),
                    refresh_token: String::new(),
                    status: AuthStatus::InvalidCredentials as i32,
                  };

                  let ws_msg = WsMessage {
                    message: Some(ws_message::Message::AuthResponse(auth_response)),
                  };

                  let bytes = ws_msg.encode_to_vec();
                  let _ = self.sender.send(bytes);
                }
              }
            }
          }
        }
      }
      Some(ws_message::Message::AuthVerifyCode(verify_code)) => {
        if let Some(verification) = self.user_storage.get_verification_code(&verify_code.verification_id).await.unwrap() {
          if verification.code == verify_code.code && !verification.verified && verification.expires_at > chrono::Utc::now().timestamp() {
            let user = crate::storage::user::User {
              id: uuid::Uuid::new_v4().to_string(),
              username: verify_code.username,
              phone_number: Some(verification.phone_number.clone()),
              password_hash: String::new(), // Phone auth doesn't use password
              full_name: verify_code.full_name,
              avatar: None,
              created_at: chrono::Utc::now().timestamp(),
              updated_at: chrono::Utc::now().timestamp(),
            };

            match self.user_storage.create_user(&user).await {
              Ok(_) => {
                let mut verification = verification;
                verification.verified = true;
                let _ = self.user_storage.update_verification_code(&verification).await;

                let auth_token = self.auth_manager.generate_jwt(&user.id).await;
                let auth_response = AuthResponse {
                  user: Some(User {
                    id: user.id,
                    name: user.username.unwrap_or_else(|| user.phone_number.unwrap_or_default()),
                    avatar: String::new(),
                  }),
                  session_token: auth_token.token,
                  refresh_token: auth_token.refresh_token,
                  status: AuthStatus::Success as i32,
                };

                let ws_msg = WsMessage {
                  message: Some(ws_message::Message::AuthResponse(auth_response)),
                };

                let bytes = ws_msg.encode_to_vec();
                let _ = self.sender.send(bytes);
              }
              Err(e) => {
                eprintln!("Error creating user: {}", e);
                let auth_response = AuthResponse {
                  user: None,
                  session_token: String::new(),
                  refresh_token: String::new(),
                  status: AuthStatus::InvalidCredentials as i32,
                };

                let ws_msg = WsMessage {
                  message: Some(ws_message::Message::AuthResponse(auth_response)),
                };

                let bytes = ws_msg.encode_to_vec();
                let _ = self.sender.send(bytes);
              }
            }
          } else {
            let auth_response = AuthResponse {
              user: None,
              session_token: String::new(),
              refresh_token: String::new(),
              status: AuthStatus::CodeExpired as i32,
            };

            let ws_msg = WsMessage {
              message: Some(ws_message::Message::AuthResponse(auth_response)),
            };

            let bytes = ws_msg.encode_to_vec();
            let _ = self.sender.send(bytes);
          }
        } else {
          let auth_response = AuthResponse {
            user: None,
            session_token: String::new(),
            refresh_token: String::new(),
            status: AuthStatus::InvalidCredentials as i32,
          };

          let ws_msg = WsMessage {
            message: Some(ws_message::Message::AuthResponse(auth_response)),
          };

          let bytes = ws_msg.encode_to_vec();
          let _ = self.sender.send(bytes);
        }
      }
      _ => {
        println!("Received unknown message type: {:?}", msg);
      }
    }
  }

  pub async fn get_user_messages(&self) -> Result<Vec<Message>, Box<dyn std::error::Error + Send + Sync>> {
    self.user_storage.get_messages(&self.user_id).await
  }
}
