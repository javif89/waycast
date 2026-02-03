use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::DataError;
use serde::{Serialize, de::DeserializeOwned};
use sqlx::SqlitePool;

pub struct CacheRepository {
    pub pool: SqlitePool,
}

fn now_epoch_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before unix epoch?")
        .as_secs() as i64
}

/// ttl: Some(Duration) => expires_at = now + ttl
/// ttl: None => no expiration stored (NULL in DB)
fn compute_expires_at(ttl: Option<Duration>) -> Option<i64> {
    ttl.map(|ttl| now_epoch_secs() + ttl.as_secs() as i64)
}

fn is_expired(expires_at: Option<i64>) -> bool {
    matches!(expires_at, Some(ts) if ts <= now_epoch_secs())
}

#[derive(sqlx::FromRow, Debug)]
pub struct CacheRow {
    pub key: String,
    pub value: String,
    pub expires_at: Option<i64>,
}

impl CacheRepository {
    pub async fn put<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> Result<(), DataError> {
        let serialized_value = serde_json::to_string(value)?;
        let expires_at = compute_expires_at(ttl);

        sqlx::query!(
            r#"
                insert into cache (key, value, expires_at)
                values (?, ?, ?)
                on conflict(key) do update set
                value = excluded.value,
                expires_at = excluded.expires_at
            "#,
            key,
            serialized_value,
            expires_at
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, DataError> {
        let result = sqlx::query_as!(
            CacheRow,
            r#"
            select
                key,
                value,
                expires_at
            from cache
            where key = ?1
        "#,
            key
        )
        .fetch_optional(&self.pool)
        .await?;

        match result {
            Some(r) => {
                // Check if it's expired. If so,
                // delete and return None
                if is_expired(r.expires_at) {
                    self.delete(key).await;
                    return Ok(None);
                }

                let value: T = serde_json::from_str(&r.value)?;

                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    pub async fn delete(&self, key: &str) -> Result<(), DataError> {
        sqlx::query!("delete from cache where key = ?1", key)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn remember<T: Serialize + DeserializeOwned, F: FnOnce() -> T>(
        &self,
        key: &str,
        ttl: Option<Duration>,
        value: F,
    ) -> Result<T, DataError> {
        if let Some(v) = self.get::<T>(key).await? {
            return Ok(v);
        }

        let v = value();

        self.put(key, &v, ttl).await?;

        Ok(v)
    }
}
