pub mod errors;
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::path::Path;
use std::sync::OnceLock;
use std::time::{Duration, SystemTime};

use crate::cache::errors::CacheError;

const CACHE_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("cache");
static CACHE_SINGLETON: OnceLock<Cache> = OnceLock::new();

pub struct CacheTTL {}

impl CacheTTL {
    pub fn hours(hours: u64) -> Option<Duration> {
        return Some(Duration::from_secs(hours * 60 * 60));
    }

    pub fn minutes(minutes: u64) -> Option<Duration> {
        return Some(Duration::from_secs(minutes * 60));
    }
}

#[derive(Serialize, Deserialize)]
struct CacheEntry<T> {
    data: T,
    expires_at: Option<SystemTime>,
}

pub struct Cache {
    db: Database,
}

pub fn get() -> &'static Cache {
    CACHE_SINGLETON.get_or_init(|| {
        let cache_path = waycast_config::cache_path("waycast_cache.db")
            .unwrap_or_else(|| std::env::current_dir().unwrap().join("waycast_cache.db"));
        
        // Ensure cache directory exists
        if let Some(parent) = cache_path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                eprintln!("Warning: Failed to create cache directory {}: {}", parent.display(), e);
            }
        }
        
        new(cache_path).expect("Failed to initialize cache :(")
    })
}

// Get an existing cache at the given path or
// create it if it doesn't exist
pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Cache, CacheError> {
    let db = Database::create(db_path)?;

    // Initialize the table if it doesn't exist
    let write_txn = db.begin_write()?;
    {
        let _ = write_txn.open_table(CACHE_TABLE)?;
    }
    write_txn.commit()?;

    Ok(Cache { db })
}

impl Cache {
    /// Cache a value with an optional TTL. If TTL is None, the value never expires.
    pub fn remember_with_ttl<T>(
        &self,
        key: &str,
        ttl: Option<Duration>,
        compute: impl FnOnce() -> T,
    ) -> Result<T, CacheError>
    where
        T: Serialize + DeserializeOwned + Clone,
    {
        // Check if caching is disabled via environment variable
        if std::env::var("WAYCAST_NO_CACHE").is_ok() {
            return Ok(compute());
        }

        // Try to get from cache first
        if let Some(entry) = self.get_cached_entry::<T>(key)? {
            // Check if entry has expired
            if let Some(expires_at) = entry.expires_at {
                if SystemTime::now() < expires_at {
                    return Ok(entry.data);
                }
                // Entry has expired, continue to recompute
            } else {
                // No expiration, return cached data
                return Ok(entry.data);
            }
        }

        // Not in cache or expired, compute the value
        let data = compute();
        let expires_at = ttl.map(|duration| SystemTime::now() + duration);
        let entry = CacheEntry {
            data: data.clone(),
            expires_at,
        };

        // Store in cache
        self.store_entry(key, &entry)?;

        Ok(data)
    }

    /// Cache a value with no expiration
    pub fn remember<T>(&self, key: &str, compute: impl FnOnce() -> T) -> Result<T, CacheError>
    where
        T: Serialize + DeserializeOwned + Clone,
    {
        self.remember_with_ttl(key, None, compute)
    }

    /// Get a cached value if it exists and hasn't expired
    pub fn get<T>(&self, key: &str) -> Result<Option<T>, CacheError>
    where
        T: Serialize + DeserializeOwned,
    {
        if let Some(entry) = self.get_cached_entry::<T>(key)? {
            // Check if entry has expired
            if let Some(expires_at) = entry.expires_at {
                if SystemTime::now() < expires_at {
                    return Ok(Some(entry.data));
                }
                // Entry has expired, remove it and return None
                self.forget(key)?;
                return Ok(None);
            } else {
                // No expiration, return cached data
                return Ok(Some(entry.data));
            }
        }
        Ok(None)
    }

    /// Store a value in the cache with optional TTL
    pub fn put<T>(&self, key: &str, value: T, ttl: Option<Duration>) -> Result<(), CacheError>
    where
        T: Serialize,
    {
        let expires_at = ttl.map(|duration| SystemTime::now() + duration);
        let entry = CacheEntry {
            data: value,
            expires_at,
        };
        self.store_entry(key, &entry)
    }

    /// Remove a key from the cache
    pub fn forget(&self, key: &str) -> Result<(), CacheError> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(CACHE_TABLE)?;
            table.remove(key)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// Clear all cached entries
    pub fn clear(&self) -> Result<(), CacheError> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(CACHE_TABLE)?;
            // Remove all entries
            let keys: Vec<String> = {
                let mut keys = Vec::new();
                let mut iter = table.iter()?;
                while let Some(Ok((key, _))) = iter.next() {
                    keys.push(key.value().to_string());
                }
                keys
            };

            for key in keys {
                table.remove(key.as_str())?;
            }
        }
        write_txn.commit()?;
        Ok(())
    }

    fn get_cached_entry<T>(&self, key: &str) -> Result<Option<CacheEntry<T>>, CacheError>
    where
        T: DeserializeOwned,
    {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(CACHE_TABLE)?;

        if let Some(cached_bytes) = table.get(key)? {
            match bincode::deserialize::<CacheEntry<T>>(cached_bytes.value()) {
                Ok(entry) => Ok(Some(entry)),
                Err(_) => {
                    // Failed to deserialize, probably corrupted or wrong format
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    }

    fn store_entry<T>(&self, key: &str, entry: &CacheEntry<T>) -> Result<(), CacheError>
    where
        T: Serialize,
    {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(CACHE_TABLE)?;
            let serialized = bincode::serialize(entry)
                .map_err(|e| CacheError::SerializationError(e.to_string()))?;
            table.insert(key, serialized.as_slice())?;
        }
        write_txn.commit()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tempfile::NamedTempFile;

    #[test]
    fn test_cache_remember() {
        let temp_file = NamedTempFile::new().unwrap();
        let cache = new(temp_file.path()).unwrap();

        let result = cache
            .remember("test_key", || "computed_value".to_string())
            .unwrap();
        assert_eq!(result, "computed_value");

        // Second call should return cached value
        let result2 = cache
            .remember("test_key", || "different_value".to_string())
            .unwrap();
        assert_eq!(result2, "computed_value");
    }

    #[test]
    fn test_cache_ttl() {
        let temp_file = NamedTempFile::new().unwrap();
        let cache = new(temp_file.path()).unwrap();

        // Cache with very short TTL
        let result = cache
            .remember_with_ttl("ttl_key", Some(Duration::from_millis(1)), || {
                "cached_value".to_string()
            })
            .unwrap();
        assert_eq!(result, "cached_value");

        // Wait for expiration
        std::thread::sleep(Duration::from_millis(10));

        // Should recompute after expiration
        let result2 = cache
            .remember_with_ttl("ttl_key", Some(Duration::from_millis(1)), || {
                "new_value".to_string()
            })
            .unwrap();
        assert_eq!(result2, "new_value");
    }
}
