//! Thread-safe SQLite database binding with WAL mode, prepared statement caching,
//! backup, transaction support, and async wrappers.

use std::path::Path;
use std::sync::{Arc, Mutex};

use rusqlite::{Connection, params_from_iter};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Error, Debug)]
pub enum SqliteError {
    #[error("rusqlite error: {0}")]
    Rusqlite(#[from] rusqlite::Error),

    #[error("mutex poisoned")]
    MutexPoisoned,

    #[error("tokio join error: {0}")]
    TokioJoin(#[from] tokio::task::JoinError),
}

pub type Result<T> = std::result::Result<T, SqliteError>;

// ---------------------------------------------------------------------------
// Row
// ---------------------------------------------------------------------------

/// A single row of query results backed by `rusqlite::types::Value`.
#[derive(Debug, Clone)]
pub struct Row {
    values: Vec<rusqlite::types::Value>,
}

impl Row {
    /// Returns the raw [`rusqlite::types::Value`] at the given column index.
    #[inline]
    pub fn get(&self, index: usize) -> Option<&rusqlite::types::Value> {
        self.values.get(index)
    }

    /// Returns the value as a string slice if the column holds `Text`.
    #[inline]
    pub fn get_str(&self, index: usize) -> Option<&str> {
        match self.values.get(index)? {
            rusqlite::types::Value::Text(s) => Some(s),
            _ => None,
        }
    }

    /// Returns the value as an `i64` if the column holds `Integer`.
    #[inline]
    pub fn get_int(&self, index: usize) -> Option<i64> {
        match self.values.get(index)? {
            rusqlite::types::Value::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Returns the value as an `f64` if the column holds `Real`.
    #[inline]
    pub fn get_float(&self, index: usize) -> Option<f64> {
        match self.values.get(index)? {
            rusqlite::types::Value::Real(f) => Some(*f),
            _ => None,
        }
    }

    /// Returns `true` if the column is NULL.
    #[inline]
    pub fn is_null(&self, index: usize) -> bool {
        matches!(self.values.get(index), Some(rusqlite::types::Value::Null) | None)
    }

    /// Number of columns in this row.
    #[inline]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns `true` if the row has no columns.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

// ---------------------------------------------------------------------------
// SqliteDb
// ---------------------------------------------------------------------------

/// A thread-safe SQLite database handle.
///
/// All synchronous methods serialise access through an internal `Mutex`.
/// WAL journal mode and foreign keys are enabled by default.
/// A prepared-statement cache (capacity 128) is active on the connection.
pub struct SqliteDb {
    conn: Mutex<Connection>,
}

impl SqliteDb {
    /// Opens or creates a SQLite database at `path`.
    ///
    /// Enables WAL journal mode, foreign keys, and sets the prepared
    /// statement cache capacity to 128 entries.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path.as_ref())?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        conn.set_prepared_statement_cache_capacity(128);
        Ok(Self { conn: Mutex::new(conn) })
    }

    /// Opens an in-memory SQLite database.
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        conn.set_prepared_statement_cache_capacity(128);
        Ok(Self { conn: Mutex::new(conn) })
    }

    /// Activates WAL journal mode on the current connection.
    pub fn enable_wal(&self) -> Result<()> {
        let conn = self.conn.lock().map_err(|_| SqliteError::MutexPoisoned)?;
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        Ok(())
    }

    /// Executes a parameterised statement and returns the number of rows
    /// inserted / updated / deleted.
    #[inline]
    pub fn execute(&self, sql: &str, params: &[rusqlite::types::Value]) -> Result<usize> {
        let conn = self.conn.lock().map_err(|_| SqliteError::MutexPoisoned)?;
        Ok(conn.execute(sql, params_from_iter(params))?)
    }

    /// Executes one or more statements with no parameter binding.
    #[inline]
    pub fn execute_batch(&self, sql: &str) -> Result<()> {
        let conn = self.conn.lock().map_err(|_| SqliteError::MutexPoisoned)?;
        Ok(conn.execute_batch(sql)?)
    }

    /// Queries the database and returns all result rows.
    #[inline]
    pub fn query(&self, sql: &str, params: &[rusqlite::types::Value]) -> Result<Vec<Row>> {
        let conn = self.conn.lock().map_err(|_| SqliteError::MutexPoisoned)?;
        let mut stmt = conn.prepare_cached(sql)?;
        let col_count = stmt.column_count();
        let rows = stmt.query_map(params_from_iter(params), |row| {
            let mut values = Vec::with_capacity(col_count);
            for i in 0..col_count {
                values.push(row.get::<_, rusqlite::types::Value>(i)?);
            }
            Ok(Row { values })
        })?;
        let mut results = Vec::with_capacity(rows.size_hint().0);
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Queries the database and returns the first row, if any.
    #[inline]
    pub fn query_one(&self, sql: &str, params: &[rusqlite::types::Value]) -> Result<Option<Row>> {
        let mut rows = self.query(sql, params)?;
        Ok(if rows.is_empty() { None } else { Some(rows.swap_remove(0)) })
    }

    /// Executes a closure inside a database transaction.
    ///
    /// The closure receives a `&Connection` (which is actually a `Transaction`
    /// that will be committed on success or rolled back on panic / error).
    pub fn transaction<T, F>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Connection) -> Result<T>,
    {
        let conn = self.conn.lock().map_err(|_| SqliteError::MutexPoisoned)?;
        let tx = conn.unchecked_transaction()?;
        let result = f(&tx)?;
        tx.commit()?;
        Ok(result)
    }

    /// Creates a live backup of the current database to another file.
    pub fn backup_to<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let conn = self.conn.lock().map_err(|_| SqliteError::MutexPoisoned)?;
        let dst = Connection::open(path.as_ref())?;
        let mut dst = dst;
        let backup = rusqlite::backup::Backup::new(&conn, &mut dst)?;
        backup.run_to_completion(100, std::time::Duration::from_millis(0), None)?;
        Ok(())
    }

    /// Returns the list of user-defined table names.
    pub fn tables(&self) -> Result<Vec<String>> {
        let rows = self.query(
            "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name",
            &[],
        )?;
        Ok(rows.iter().filter_map(|r| r.get_str(0).map(String::from)).collect())
    }

    // ---- Async wrappers ---------------------------------------------------

    /// Asynchronously executes a statement via `spawn_blocking`.
    pub async fn execute_async(
        self: Arc<Self>,
        sql: String,
        params: Vec<rusqlite::types::Value>,
    ) -> Result<usize> {
        tokio::task::spawn_blocking(move || self.execute(&sql, &params)).await?
    }

    /// Asynchronously queries via `spawn_blocking`.
    pub async fn query_async(
        self: Arc<Self>,
        sql: String,
        params: Vec<rusqlite::types::Value>,
    ) -> Result<Vec<Row>> {
        tokio::task::spawn_blocking(move || self.query(&sql, &params)).await?
    }

    /// Asynchronously queries a single row via `spawn_blocking`.
    pub async fn query_one_async(
        self: Arc<Self>,
        sql: String,
        params: Vec<rusqlite::types::Value>,
    ) -> Result<Option<Row>> {
        tokio::task::spawn_blocking(move || self.query_one(&sql, &params)).await?
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_in_memory_and_create_table() {
        let db = SqliteDb::open_in_memory().unwrap();
        db.execute_batch("CREATE TABLE t (id INTEGER PRIMARY KEY, name TEXT NOT NULL)")
            .unwrap();
        let tables = db.tables().unwrap();
        assert!(tables.contains(&"t".to_string()));
    }

    #[test]
    fn test_insert_and_query_row() {
        let db = SqliteDb::open_in_memory().unwrap();
        db.execute_batch("CREATE TABLE t (id INTEGER PRIMARY KEY, name TEXT)").unwrap();
        db.execute(
            "INSERT INTO t (name) VALUES (?1)",
            &[rusqlite::types::Value::Text("alice".into())],
        )
        .unwrap();
        let rows = db.query("SELECT id, name FROM t", &[]).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].get_int(0), Some(1));
        assert_eq!(rows[0].get_str(1), Some("alice"));
    }

    #[test]
    fn test_query_one_returns_none_when_empty() {
        let db = SqliteDb::open_in_memory().unwrap();
        db.execute_batch("CREATE TABLE t (id INTEGER PRIMARY KEY, name TEXT)").unwrap();
        let row = db.query_one("SELECT * FROM t WHERE id = 99", &[]).unwrap();
        assert!(row.is_none());
    }

    #[test]
    fn test_transaction_commit() {
        let db = SqliteDb::open_in_memory().unwrap();
        db.execute_batch("CREATE TABLE t (id INTEGER PRIMARY KEY, name TEXT)").unwrap();
        db.transaction(|conn| {
            conn.execute("INSERT INTO t (name) VALUES (?1)", ["txn_msg"])?;
            Ok(())
        })
        .unwrap();
        let rows = db.query("SELECT name FROM t", &[]).unwrap();
        assert_eq!(rows[0].get_str(0), Some("txn_msg"));
    }

    #[test]
    fn test_backup_to_creates_valid_copy() {
        let src = SqliteDb::open_in_memory().unwrap();
        src.execute_batch("CREATE TABLE t (v INTEGER)").unwrap();
        src.execute("INSERT INTO t VALUES (42)", &[]).unwrap();

        let tmp = std::env::temp_dir().join(format!("klyron_backup_{}.db", std::process::id()));
        src.backup_to(&tmp).unwrap();

        let dst = SqliteDb::open(&tmp).unwrap();
        let rows = dst.query("SELECT v FROM t", &[]).unwrap();
        assert_eq!(rows[0].get_int(0), Some(42));
        let _ = std::fs::remove_file(&tmp);
    }

    #[tokio::test]
    async fn test_async_query() {
        let db = Arc::new(SqliteDb::open_in_memory().unwrap());
        db.execute_batch("CREATE TABLE t (id INTEGER PRIMARY KEY, val TEXT)").unwrap();
        db.execute("INSERT INTO t (val) VALUES (?1)", &[rusqlite::types::Value::Text("async".into())])
            .unwrap();
        let rows = db.clone().query_async("SELECT val FROM t".into(), vec![]).await.unwrap();
        assert_eq!(rows[0].get_str(0), Some("async"));
    }

    #[test]
    fn test_enable_wal_on_file_db() {
        let tmp = std::env::temp_dir().join(format!("klyron_wal_test_{}.db", std::process::id()));
        let db = SqliteDb::open(&tmp).unwrap();
        // Already in WAL mode from open(). Ensure it doesn't error.
        db.enable_wal().unwrap();
        let rows = db.query("PRAGMA journal_mode", &[]).unwrap();
        assert_eq!(rows[0].get_str(0), Some("wal"));
        let _ = std::fs::remove_file(&tmp);
        let _ = std::fs::remove_file(tmp.with_extension("db-wal"));
        let _ = std::fs::remove_file(tmp.with_extension("db-shm"));
    }
}
