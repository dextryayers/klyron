use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use sqlx::{Column, Row as SqlxRow};

use crate::error::Result;
use crate::row::{mysql_row_to_row, Row};

pub struct MySqlDb {
    pool: MySqlPool,
}

impl MySqlDb {
    pub async fn connect(url: &str) -> Result<Self> {
        let pool = MySqlPoolOptions::new()
            .max_connections(10)
            .connect(url)
            .await?;
        Ok(Self { pool })
    }

    pub async fn connect_with_pool_size(url: &str, max_connections: u32) -> Result<Self> {
        let pool = MySqlPoolOptions::new()
            .max_connections(max_connections)
            .connect(url)
            .await?;
        Ok(Self { pool })
    }

    pub fn pool(&self) -> &MySqlPool {
        &self.pool
    }

    #[inline]
    pub async fn execute(&self, sql: &str) -> Result<u64> {
        Ok(sqlx::query(sql).execute(&self.pool).await?.rows_affected())
    }

    #[inline]
    pub async fn query(&self, sql: &str) -> Result<Vec<Row>> {
        let rows = sqlx::query(sql).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(mysql_row_to_row).collect())
    }

    #[inline]
    pub async fn query_one(&self, sql: &str) -> Result<Option<Row>> {
        let rows = sqlx::query(sql).fetch_all(&self.pool).await?;
        Ok(rows.first().map(mysql_row_to_row))
    }

    pub async fn ping(&self) -> Result<bool> {
        Ok(sqlx::query("SELECT 1").execute(&self.pool).await.is_ok())
    }

    pub async fn list_tables(&self) -> Result<Vec<String>> {
        let rows = sqlx::query("SHOW TABLES").fetch_all(&self.pool).await?;
        Ok(rows.iter().filter_map(|r| r.try_get::<String, _>(0).ok()).collect())
    }

    pub async fn describe_table(&self, table: &str) -> Result<Vec<serde_json::Value>> {
        let sql = format!("DESCRIBE `{table}`");
        let rows = sqlx::query(&sql).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(|r| {
            let mut map = serde_json::Map::new();
            for (i, col) in r.columns().iter().enumerate() {
                let name = col.name();
                if let Ok(val) = r.try_get::<String, _>(i) {
                    map.insert(name.to_string(), serde_json::Value::String(val));
                }
            }
            serde_json::Value::Object(map)
        }).collect())
    }

    pub async fn execute_with_params(&self, sql: &str, params: &[&str]) -> Result<u64> {
        let mut q = sqlx::query(sql);
        for p in params {
            q = q.bind(p);
        }
        Ok(q.execute(&self.pool).await?.rows_affected())
    }
}
