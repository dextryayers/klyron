use std::path::Path;
use std::sync::Mutex;

use rusqlite::{Connection, OpenFlags};

pub struct SqliteDb {
    conn: Mutex<Connection>,
}

impl SqliteDb {
    pub fn open(path: &Path) -> anyhow::Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    pub fn open_in_memory() -> anyhow::Result<Self> {
        let conn = Connection::open_in_memory()?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    pub fn open_with_flags(path: &Path, flags: OpenFlags) -> anyhow::Result<Self> {
        let conn = Connection::open_with_flags(path, flags)?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    pub fn execute(&self, sql: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(sql)?;
        Ok(())
    }

    pub fn query(&self, sql: &str) -> anyhow::Result<Vec<Vec<serde_json::Value>>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(sql)?;
        let col_count = stmt.column_count();
        let rows = stmt.query_map([], |row| {
            let mut values = Vec::new();
            for i in 0..col_count {
                let val = match row.get_ref(i)? {
                    rusqlite::types::ValueRef::Null => serde_json::Value::Null,
                    rusqlite::types::ValueRef::Integer(v) => serde_json::json!(v),
                    rusqlite::types::ValueRef::Real(v) => serde_json::json!(v),
                    rusqlite::types::ValueRef::Text(v) => serde_json::json!(String::from_utf8_lossy(v).to_string()),
                    rusqlite::types::ValueRef::Blob(v) => serde_json::json!(base64_encode(v)),
                };
                values.push(val);
            }
            Ok(values)
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn query_rows(&self, sql: &str) -> anyhow::Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(sql)?;
        let col_count = stmt.column_count();
        let col_names: Vec<String> = (0..col_count).map(|i| stmt.column_name(i).unwrap_or("?").to_string()).collect();

        let rows = stmt.query_map([], |row| {
            let mut map = serde_json::Map::new();
            for i in 0..col_count {
                let val = match row.get_ref(i)? {
                    rusqlite::types::ValueRef::Null => serde_json::Value::Null,
                    rusqlite::types::ValueRef::Integer(v) => serde_json::json!(v),
                    rusqlite::types::ValueRef::Real(v) => serde_json::json!(v),
                    rusqlite::types::ValueRef::Text(v) => serde_json::json!(String::from_utf8_lossy(v).to_string()),
                    rusqlite::types::ValueRef::Blob(v) => serde_json::json!(base64_encode(v)),
                };
                map.insert(col_names[i].clone(), val);
            }
            Ok(serde_json::Value::Object(map))
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn exec(&self, sql: &str) -> anyhow::Result<u64> {
        let conn = self.conn.lock().unwrap();
        let count = conn.execute(sql, [])?;
        Ok(count as u64)
    }

    pub fn transaction<F>(&self, f: F) -> anyhow::Result<()>
    where
        F: FnOnce(&Connection) -> anyhow::Result<()>,
    {
        let conn = self.conn.lock().unwrap();
        f(&conn)?;
        Ok(())
    }

    pub fn tables(&self) -> anyhow::Result<Vec<String>> {
        let rows = self.query("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")?;
        Ok(rows.into_iter().filter_map(|r| r.first().and_then(|v| v.as_str().map(String::from))).collect())
    }
}

fn base64_encode(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sqlite_in_memory() {
        let db = SqliteDb::open_in_memory().unwrap();
        db.execute("CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)").unwrap();
        db.exec("INSERT INTO test (name) VALUES ('hello')").unwrap();
        let rows = db.query_rows("SELECT * FROM test").unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["name"], serde_json::json!("hello"));
    }
}
