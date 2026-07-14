use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use sqlx::Row;
use sqlx::Column;
use sqlx::TypeInfo;

pub struct MySqlDb {
    pool: MySqlPool,
}

impl MySqlDb {
    pub async fn connect(host: &str, port: u16, user: &str, password: &str, dbname: &str) -> anyhow::Result<Self> {
        let url = format!("mysql://{user}:{password}@{host}:{port}/{dbname}");
        let pool = MySqlPoolOptions::new()
            .max_connections(5)
            .connect(&url)
            .await?;
        Ok(Self { pool })
    }

    pub async fn connect_url(url: &str) -> anyhow::Result<Self> {
        let pool = MySqlPoolOptions::new()
            .max_connections(5)
            .connect(url)
            .await?;
        Ok(Self { pool })
    }

    pub async fn query(&self, sql: &str) -> anyhow::Result<Vec<serde_json::Value>> {
        let rows = sqlx::query(sql).fetch_all(&self.pool).await?;
        let results: Vec<serde_json::Value> = rows.iter().map(|row| {
            let mut map = serde_json::Map::new();
            for (i, column) in row.columns().iter().enumerate() {
                let name = column.name();
                let type_name = column.type_info().name();
                let val: Option<serde_json::Value> = match type_name {
                    "TINYINT" | "SMALLINT" | "INT" | "BIGINT" | "YEAR" => {
                        row.try_get::<i64, _>(i).ok().map(serde_json::Value::from)
                    }
                    "FLOAT" | "DOUBLE" | "DECIMAL" => {
                        row.try_get::<f64, _>(i).ok().map(serde_json::Value::from)
                    }
                    "VARCHAR" | "CHAR" | "TEXT" | "ENUM" | "DATE" | "DATETIME" | "TIMESTAMP" => {
                        row.try_get::<String, _>(i).ok().map(serde_json::Value::from)
                    }
                    "JSON" => {
                        row.try_get::<serde_json::Value, _>(i).ok()
                    }
                    _ => None,
                };
                if let Some(v) = val {
                    map.insert(name.to_string(), v);
                }
            }
            serde_json::Value::Object(map)
        }).collect();
        Ok(results)
    }

    pub async fn execute(&self, sql: &str) -> anyhow::Result<u64> {
        Ok(sqlx::query(sql).execute(&self.pool).await?.rows_affected())
    }

    pub async fn ping(&self) -> anyhow::Result<bool> {
        Ok(sqlx::query("SELECT 1").execute(&self.pool).await.is_ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_connect_fail() {
        let result = MySqlDb::connect("localhost", 3306, "test", "test", "test").await;
        assert!(result.is_err());
    }
}
