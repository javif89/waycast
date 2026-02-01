use sqlx::SqlitePool;

use crate::DataError;

#[derive(sqlx::FromRow, Debug)]
pub struct IconRow {
    pub id: Option<i64>,
    pub name: String,
    pub path: String,
}

pub struct IconRepository {
    pub pool: SqlitePool,
}

impl IconRepository {
    pub async fn insert(&self, items: Vec<IconRow>) -> Result<(), DataError> {
        let mut tx = self.pool.begin().await?;
        for item in items {
            sqlx::query!(
                r#"
                    insert into icons (
                        name,
                        path
                    )
                    values (?, ?)
                    on conflict(name) do update set
                    path = excluded.path
                "#,
                item.name,
                item.path
            )
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;

        Ok(())
    }

    pub async fn get(&self, name: String) -> Result<IconRow, DataError> {
        let items = sqlx::query_as!(
            IconRow,
            r#"
            select
                id,
                name,
                path
            from icons
            where name = ?1
        "#,
            name
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(items)
    }

    pub async fn all(&self) -> Result<Vec<IconRow>, DataError> {
        let items = sqlx::query_as!(
            IconRow,
            r#"
            select
                id,
                name,
                path
            from icons
        "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(items)
    }
}
