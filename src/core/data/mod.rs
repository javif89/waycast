use thiserror::Error;

use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};

pub mod cache;
pub mod items;

pub use items::LauncherItemRepository;

use self::cache::CacheRepository;

#[derive(Debug, Error)]
pub enum DataError {
    #[error("Failed to create database directory {}: {source}", path.display())]
    CreateDatabaseDirectory {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Database error {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Failed to open database {}: {source}", path.display())]
    OpenDatabase {
        path: PathBuf,
        #[source]
        source: sqlx::Error,
    },

    #[error("Failed to migrate database {}: {source}", path.display())]
    MigrationError {
        path: PathBuf,
        #[source]
        source: sqlx::migrate::MigrateError,
    },

    #[error("Query error: {0}")]
    QueryError(String),

    #[error("Failed to serialize/deserialize value")]
    SerializationError(#[from] serde_json::Error),
}

#[derive(Clone)]
pub struct WaycastData {
    pool: SqlitePool,
}

impl WaycastData {
    pub async fn read_only_connection(path: impl AsRef<Path>) -> Result<Self, DataError> {
        let path = path.as_ref();
        let opts = SqliteConnectOptions::new()
            .filename(path)
            .read_only(true)
            .foreign_keys(true)
            .busy_timeout(Duration::from_secs(10));

        let pool = open(opts).await.map_err(|source| DataError::OpenDatabase {
            path: path.to_path_buf(),
            source,
        })?;

        Ok(Self { pool })
    }

    pub async fn writeable_connection(path: impl AsRef<Path>) -> Result<Self, DataError> {
        let path = path.as_ref();
        create_database_directory(path).await?;

        let opts = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
            .foreign_keys(true)
            .busy_timeout(Duration::from_secs(10));

        let pool = open(opts).await.map_err(|source| DataError::OpenDatabase {
            path: path.to_path_buf(),
            source,
        })?;

        sqlx::migrate!()
            .run(&pool)
            .await
            .map_err(|source| DataError::MigrationError {
                path: path.to_path_buf(),
                source,
            })?;

        Ok(Self { pool })
    }

    pub fn items(&self) -> LauncherItemRepository {
        LauncherItemRepository {
            pool: self.pool.clone(),
        }
    }

    pub fn cache(&self) -> CacheRepository {
        CacheRepository {
            pool: self.pool.clone(),
        }
    }
}

async fn create_database_directory(database_path: &Path) -> Result<(), DataError> {
    let Some(parent) = database_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    else {
        return Ok(());
    };

    tokio::fs::create_dir_all(parent)
        .await
        .map_err(|source| DataError::CreateDatabaseDirectory {
            path: parent.to_path_buf(),
            source,
        })
}

async fn open(connection_options: SqliteConnectOptions) -> Result<SqlitePool, sqlx::Error> {
    SqlitePoolOptions::new()
        .max_connections(8)
        .connect_with(connection_options)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn creates_missing_database_directories_and_runs_migrations() {
        let root = tempfile::tempdir().expect("temporary directory");
        let database_path = root.path().join("nested/data/waycast.db");

        let database = WaycastData::writeable_connection(&database_path)
            .await
            .expect("initialize database");

        assert!(database_path.is_file());
        let items_table: i64 = sqlx::query_scalar(
            "select count(*) from sqlite_master where type = 'table' and name = 'items'",
        )
        .fetch_one(&database.pool)
        .await
        .expect("query migrated schema");
        assert_eq!(items_table, 1);
    }

    #[tokio::test]
    async fn database_initialization_is_idempotent() {
        let root = tempfile::tempdir().expect("temporary directory");
        let database_path = root.path().join("data/waycast.db");

        WaycastData::writeable_connection(&database_path)
            .await
            .expect("first initialization");
        let database = WaycastData::writeable_connection(&database_path)
            .await
            .expect("second initialization");

        let applied_migrations: i64 = sqlx::query_scalar("select count(*) from _sqlx_migrations")
            .fetch_one(&database.pool)
            .await
            .expect("query migration history");
        assert_eq!(applied_migrations, 1);
    }

    #[tokio::test]
    async fn reports_the_parent_directory_that_could_not_be_created() {
        let root = tempfile::tempdir().expect("temporary directory");
        let invalid_parent = root.path().join("not-a-directory");
        std::fs::write(&invalid_parent, "file").expect("create blocking file");

        let result = WaycastData::writeable_connection(invalid_parent.join("waycast.db")).await;

        match result {
            Err(DataError::CreateDatabaseDirectory { path, .. }) => {
                assert_eq!(path, invalid_parent);
            }
            Err(error) => panic!("unexpected error: {error}"),
            Ok(_) => panic!("database initialization unexpectedly succeeded"),
        }
    }

    #[tokio::test]
    async fn skips_directory_creation_for_a_bare_filename() {
        create_database_directory(Path::new("waycast.db"))
            .await
            .expect("bare filename should not require a parent directory");
    }
}
