pub use sqlx;
use thiserror::Error;

use std::{path::Path, str::FromStr, time::Duration};

use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};

pub mod cache;
pub mod icons;
pub mod items;

pub use items::LauncherItemRepository;

use crate::{cache::CacheRepository, icons::IconRepository};

#[derive(Debug, Error)]
pub enum DataError {
    #[error("Database error {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Query error: {0}")]
    QueryError(String),

    #[error("Failed to serialize/deserialize value")]
    SerializationError(#[from] serde_json::Error),
}

pub struct WaycastData {
    pool: SqlitePool,
}

impl WaycastData {
    pub async fn read_only_connection(path: impl AsRef<Path>) -> Self {
        let opts = SqliteConnectOptions::from_str(path.as_ref().to_string_lossy().as_ref())
            .expect("Failed lol")
            .read_only(true)
            .foreign_keys(true)
            .busy_timeout(Duration::from_secs(10));

        let pool = open(opts).await;

        Self { pool }
    }

    pub async fn writeable_connection(path: impl AsRef<Path>) -> Self {
        let opts = SqliteConnectOptions::from_str(path.as_ref().to_string_lossy().as_ref())
            .expect("Failed lol")
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
            .foreign_keys(true)
            .busy_timeout(Duration::from_secs(10));

        let pool = open(opts).await;

        sqlx::migrate!()
            .run(&pool)
            .await
            .expect("Failed to migrate");

        Self { pool }
    }

    pub fn items(&self) -> LauncherItemRepository {
        LauncherItemRepository {
            pool: self.pool.clone(),
        }
    }

    pub fn icons(&self) -> IconRepository {
        IconRepository {
            pool: self.pool.clone(),
        }
    }

    pub fn cache(&self) -> CacheRepository {
        CacheRepository {
            pool: self.pool.clone(),
        }
    }
}

async fn open(connection_options: SqliteConnectOptions) -> SqlitePool {
    SqlitePoolOptions::new()
        .max_connections(8)
        .connect_with(connection_options)
        .await
        .expect("Failed to make pool")
}
