use deadpool_postgres::{Manager, ManagerConfig, RecyclingMethod, Runtime, Pool as Deadpool};
use thiserror::Error;
use tokio_postgres::{Config, NoTls};

use crate::{PostgresError, Result, Row, pg_row_to_row};

#[derive(Error, Debug)]
pub enum PoolError {
    #[error("pool config error: {0}")]
    Config(String),

    #[error("deadpool error: {0}")]
    Deadpool(#[from] deadpool_postgres::PoolError),

    #[error("tokio-postgres error: {0}")]
    TokioPostgres(#[from] tokio_postgres::Error),

    #[error("pool timeout")]
    Timeout,
}

pub struct ConnectionPool {
    pool: Deadpool,
    max_size: usize,
}

impl ConnectionPool {
    pub fn new(conn_str: &str, max_size: usize) -> Result<Self> {
        let pg_config: Config = conn_str
            .parse()
            .map_err(|e: tokio_postgres::Error| PostgresError::Config(e.to_string()))?;

        let mgr = Manager::from_config(pg_config, NoTls, ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        });

        let pool = Deadpool::builder(mgr)
            .max_size(max_size)
            .runtime(Runtime::Tokio1)
            .build()
            .map_err(|e| PostgresError::Config(e.to_string()))?;

        Ok(Self { pool, max_size })
    }

    pub async fn get(&self) -> std::result::Result<deadpool_postgres::Client, PoolError> {
        self.pool.get().await.map_err(PoolError::Deadpool)
    }

    pub async fn execute(&self, sql: &str, params: &[&(dyn tokio_postgres::types::ToSql + Sync)]) -> Result<u64> {
        let client = self.pool.get().await?;
        Ok(client.execute(sql, params).await?)
    }

    pub async fn query(&self, sql: &str, params: &[&(dyn tokio_postgres::types::ToSql + Sync)]) -> Result<Vec<Row>> {
        let client = self.pool.get().await?;
        let rows = client.query(sql, params).await?;
        Ok(rows.iter().map(pg_row_to_row).collect())
    }

    pub async fn query_one(
        &self,
        sql: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<Option<Row>> {
        let client = self.pool.get().await?;
        let row = client.query_opt(sql, params).await?;
        Ok(row.as_ref().map(pg_row_to_row))
    }

    pub fn max_size(&self) -> usize {
        self.max_size
    }

    pub async fn status(&self) -> deadpool_postgres::Status {
        self.pool.status()
    }

    pub async fn ping(&self) -> Result<bool> {
        let client = self.pool.get().await?;
        Ok(client.query_one("SELECT 1", &[]).await.is_ok())
    }
}

impl Clone for ConnectionPool {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            max_size: self.max_size,
        }
    }
}
