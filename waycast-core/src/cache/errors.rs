use redb::{CommitError, DatabaseError, StorageError, TableError, TransactionError};

#[derive(Debug)]
pub enum CacheError {
    DatabaseError(String),
    SerializationError(String),
    Other(String),
}

impl From<DatabaseError> for CacheError {
    fn from(err: DatabaseError) -> Self {
        CacheError::DatabaseError(err.to_string())
    }
}

impl From<TransactionError> for CacheError {
    fn from(err: TransactionError) -> Self {
        CacheError::DatabaseError(err.to_string())
    }
}

impl From<TableError> for CacheError {
    fn from(err: TableError) -> Self {
        CacheError::DatabaseError(err.to_string())
    }
}

impl From<StorageError> for CacheError {
    fn from(err: StorageError) -> Self {
        CacheError::DatabaseError(err.to_string())
    }
}

impl From<CommitError> for CacheError {
    fn from(err: CommitError) -> Self {
        CacheError::DatabaseError(err.to_string())
    }
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheError::DatabaseError(e) => write!(f, "Database error: {}", e),
            CacheError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            CacheError::Other(msg) => write!(f, "Cache error: {}", msg),
        }
    }
}

impl std::error::Error for CacheError {}
