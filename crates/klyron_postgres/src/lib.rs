use tokio_postgres::{Client, NoTls, Row};

pub struct PostgresDb {
    client: Client,
}

impl PostgresDb {
    pub async fn connect(host: &str, port: u16, user: &str, password: &str, dbname: &str) -> anyhow::Result<Self> {
        let conn_str = format!("host={host} port={port} user={user} password={password} dbname={dbname}");
        let (client, connection) = tokio_postgres::connect(&conn_str, NoTls).await?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("PostgreSQL connection error: {e}");
            }
        });
        Ok(Self { client })
    }

    pub async fn connect_url(url: &str) -> anyhow::Result<Self> {
        let (client, connection) = tokio_postgres::connect(url, NoTls).await?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("PostgreSQL connection error: {e}");
            }
        });
        Ok(Self { client })
    }

    pub async fn query(&self, sql: &str) -> anyhow::Result<Vec<serde_json::Value>> {
        let rows = self.client.query(sql, &[]).await?;
        Ok(rows_to_json(&rows))
    }

    pub async fn query_params(&self, sql: &str, params: &[&(dyn tokio_postgres::types::ToSql + Sync)]) -> anyhow::Result<Vec<serde_json::Value>> {
        let rows = self.client.query(sql, params).await?;
        Ok(rows_to_json(&rows))
    }

    pub async fn execute(&self, sql: &str) -> anyhow::Result<u64> {
        Ok(self.client.execute(sql, &[]).await?)
    }

    pub async fn execute_params(&self, sql: &str, params: &[&(dyn tokio_postgres::types::ToSql + Sync)]) -> anyhow::Result<u64> {
        Ok(self.client.execute(sql, params).await?)
    }

    pub async fn ping(&self) -> anyhow::Result<bool> {
        Ok(self.client.query_one("SELECT 1", &[]).await.is_ok())
    }
}

fn rows_to_json(rows: &[Row]) -> Vec<serde_json::Value> {
    rows.iter().map(|row| {
        let mut map = serde_json::Map::new();
        for (i, column) in row.columns().iter().enumerate() {
            let col_name = column.name();
            let val: Option<serde_json::Value> = match column.type_().name() {
                "int2" | "int4" | "int8" => row.try_get::<_, i64>(i).ok().map(serde_json::Value::from),
                "float4" | "float8" => row.try_get::<_, f64>(i).ok().map(serde_json::Value::from),
                "bool" => row.try_get::<_, bool>(i).ok().map(serde_json::Value::from),
                "varchar" | "text" | "name" | "bpchar" => row.try_get::<_, String>(i).ok().map(serde_json::Value::from),
                "json" | "jsonb" => row.try_get::<_, String>(i).ok().and_then(|s| serde_json::from_str(&s).ok()),
                _ => None,
            };
            if let Some(v) = val {
                map.insert(col_name.to_string(), v);
            }
        }
        serde_json::Value::Object(map)
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_connect_fail() {
        let result = PostgresDb::connect("localhost", 5432, "test", "test", "test").await;
        assert!(result.is_err());
    }
}
