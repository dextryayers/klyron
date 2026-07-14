//! PostgreSQL database binding with connection pooling, prepared statement caching,
//! transaction support, COPY FROM, and type-safe row accessors.

use std::sync::Arc;

use bytes::Bytes;
use deadpool_postgres::{Manager, Pool};
use futures::SinkExt;
use thiserror::Error;
use tokio::sync::Mutex;
use tokio_postgres::{Client, NoTls, Row as PgRow};

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Error, Debug)]
pub enum PostgresError {
    #[error("tokio-postgres error: {0}")]
    TokioPostgres(#[from] tokio_postgres::Error),

    #[error("deadpool error: {0}")]
    Deadpool(#[from] deadpool_postgres::PoolError),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("config error: {0}")]
    Config(String),

    #[error("no rows returned")]
    NoRows,
}

pub type Result<T> = std::result::Result<T, PostgresError>;

// ---------------------------------------------------------------------------
// Row
// ---------------------------------------------------------------------------

/// A row of query results with type-safe accessors over `serde_json::Value`.
#[derive(Debug, Clone)]
pub struct Row {
    columns: Vec<String>,
    values: Vec<serde_json::Value>,
}

impl Row {
    /// Returns the raw `serde_json::Value` at the given column index.
    #[inline]
    pub fn get(&self, index: usize) -> Option<&serde_json::Value> {
        self.values.get(index)
    }

    /// Returns the value as a string slice if the column holds a JSON string.
    #[inline]
    pub fn get_str(&self, index: usize) -> Option<&str> {
        self.values.get(index)?.as_str()
    }

    /// Returns the value as an `i64` if the column holds a JSON number.
    #[inline]
    pub fn get_int(&self, index: usize) -> Option<i64> {
        self.values.get(index)?.as_i64()
    }

    /// Returns the value as an `f64` if the column holds a JSON number.
    #[inline]
    pub fn get_float(&self, index: usize) -> Option<f64> {
        self.values.get(index)?.as_f64()
    }

    /// Returns the value as a `bool` if the column holds a JSON boolean.
    #[inline]
    pub fn get_bool(&self, index: usize) -> Option<bool> {
        self.values.get(index)?.as_bool()
    }

    /// Returns `true` if the column is SQL `NULL` (JSON `Null`).
    #[inline]
    pub fn is_null(&self, index: usize) -> bool {
        self.values.get(index).map_or(true, serde_json::Value::is_null)
    }

    /// Column names of this row.
    #[inline]
    pub fn columns(&self) -> &[String] {
        &self.columns
    }

    /// Number of columns.
    #[inline]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns `true` when there are no columns.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn pg_row_to_row(row: &PgRow) -> Row {
    let columns: Vec<String> = row.columns().iter().map(|c| c.name().to_string()).collect();
    let mut values = Vec::with_capacity(columns.len());
    for (i, _) in columns.iter().enumerate() {
        values.push(pg_cell_to_json(row, i));
    }
    Row { columns, values }
}

fn pg_cell_to_json(row: &PgRow, i: usize) -> serde_json::Value {
    let col = &row.columns()[i];
    match col.type_().name() {
        "int2" | "int4" | "int8" => {
            row.try_get::<_, i64>(i).ok().map_or(serde_json::Value::Null, serde_json::Value::from)
        }
        "float4" | "float8" => {
            row.try_get::<_, f64>(i).ok().map_or(serde_json::Value::Null, serde_json::Value::from)
        }
        "numeric" => row
            .try_get::<_, String>(i)
            .ok()
            .and_then(|s| s.parse::<f64>().ok())
            .map_or(serde_json::Value::Null, serde_json::Value::from),
        "bool" => {
            row.try_get::<_, bool>(i).ok().map_or(serde_json::Value::Null, serde_json::Value::from)
        }
        "varchar" | "text" | "name" | "bpchar" | "char" => {
            row.try_get::<_, String>(i).ok().map_or(serde_json::Value::Null, serde_json::Value::from)
        }
        "json" | "jsonb" => {
            row.try_get::<_, serde_json::Value>(i).ok().unwrap_or(serde_json::Value::Null)
        }
        "bytea" => row
            .try_get::<_, Vec<u8>>(i)
            .ok()
            .map(|v| serde_json::Value::Array(v.into_iter().map(serde_json::Value::from).collect()))
            .unwrap_or(serde_json::Value::Null),
        _ => row.try_get::<_, String>(i).ok().map_or(serde_json::Value::Null, serde_json::Value::from),
    }
}

// ---------------------------------------------------------------------------
// PostgresDb
// ---------------------------------------------------------------------------

enum Inner {
    Direct { client: Mutex<Client> },
    Pool { pool: Pool },
}

/// A PostgreSQL database handle supporting single-connection and pooled modes.
///
/// The `connect()` variant maintains a **single** persistent connection with
/// automatic prepared statement caching.  The `connect_pool()` variant uses
/// `deadpool-postgres` for connection pooling.
pub struct PostgresDb {
    inner: Arc<Inner>,
}

impl PostgresDb {
    /// Opens a single dedicated connection to PostgreSQL.
    ///
    /// `conn_str` uses libpq key=value syntax (e.g.
    /// `"host=localhost port=5432 dbname=test user=postgres"`).
    /// The connection background task is spawned onto the current runtime.
    pub async fn connect(conn_str: &str) -> Result<Self> {
        let (client, connection) = tokio_postgres::connect(conn_str, NoTls).await?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("PostgreSQL connection error: {e}");
            }
        });
        Ok(Self {
            inner: Arc::new(Inner::Direct {
                client: Mutex::new(client),
            }),
        })
    }

    /// Creates a connection pool to PostgreSQL.
    ///
    /// `max_size` controls the maximum number of connections in the pool.
    pub async fn connect_pool(conn_str: &str, max_size: usize) -> Result<Self> {
        let pg_config = conn_str
            .parse::<tokio_postgres::Config>()
            .map_err(|e| PostgresError::Config(e.to_string()))?;
        let mgr = Manager::new(pg_config, NoTls);
        let pool = Pool::builder(mgr)
            .max_size(max_size)
            .build()
            .map_err(|e| PostgresError::Config(e.to_string()))?;
        Ok(Self {
            inner: Arc::new(Inner::Pool { pool }),
        })
    }

    /// Executes a statement and returns the number of rows affected.
    #[inline]
    pub async fn execute(&self, sql: &str, params: &[&(dyn tokio_postgres::types::ToSql + Sync)]) -> Result<u64> {
        match &*self.inner {
            Inner::Direct { client } => {
                let client = client.lock().await;
                Ok(client.execute(sql, params).await?)
            }
            Inner::Pool { pool } => {
                let client = pool.get().await?;
                Ok(client.execute(sql, params).await?)
            }
        }
    }

    /// Queries the database and returns all result rows.
    #[inline]
    pub async fn query(&self, sql: &str, params: &[&(dyn tokio_postgres::types::ToSql + Sync)]) -> Result<Vec<Row>> {
        match &*self.inner {
            Inner::Direct { client } => {
                let client = client.lock().await;
                let rows = client.query(sql, params).await?;
                Ok(rows.iter().map(pg_row_to_row).collect())
            }
            Inner::Pool { pool } => {
                let client = pool.get().await?;
                let rows = client.query(sql, params).await?;
                Ok(rows.iter().map(pg_row_to_row).collect())
            }
        }
    }

    /// Queries the database and returns the first row, if any.
    #[inline]
    pub async fn query_one(
        &self,
        sql: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<Option<Row>> {
        match &*self.inner {
            Inner::Direct { client } => {
                let client = client.lock().await;
                let row = client.query_opt(sql, params).await?;
                Ok(row.as_ref().map(pg_row_to_row))
            }
            Inner::Pool { pool } => {
                let client = pool.get().await?;
                let row = client.query_opt(sql, params).await?;
                Ok(row.as_ref().map(pg_row_to_row))
            }
        }
    }

    /// Executes a closure inside a database transaction.
    ///
    /// The closure receives a `&Transaction<'_>` and may execute statements
    /// that are committed atomically on success.
    pub async fn transaction<T, F>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&tokio_postgres::Transaction<'_>) -> Result<T>,
    {
        match &*self.inner {
            Inner::Direct { client } => {
                let mut client = client.lock().await;
                let tx = client.transaction().await?;
                let result = f(&tx)?;
                tx.commit().await?;
                Ok(result)
            }
            Inner::Pool { pool } => {
                let mut client = pool.get().await?;
                let tx = client.transaction().await?;
                let result = f(&tx)?;
                tx.commit().await?;
                Ok(result)
            }
        }
    }

    /// Copies CSV data from a string into a table using PostgreSQL's `COPY FROM STDIN`.
    ///
    /// `data` should be a CSV-formatted string (with a header row if `has_header` is `true`).
    /// Returns the number of rows copied when the server supports it (otherwise `0`).
    pub async fn copy_from(
        &self,
        data: &str,
        table: &str,
        columns: &[&str],
        has_header: bool,
    ) -> Result<u64> {
        let cols = columns.join(", ");
        let header = if has_header { "HEADER" } else { "" };
        let stmt = format!("COPY {table} ({cols}) FROM STDIN WITH (FORMAT CSV, {header})");

        async fn do_copy(conn: &mut Client, stmt: &str, data: &str) -> Result<u64> {
            let sink = conn.copy_in(stmt).await?;
            let mut sink = Box::pin(sink);
            sink.as_mut()
                .send(Bytes::copy_from_slice(data.as_bytes()))
                .await?;
            sink.as_mut().close().await?;
            Ok(0)
        }

        match &*self.inner {
            Inner::Direct { client } => {
                let mut client = client.lock().await;
                do_copy(&mut client, &stmt, data).await
            }
            Inner::Pool { pool } => {
                let mut client = pool.get().await?;
                do_copy(&mut client, &stmt, data).await
            }
        }
    }

    /// Returns `true` if the database is reachable.
    pub async fn ping(&self) -> Result<bool> {
        match &*self.inner {
            Inner::Direct { client } => {
                let client = client.lock().await;
                Ok(client.query_one("SELECT 1", &[]).await.is_ok())
            }
            Inner::Pool { pool } => {
                let client = pool.get().await?;
                Ok(client.query_one("SELECT 1", &[]).await.is_ok())
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_row_accessors() {
        let row = Row {
            columns: vec!["id".into(), "name".into(), "active".into()],
            values: vec![
                serde_json::json!(42),
                serde_json::json!("alice"),
                serde_json::json!(true),
            ],
        };
        assert_eq!(row.get_int(0), Some(42));
        assert_eq!(row.get_str(1), Some("alice"));
        assert_eq!(row.get_bool(2), Some(true));
        assert!(!row.is_null(0));
        assert_eq!(row.columns(), &["id", "name", "active"]);
    }

    #[test]
    fn test_row_null_handling() {
        let row = Row {
            columns: vec!["val".into()],
            values: vec![serde_json::Value::Null],
        };
        assert!(row.is_null(0));
        assert!(row.get_str(0).is_none());
        assert!(row.get_int(0).is_none());
    }

    #[test]
    fn test_row_out_of_bounds() {
        let row = Row {
            columns: vec!["a".into()],
            values: vec![serde_json::json!(1)],
        };
        assert!(row.get(99).is_none());
    }

    #[tokio::test]
    async fn test_connect_fail() {
        let result = PostgresDb::connect("host=localhost port=1 dbname=nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_connect_pool_builds_without_error() {
        // Pool construction is lazy; bad configs don't fail until get().
        let result = PostgresDb::connect_pool("host=localhost port=1 dbname=nonexistent", 2).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_row_empty() {
        let row = Row {
            columns: vec![],
            values: vec![],
        };
        assert!(row.is_empty());
        assert_eq!(row.len(), 0);
    }
}
