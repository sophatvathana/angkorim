pub mod proto {
  include!(concat!(env!("OUT_DIR"), "/chat.rs"));
  include!(concat!(env!("OUT_DIR"), "/protocol.rs"));
}
use proto::{ ChatMessage, EncryptionType, HandshakeMessage, MessageType };
use tokio_tungstenite::{ connect_async, tungstenite::Message };
use futures::prelude::*;
use prost::Message as _;
use std::{ io::{ self, Write }, string::FromUtf8Error };
use ed25519_dalek::{ Signature, Signer, SigningKey };
use rand_core::{ RngCore, OsRng };
pub fn message_to_bytes(message: &str) -> Vec<u8> {
  message.as_bytes().to_vec()
}

pub fn bytes_to_message(bytes: &[u8]) -> Result<String, FromUtf8Error> {
  String::from_utf8(bytes.to_vec())
}
#[tokio::main]
async fn main() {
  let args: Vec<String> = std::env::args().collect();
  if args.len() != 3 {
    println!("Usage: {} <node_port> <user_id>", args[0]);
    return;
  }

  let port = args[1].parse::<u16>().unwrap();
  let user_id = args[2].clone();

  let url = format!("ws://127.0.0.1:{}/ws", port);
  let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
  println!("Connected to node on port {}", port);
  println!("You are user: {}", user_id);

  let (mut write, read) = ws_stream.split();
  let mut rng = OsRng;
  // Generate keypair
  let signing_key = SigningKey::generate(&mut rng);
  let verifying_key = signing_key.verifying_key();

  // Create and sign handshake
  let timestamp = std::time::SystemTime
    ::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_secs();

  let message = format!("{}{}", timestamp, user_id);
  let signature = signing_key.sign(message.as_bytes());

  // Send handshake message
  let handshake = HandshakeMessage {
    user_id: user_id.clone(),
    public_key: verifying_key.to_bytes().to_vec(),
    signature: signature.to_bytes().to_vec(),
    timestamp,
  };
  let bytes = handshake.encode_to_vec();
  if let Err(e) = write.send(Message::Binary(bytes.into())).await {
    eprintln!("Error sending handshake: {}", e);
    return;
  }
  println!("Sent handshake for user: {}", user_id);

  // Handle incoming messages
  let receive_handle = tokio::spawn(async move {
    let mut read = read;
    while let Some(Ok(msg)) = read.next().await {
      if let Message::Binary(bytes) = msg {
        if let Ok(chat_msg) = ChatMessage::decode(&bytes[..]) {
          println!(
            "\nReceived message: [From: {}, To: {}, Content: {}]\n> ",
            chat_msg.sender,
            chat_msg.receiver,
            bytes_to_message(&chat_msg.content).unwrap()
          );
          io::stdout().flush().unwrap();
        }
      }
    }
  });

  // Handle outgoing messages
  let send_handle = tokio::spawn(async move {
    let mut write = write;
    loop {
      print!("> ");
      io::stdout().flush().unwrap();

      let mut input = String::new();
      io::stdin().read_line(&mut input).unwrap();
      let input = input.trim();

      if input == "/quit" {
        break;
      }

      // Format: @receiver message
      if let Some(receiver) = input.strip_prefix('@') {
        if let Some((receiver, content)) = receiver.split_once(' ') {
          let id = uuid::Uuid::new_v4().to_string();
          let timestamp = chrono::Utc::now().timestamp() as u64;

          // Create signature first
          let message = format!("{}{}{}{}{}", id, user_id, receiver, content, timestamp);
          let signature = signing_key.sign(message.as_bytes()).to_bytes().to_vec();

          let msg = ChatMessage {
            id,
            sender: user_id.clone(),
            receiver: receiver.to_string(),
            content: message_to_bytes(&content),
            timestamp,
            message_type: MessageType::Direct as i32,
            version_vector: String::new(),
            signature,
            encryption_type: EncryptionType::None as i32,
            metadata: None,
          };

          let bytes = msg.encode_to_vec();
          if let Err(e) = write.send(Message::Binary(bytes.into())).await {
            eprintln!("Error sending message: {}", e);
          }
        }
      }
    }
  });

  tokio::select! {
        _ = receive_handle => {}
        _ = send_handle => {}
    }
}
