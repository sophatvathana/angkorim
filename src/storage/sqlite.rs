use async_trait::async_trait;
use sqlx::{ Row, SqlitePool, sqlite::SqlitePoolOptions, migrate };
use super::{ Storage, Message, MessageType };

pub struct SqliteStorage {
  pool: SqlitePool,
}

impl SqliteStorage {
  pub async fn new(path: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
    let pool = SqlitePoolOptions::new()
      .max_connections(5)
      .connect(&format!("sqlite:{}", path)).await?;

    migrate!("./src/storage/sqlite/migrations").run(&pool).await?;

    Ok(Self { pool })
  }
}

#[async_trait]
impl Storage for SqliteStorage {
  async fn save_message(
    &self,
    message: Message
  ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pool = self.pool.clone();

    sqlx
      ::query(
        "INSERT OR IGNORE INTO messages (id, sender, receiver, content, timestamp, message_type, version, origin_node)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
      )
      .bind(&message.id)
      .bind(&message.sender)
      .bind(&message.receiver)
      .bind(&message.content)
      .bind(message.timestamp)
      .bind(message.message_type as i32)
      .bind(message.version as i64)
      .bind(&message.origin_node)
      .execute(&pool).await?;

    Ok(())
  }

  async fn get_messages(
    &self,
    user: &str
  ) -> Result<Vec<Message>, Box<dyn std::error::Error + Send + Sync>> {
    let rows = sqlx
      ::query(
        r#"
            SELECT * FROM messages
            WHERE sender = ? OR receiver = ?
            ORDER BY timestamp DESC
            "#
      )
      .bind(user)
      .bind(user)
      .fetch_all(&self.pool).await?;

    Ok(
      rows
        .into_iter()
        .map(|row| Message {
          id: row.get("id"),
          sender: row.get("sender"),
          receiver: row.get("receiver"),
          content: row.get("content"),
          timestamp: row.get("timestamp"),
          message_type: MessageType::from(row.get::<i32, _>("message_type")),
          version: row.get("version"),
          origin_node: row.get("origin_node"),
        })
        .collect()
    )
  }
}
