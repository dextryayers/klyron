pub mod migrate;
pub mod pool;

use std::path::Path;
use std::sync::{Arc, Mutex};

use rusqlite::{Connection, params_from_iter};
use thiserror::Error;

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

#[derive(Debug, Clone)]
pub struct Row {
    values: Vec<rusqlite::types::Value>,
}

impl Row {
    #[inline]
    pub fn get(&self, index: usize) -> Option<&rusqlite::types::Value> {
        self.values.get(index)
    }

    #[inline]
    pub fn get_str(&self, index: usize) -> Option<&str> {
        match self.values.get(index)? {
            rusqlite::types::Value::Text(s) => Some(s),
            _ => None,
        }
    }

    #[inline]
    pub fn get_int(&self, index: usize) -> Option<i64> {
        match self.values.get(index)? {
            rusqlite::types::Value::Integer(i) => Some(*i),
            _ => None,
        }
    }

    #[inline]
    pub fn get_float(&self, index: usize) -> Option<f64> {
        match self.values.get(index)? {
            rusqlite::types::Value::Real(f) => Some(*f),
            _ => None,
        }
    }

    #[inline]
    pub fn is_null(&self, index: usize) -> bool {
        matches!(self.values.get(index), Some(rusqlite::types::Value::Null) | None)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

pub struct SqliteDb {
    conn: Mutex<Connection>,
}

impl SqliteDb {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path.as_ref())?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        conn.set_prepared_statement_cache_capacity(128);
        Ok(Self { conn: Mutex::new(conn) })
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        conn.set_prepared_statement_cache_capacity(128);
        Ok(Self { conn: Mutex::new(conn) })
    }

    pub fn enable_wal(&self) -> Result<()> {
        let conn = self.conn.lock().map_err(|_| SqliteError::MutexPoisoned)?;
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        Ok(())
    }

    #[inline]
    pub fn execute(&self, sql: &str, params: &[rusqlite::types::Value]) -> Result<usize> {
        let conn = self.conn.lock().map_err(|_| SqliteError::MutexPoisoned)?;
        Ok(conn.execute(sql, params_from_iter(params))?)
    }

    #[inline]
    pub fn execute_batch(&self, sql: &str) -> Result<()> {
        let conn = self.conn.lock().map_err(|_| SqliteError::MutexPoisoned)?;
        Ok(conn.execute_batch(sql)?)
    }

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

    #[inline]
    pub fn query_one(&self, sql: &str, params: &[rusqlite::types::Value]) -> Result<Option<Row>> {
        let mut rows = self.query(sql, params)?;
        Ok(if rows.is_empty() { None } else { Some(rows.swap_remove(0)) })
    }

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

