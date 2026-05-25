use anyhow::Context;
use sqlx::migrate::MigrateDatabase;

pub const URL: &str = "sqlite:///opt/kuma-relay/test.db?mode=rwc";

pub struct Ledger {
    pool: sqlx::sqlite::SqlitePool,
}

impl Ledger {
    pub async fn new(path: impl Into<String>) -> anyhow::Result<Self> {
        let path = path.into();
        if !sqlx::Sqlite::database_exists(&path)
            .await
            .context("Failed to check if database exists")?
        {
            sqlx::Sqlite::create_database(&path)
                .await
                .context("Failed to create database")?;
        }

        let pool = sqlx::sqlite::SqlitePool::connect(&path)
            .await
            .context("Failed to create a connection to the database")?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            content BLOB NOT NULL
        )",
        )
        .execute(&pool)
        .await
        .context("Failed to create the table inside SQLite")?;

        Ok(Self { pool })
    }

    pub async fn insert(&mut self, body: Vec<u8>) -> anyhow::Result<()> {
        sqlx::query("INSERT INTO messages (content) VALUES (?)")
            .bind(body)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn insert_json<T>(&mut self, message: &T) -> anyhow::Result<()>
    where
        T: serde::Serialize,
    {
        let body =
            serde_json::to_vec(message).context("Failed to serialize message to a buffer")?;
        self.insert(body).await
    }
}
