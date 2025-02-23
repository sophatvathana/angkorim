use std::string::FromUtf8Error;

pub fn message_to_bytes(message: &str) -> Vec<u8> {
  message.as_bytes().to_vec()
}

pub fn bytes_to_message(bytes: &[u8]) -> Result<String, FromUtf8Error> {
  String::from_utf8(bytes.to_vec())
}