    pub fn backup_to<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let conn = self.conn.lock().map_err(|_| SqliteError::MutexPoisoned)?;
        let dst = Connection::open(path.as_ref())?;
        let mut dst = dst;
        let backup = rusqlite::backup::Backup::new(&conn, &mut dst)?;
        backup.run_to_completion(100, std::time::Duration::from_millis(0), None)?;
        Ok(())
    }

    pub fn tables(&self) -> Result<Vec<String>> {
        let rows = self.query(
            "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name",
            &[],
        )?;
        Ok(rows.iter().filter_map(|r| r.get_str(0).map(String::from)).collect())
    }

    pub fn enable_fts(&self, table_name: &str, columns: &[&str]) -> Result<()> {
        let cols = columns.join(", ");
        let sql = format!(
            "CREATE VIRTUAL TABLE IF NOT EXISTS {}_fts USING fts5({}, content={})",
            table_name, cols, table_name
        );
        self.execute_batch(&sql)?;

        let trigger_insert = format!(
            "CREATE TRIGGER IF NOT EXISTS {}_fts_insert AFTER INSERT ON {} BEGIN
                INSERT INTO {}_fts(rowid, {}) VALUES (new.rowid, {});
            END",
            table_name, table_name, table_name, cols,
            columns.iter().map(|c| format!("new.{}", c)).collect::<Vec<_>>().join(", ")
        );
        self.execute_batch(&trigger_insert)?;

        let trigger_delete = format!(
            "CREATE TRIGGER IF NOT EXISTS {}_fts_delete AFTER DELETE ON {} BEGIN
                INSERT INTO {}_fts({}_fts, rowid, {}) VALUES ('delete', old.rowid, {});
            END",
            table_name, table_name, table_name, table_name, cols,
            columns.iter().map(|c| format!("old.{}", c)).collect::<Vec<_>>().join(", ")
        );
        self.execute_batch(&trigger_delete)?;

        Ok(())
    }

    pub fn fts_search(&self, fts_table: &str, query: &str) -> Result<Vec<Row>> {
        let sql = format!("SELECT * FROM {} WHERE {} MATCH ?1", fts_table, fts_table);
        self.query(&sql, &[rusqlite::types::Value::Text(query.to_string())])
    }

    pub async fn execute_async(
        self: Arc<Self>,
        sql: String,
        params: Vec<rusqlite::types::Value>,
    ) -> Result<usize> {
        tokio::task::spawn_blocking(move || self.execute(&sql, &params)).await?
    }

    pub async fn query_async(
        self: Arc<Self>,
        sql: String,
        params: Vec<rusqlite::types::Value>,
    ) -> Result<Vec<Row>> {
        tokio::task::spawn_blocking(move || self.query(&sql, &params)).await?
    }

    pub async fn query_one_async(
        self: Arc<Self>,
        sql: String,
        params: Vec<rusqlite::types::Value>,
    ) -> Result<Option<Row>> {
        tokio::task::spawn_blocking(move || self.query_one(&sql, &params)).await?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_in_memory_and_create_table() {
        let db = SqliteDb::open_in_memory().unwrap();
        db.execute_batch("CREATE TABLE t (id INTEGER PRIMARY KEY, name TEXT NOT NULL)").unwrap();
        let tables = db.tables().unwrap();
        assert!(tables.contains(&"t".to_string()));
    }

    #[test]
    fn test_insert_and_query_row() {
        let db = SqliteDb::open_in_memory().unwrap();
        db.execute_batch("CREATE TABLE t (id INTEGER PRIMARY KEY, name TEXT)").unwrap();
        db.execute("INSERT INTO t (name) VALUES (?1)", &[rusqlite::types::Value::Text("alice".into())]).unwrap();
        let rows = db.query("SELECT id, name FROM t", &[]).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].get_int(0), Some(1));
        assert_eq!(rows[0].get_str(1), Some("alice"));
    }

    #[test]
    fn test_transaction_commit() {
        let db = SqliteDb::open_in_memory().unwrap();
        db.execute_batch("CREATE TABLE t (id INTEGER PRIMARY KEY, name TEXT)").unwrap();
        db.transaction(|conn| {
            conn.execute("INSERT INTO t (name) VALUES (?1)", ["txn_msg"])?;
            Ok(())
        }).unwrap();
        let rows = db.query("SELECT name FROM t", &[]).unwrap();
        assert_eq!(rows[0].get_str(0), Some("txn_msg"));
    }

    #[test]
    fn test_fts() {
        let db = SqliteDb::open_in_memory().unwrap();
        db.execute_batch("CREATE TABLE articles (id INTEGER PRIMARY KEY, title TEXT, body TEXT)").unwrap();
        db.enable_fts("articles", &["title", "body"]).unwrap();
        db.execute("INSERT INTO articles (title, body) VALUES (?1, ?2)",
            &[rusqlite::types::Value::Text("hello world".into()),
              rusqlite::types::Value::Text("this is a test".into())]).unwrap();
        let results = db.fts_search("articles_fts", "world").unwrap();
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_async_query() {
        let db = Arc::new(SqliteDb::open_in_memory().unwrap());
        db.execute_batch("CREATE TABLE t (id INTEGER PRIMARY KEY, val TEXT)").unwrap();
        db.execute("INSERT INTO t (val) VALUES (?1)", &[rusqlite::types::Value::Text("async".into())]).unwrap();
        let rows = db.clone().query_async("SELECT val FROM t".into(), vec![]).await.unwrap();
        assert_eq!(rows[0].get_str(0), Some("async"));
    }
}
