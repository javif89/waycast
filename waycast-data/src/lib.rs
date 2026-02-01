pub use sqlx;
use thiserror::Error;
use tracing::info;
use waycast_core::LauncherItem;

use std::{path::Path, str::FromStr, time::Duration};

use serde::{Deserialize, Serialize};
use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};

#[derive(Debug, Error)]
pub enum DataError {
    #[error("Database error {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Query error: {0}")]
    QueryError(String),
}

pub struct DB {
    pool: SqlitePool,
}

pub fn wal_connection(path: impl AsRef<Path>) -> SqliteConnectOptions {
    SqliteConnectOptions::from_str(path.as_ref().to_string_lossy().as_ref())
        .expect("Failed lol")
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
        .foreign_keys(true)
        .busy_timeout(Duration::from_secs(10))
}

pub fn ro_connection(path: impl AsRef<Path>) -> SqliteConnectOptions {
    SqliteConnectOptions::from_str(path.as_ref().to_string_lossy().as_ref())
        .expect("Failed lol")
        .read_only(true)
        .foreign_keys(true)
        .busy_timeout(Duration::from_secs(10))
}

#[derive(Debug, sqlx::Type, Deserialize, Serialize)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum ItemKind {
    DesktopEntry,
    File,
    Project,
    Unknown,
}

impl From<String> for ItemKind {
    fn from(value: String) -> Self {
        match value.as_str() {
            "desktopentry" => Self::DesktopEntry,
            "file" => Self::File,
            "project" => Self::Project,
            _ => Self::Unknown,
        }
    }
}

impl From<waycast_core::ItemKind> for ItemKind {
    fn from(value: waycast_core::ItemKind) -> Self {
        match value {
            waycast_core::ItemKind::DesktopEntry => Self::DesktopEntry,
            waycast_core::ItemKind::File => Self::File,
            waycast_core::ItemKind::Project => Self::Project,
            waycast_core::ItemKind::Unknown => Self::Unknown,
        }
    }
}

impl From<ItemKind> for waycast_core::ItemKind {
    fn from(value: ItemKind) -> Self {
        match value {
            ItemKind::DesktopEntry => waycast_core::ItemKind::DesktopEntry,
            ItemKind::File => waycast_core::ItemKind::File,
            ItemKind::Project => waycast_core::ItemKind::Project,
            ItemKind::Unknown => waycast_core::ItemKind::Unknown,
        }
    }
}

#[derive(sqlx::FromRow, Debug)]
pub struct ItemRow {
    pub id: String,
    pub kind: ItemKind,
    pub title: String,
    pub description: Option<String>,
    pub icon: String,
}

impl From<LauncherItem> for ItemRow {
    fn from(value: LauncherItem) -> Self {
        Self {
            id: value.id,
            kind: value.kind.into(),
            title: value.title,
            description: value.description,
            icon: value.icon,
        }
    }
}

impl From<ItemRow> for LauncherItem {
    fn from(value: ItemRow) -> Self {
        Self {
            id: value.id,
            kind: value.kind.into(),
            title: value.title,
            description: value.description,
            icon: value.icon,
        }
    }
}

impl DB {
    pub async fn open(connection_options: SqliteConnectOptions) -> Self {
        let pool = SqlitePoolOptions::new()
            .max_connections(8)
            .connect_with(connection_options)
            .await
            .expect("Failed to make pool");

        sqlx::migrate!()
            .run(&pool)
            .await
            .expect("Failed to migrate");

        Self { pool }
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    async fn reset_items_staging(&self) -> Result<(), DataError> {
        let result = sqlx::query!("delete from items_staging")
            .execute(&self.pool)
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(DataError::QueryError(err.to_string())),
        }
    }

    /// Insert items into the database. Steps are as follows:
    ///
    /// 1. Truncate the items_staging table
    /// 2. Add items to items_staging
    /// 3. Delete any items in items not found in items_staging (by item_id + kind)
    /// 4. Add any items to the items table present in items_staging and not in items
    ///
    /// This approach prevents having a situation where the user
    /// opens waycast to find no items while we're in the middle
    /// of this operation.
    pub async fn insert_items(&self, items: Vec<ItemRow>) -> Result<(), DataError> {
        self.reset_items_staging().await?;

        let mut tx = self.pool.begin().await?;
        for item in items {
            sqlx::query!(
                r#"
                    insert into items_staging (
                        item_id,
                        kind,
                        title,
                        description,
                        icon
                    )
                    values (?, ?, ?, ?, ?)
                    on conflict(item_id, kind) do update set
                    title = excluded.title,
                    description = excluded.description,
                    icon = excluded.icon
                "#,
                item.id,
                item.kind,
                item.title,
                item.description,
                item.icon
            )
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;

        // Delete anything not in staging
        sqlx::query!(
            r#"
            delete from items
            where not exists (
                select 1 from items_staging iss 
                where iss.item_id = items.item_id
                and iss.kind = items.kind
            );
            "#
        )
        .execute(&self.pool)
        .await?;

        // Insert anything we don't have from staging
        sqlx::query!(
            r#"
                insert into items (
                    item_id,
                    kind,
                    title,
                    description,
                    icon
                )
                select
                    iss.item_id,
                    iss.kind,
                    iss.title,
                    iss.description,
                    iss.icon
                from items_staging iss
                where not exists (
                    select 1 from items
                    where items.item_id = iss.item_id
                    and items.kind = iss.kind
                );
            "#
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_items(&self, kind: Option<ItemKind>) -> Result<Vec<ItemRow>, DataError> {
        let items = sqlx::query_as!(
            ItemRow,
            r#"
            select
                item_id as id,
                kind,
                title,
                description,
                icon
            from items 
            where (?1 is null or kind = ?1)
        "#,
            kind
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(items)
    }

    pub async fn search(&self, query: String) -> Result<Vec<ItemRow>, DataError> {
        let fts_query = build_fts_query(&query);

        info!("Searching fts index for {}", fts_query);

        let results = sqlx::query_as!(
            ItemRow,
            r#"
                select
                    i.item_id as id,
                    i.kind,
                    i.title,
                    i.description,
                    i.icon
                from items_fts
                join items i on i.id = items_fts.rowid
                where items_fts match ?1
                order by bm25(items_fts, 10.0, 3.0) desc 
            "#,
            fts_query
        )
        .fetch_all(&self.pool)
        .await?;

        info!("Found {} items", results.len());

        Ok(results)
    }
}

fn build_fts_query(input: &str) -> String {
    let cleaned = input
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect::<String>();

    cleaned
        .split_whitespace()
        .map(|t| format!("{t}*"))
        .collect::<Vec<_>>()
        .join(" ")
}
