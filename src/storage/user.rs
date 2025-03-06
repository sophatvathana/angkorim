use async_trait::async_trait;
use sqlx::{ Row, SqlitePool };
use sha2::{ Sha256, Digest };
use std::time::{ SystemTime, UNIX_EPOCH };

#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub username: Option<String>,
    pub phone_number: Option<String>,
    pub password_hash: String,
    pub full_name: Option<String>,
    pub avatar: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug)]
pub struct VerificationCode {
    pub id: String,
    pub phone_number: String,
    pub code: String,
    pub expires_at: i64,
    pub attempts: i32,
    pub verified: bool,
    pub created_at: i64,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub id: String,
    pub sender: String,
    pub receiver: String,
    pub content: String,
    pub timestamp: i64,
    pub message_type: i32,
    pub version: i64,
    pub origin_node: String,
    pub delivered: bool,
    pub read: bool,
    pub delivered_at: Option<i64>,
    pub read_at: Option<i64>,
}

#[async_trait]
pub trait UserStorage: Send + Sync {
    async fn create_user(&self, user: &User) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_user_by_phone(&self, phone: &str) -> Result<Option<User>, Box<dyn std::error::Error + Send + Sync>>;
    async fn save_verification_code(&self, code: &VerificationCode) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn get_verification_code(&self, phone: &str) -> Result<Option<VerificationCode>, Box<dyn std::error::Error + Send + Sync>>;
    async fn update_verification_code(&self, code: &VerificationCode) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    // Message related methods
    async fn save_message(&self, message: &Message) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn get_messages(&self, user_id: &str) -> Result<Vec<Message>, Box<dyn std::error::Error + Send + Sync>>;
    async fn mark_message_delivered(&self, message_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn mark_message_read(&self, message_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn get_unread_messages(&self, user_id: &str) -> Result<Vec<Message>, Box<dyn std::error::Error + Send + Sync>>;
}

pub struct SqliteUserStorage {
    pool: SqlitePool,
}

impl SqliteUserStorage {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub fn hash_password(password: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

#[async_trait]
impl UserStorage for SqliteUserStorage {
    async fn create_user(&self, user: &User) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        sqlx::query(
            "INSERT INTO users (id, username, phone_number, password_hash, full_name, avatar, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&user.id)
        .bind(&user.username)
        .bind(&user.phone_number)
        .bind(&user.password_hash)
        .bind(&user.full_name)
        .bind(&user.avatar)
        .bind(user.created_at)
        .bind(user.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, Box<dyn std::error::Error + Send + Sync>> {
        let row = sqlx::query(
            "SELECT * FROM users WHERE username = ?"
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| User {
            id: r.get("id"),
            username: r.get("username"),
            phone_number: r.get("phone_number"),
            password_hash: r.get("password_hash"),
            full_name: r.get("full_name"),
            avatar: r.get("avatar"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    async fn get_user_by_phone(&self, phone: &str) -> Result<Option<User>, Box<dyn std::error::Error + Send + Sync>> {
        let row = sqlx::query(
            "SELECT * FROM users WHERE phone_number = ?"
        )
        .bind(phone)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| User {
            id: r.get("id"),
            username: r.get("username"),
            phone_number: r.get("phone_number"),
            password_hash: r.get("password_hash"),
            full_name: r.get("full_name"),
            avatar: r.get("avatar"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    async fn save_verification_code(&self, code: &VerificationCode) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        sqlx::query(
            "INSERT INTO verification_codes (id, phone_number, code, expires_at, attempts, verified, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&code.id)
        .bind(&code.phone_number)
        .bind(&code.code)
        .bind(code.expires_at)
        .bind(code.attempts)
        .bind(code.verified)
        .bind(code.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_verification_code(&self, phone: &str) -> Result<Option<VerificationCode>, Box<dyn std::error::Error + Send + Sync>> {
        let row = sqlx::query(
            "SELECT * FROM verification_codes
             WHERE phone_number = ?
             AND verified = FALSE
             AND expires_at > ?
             ORDER BY created_at DESC
             LIMIT 1"
        )
        .bind(phone)
        .bind(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| VerificationCode {
            id: r.get("id"),
            phone_number: r.get("phone_number"),
            code: r.get("code"),
            expires_at: r.get("expires_at"),
            attempts: r.get("attempts"),
            verified: r.get("verified"),
            created_at: r.get("created_at"),
        }))
    }

    async fn update_verification_code(&self, code: &VerificationCode) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        sqlx::query(
            "UPDATE verification_codes
             SET attempts = ?, verified = ?
             WHERE id = ?"
        )
        .bind(code.attempts)
        .bind(code.verified)
        .bind(&code.id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn save_message(&self, message: &Message) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        sqlx::query(
            "INSERT INTO messages (id, sender, receiver, content, timestamp, message_type, version, origin_node, delivered, read, delivered_at, read_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&message.id)
        .bind(&message.sender)
        .bind(&message.receiver)
        .bind(&message.content)
        .bind(message.timestamp)
        .bind(message.message_type)
        .bind(message.version)
        .bind(&message.origin_node)
        .bind(message.delivered)
        .bind(message.read)
        .bind(message.delivered_at)
        .bind(message.read_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_messages(&self, user_id: &str) -> Result<Vec<Message>, Box<dyn std::error::Error + Send + Sync>> {
        let rows = sqlx::query(
            "SELECT * FROM messages WHERE sender = ? OR receiver = ? ORDER BY timestamp DESC"
        )
        .bind(user_id)
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| Message {
            id: r.get("id"),
            sender: r.get("sender"),
            receiver: r.get("receiver"),
            content: r.get("content"),
            timestamp: r.get("timestamp"),
            message_type: r.get("message_type"),
            version: r.get("version"),
            origin_node: r.get("origin_node"),
            delivered: r.get("delivered"),
            read: r.get("read"),
            delivered_at: r.get("delivered_at"),
            read_at: r.get("read_at"),
        }).collect())
    }

    async fn mark_message_delivered(&self, message_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        sqlx::query(
            "UPDATE messages
             SET delivered = TRUE
             WHERE id = ?"
        )
        .bind(message_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn mark_message_read(&self, message_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        sqlx::query(
            "UPDATE messages
             SET read = TRUE
             WHERE id = ?"
        )
        .bind(message_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_unread_messages(&self, user_id: &str) -> Result<Vec<Message>, Box<dyn std::error::Error + Send + Sync>> {
        let rows = sqlx::query(
            "SELECT * FROM messages
             WHERE receiver = ?
             AND message_type = ?
             AND id NOT IN (
                SELECT content FROM messages
                WHERE message_type = ?
                AND content LIKE 'read:%'
             )
             ORDER BY timestamp DESC"
        )
        .bind(user_id)
        .bind(crate::proto::MessageType::Direct as i32)
        .bind(crate::proto::MessageType::ReadReceipt as i32)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| Message {
            id: r.get("id"),
            sender: r.get("sender"),
            receiver: r.get("receiver"),
            content: r.get("content"),
            timestamp: r.get("timestamp"),
            message_type: r.get("message_type"),
            version: r.get("version"),
            origin_node: r.get("origin_node"),
            delivered: r.get("delivered"),
            read: r.get("read"),
            delivered_at: r.get("delivered_at"),
            read_at: r.get("read_at"),
        }).collect())
    }
}
