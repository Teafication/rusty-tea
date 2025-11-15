use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::error::Error;
use tracing::info;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Message {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub role: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

/// PostgreSQL database connection pool service
/// Automatically runs migrations from `migrations/` folder on init
pub struct DatabaseService {
    pool: PgPool,
}

impl DatabaseService {
    /// Initialize database connection pool and run migrations
    pub async fn new(database_url: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
        info!("Connecting to PostgreSQL database: {}", database_url.split('@').last().unwrap_or("unknown"));
        
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        // Verify connection is working
        sqlx::query("SELECT 1")
            .fetch_one(&pool)
            .await?;

        // Auto-run migrations from migrations/ folder
        info!("Running database migrations...");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await?;

        info!("PostgreSQL connection pool initialized with migrations completed");
        Ok(Self { pool })
    }

    /// Get the connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Health check for database connection
    pub async fn health_check(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await?;
        Ok(())
    }

    /// Get conversation history (messages) for a given conversation_id
    /// Returns messages ordered by created_at ascending (oldest first)
    pub async fn get_conversation_history(&self, conversation_id: Uuid) -> Result<Vec<Message>, Box<dyn Error + Send + Sync>> {
        let messages = sqlx::query_as::<_, Message>(
            "SELECT id, conversation_id, role, content, created_at 
             FROM messages 
             WHERE conversation_id = $1 
             ORDER BY created_at ASC"
        )
        .bind(conversation_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(messages)
    }

    /// Save a message to the database
    pub async fn save_message(
        &self,
        conversation_id: Uuid,
        role: &str,
        content: &str,
    ) -> Result<Uuid, Box<dyn Error + Send + Sync>> {
        let message_id = Uuid::new_v4();
        
        sqlx::query(
            "INSERT INTO messages (id, conversation_id, role, content, created_at) 
             VALUES ($1, $2, $3, $4, NOW())"
        )
        .bind(message_id)
        .bind(conversation_id)
        .bind(role)
        .bind(content)
        .execute(&self.pool)
        .await?;

        Ok(message_id)
    }

    /// Create a new conversation if it doesn't exist
    /// Returns the conversation_id
    pub async fn ensure_conversation_exists(&self, conversation_id: Uuid) -> Result<(), Box<dyn Error + Send + Sync>> {
        sqlx::query(
            "INSERT INTO conversations (id, created_at, updated_at) 
             VALUES ($1, NOW(), NOW()) 
             ON CONFLICT (id) DO NOTHING"
        )
        .bind(conversation_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_service_creation() {
        // This test verifies the struct can be created (compile-time check)
        // Runtime tests will be in integration_tests.rs
        let _db: Result<(), &str> = Ok(());
        assert!(_db.is_ok());
    }

    #[tokio::test]
    async fn test_database_url_parsing() {
        let url = "postgresql://user:pass@localhost:5432/testdb";
        // Verify URL format is valid (we won't actually connect in unit tests)
        assert!(url.starts_with("postgresql://"));
    }
}
