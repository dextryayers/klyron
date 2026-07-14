//! MySQL / MariaDB database binding with connection pooling, schema introspection,
//! a fluent query builder, and type-safe row accessors.

use sqlx::mysql::{MySqlPool, MySqlPoolOptions, MySqlRow};
use sqlx::{Column, Row as SqlxRow, TypeInfo};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Error, Debug)]
pub enum MySqlError {
    #[error("sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("no rows returned")]
    NoRows,
}

pub type Result<T> = std::result::Result<T, MySqlError>;

// ---------------------------------------------------------------------------
// Row
// ---------------------------------------------------------------------------

/// A row of query results backed by `serde_json::Value`.
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

    /// Returns `true` if the column is SQL `NULL`.
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

fn mysql_row_to_row(row: &MySqlRow) -> Row {
    let columns: Vec<String> = row.columns().iter().map(|c| c.name().to_string()).collect();
    let mut values = Vec::with_capacity(columns.len());
    for (i, _) in columns.iter().enumerate() {
        values.push(mysql_cell_to_json(row, i));
    }
    Row { columns, values }
}

fn mysql_cell_to_json(row: &MySqlRow, i: usize) -> serde_json::Value {
    let col = &row.columns()[i];
    let type_name = col.type_info().name();
    match type_name {
        "TINYINT" | "SMALLINT" | "INT" | "BIGINT" | "YEAR" | "MEDIUMINT" | "BIT" => {
            row.try_get::<i64, _>(i).ok().map_or(serde_json::Value::Null, serde_json::Value::from)
        }
        "FLOAT" | "DOUBLE" | "DECIMAL" => {
            row.try_get::<f64, _>(i).ok().map_or(serde_json::Value::Null, serde_json::Value::from)
        }
        "VARCHAR" | "CHAR" | "TEXT" | "TINYTEXT" | "MEDIUMTEXT" | "LONGTEXT" | "ENUM" | "SET"
        | "DATE" | "DATETIME" | "TIMESTAMP" | "TIME" => {
            row.try_get::<String, _>(i).ok().map_or(serde_json::Value::Null, serde_json::Value::from)
        }
        "JSON" => {
            row.try_get::<serde_json::Value, _>(i).ok().unwrap_or(serde_json::Value::Null)
        }
        "BLOB" | "TINYBLOB" | "MEDIUMBLOB" | "LONGBLOB" | "BINARY" | "VARBINARY" => {
            row.try_get::<Vec<u8>, _>(i).ok()
                .map(|v| serde_json::Value::Array(v.into_iter().map(serde_json::Value::from).collect()))
                .unwrap_or(serde_json::Value::Null)
        }
        _ => {
            row.try_get::<String, _>(i).ok().map_or(serde_json::Value::Null, serde_json::Value::from)
        }
    }
}

// ---------------------------------------------------------------------------
// SelectBuilder
// ---------------------------------------------------------------------------

/// A fluent SELECT query builder for MySQL.
#[derive(Debug, Clone)]
pub struct SelectBuilder {
    table: String,
    columns: Vec<String>,
    conditions: Vec<String>,
    order_by: Option<(String, bool)>,
    limit: Option<u64>,
    offset: Option<u64>,
}

impl SelectBuilder {
    /// Creates a new SELECT builder targeting `table`.
    pub fn new(table: &str) -> Self {
        Self {
            table: table.to_string(),
            columns: vec!["*".into()],
            conditions: vec![],
            order_by: None,
            limit: None,
            offset: None,
        }
    }

    /// Sets the selected columns (defaults to `*`).
    pub fn columns(&mut self, cols: &[&str]) -> &mut Self {
        self.columns = cols.iter().map(|c| c.to_string()).collect();
        self
    }

    /// Adds a WHERE condition (AND-ed with existing conditions).
    pub fn where_(&mut self, condition: &str) -> &mut Self {
        self.conditions.push(condition.to_string());
        self
    }

    /// Sets the ORDER BY clause.  `asc` controls the sort direction.
    pub fn order_by(&mut self, col: &str, asc: bool) -> &mut Self {
        self.order_by = Some((col.to_string(), asc));
        self
    }

    /// Sets the LIMIT.
    pub fn limit(&mut self, limit: u64) -> &mut Self {
        self.limit = Some(limit);
        self
    }

    /// Sets the OFFSET.
    pub fn offset(&mut self, offset: u64) -> &mut Self {
        self.offset = Some(offset);
        self
    }

    /// Builds the final SQL query string.
    pub fn build(&self) -> String {
        let cols = self.columns.join(", ");
        let mut sql = format!("SELECT {cols} FROM {}", self.table);
        if !self.conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&self.conditions.join(" AND "));
        }
        if let Some((col, asc)) = &self.order_by {
            let dir = if *asc { "ASC" } else { "DESC" };
            sql.push_str(&format!(" ORDER BY {col} {dir}"));
        }
        if let Some(limit) = self.limit {
            sql.push_str(&format!(" LIMIT {limit}"));
        }
        if let Some(offset) = self.offset {
            sql.push_str(&format!(" OFFSET {offset}"));
        }
        sql
    }
}

// ---------------------------------------------------------------------------
// MySqlDb
// ---------------------------------------------------------------------------

/// A MySQL / MariaDB database handle backed by `sqlx`.
pub struct MySqlDb {
    pool: MySqlPool,
}

impl MySqlDb {
    /// Opens a connection pool to a MySQL server using a connection string.
    ///
    /// The URL format is `mysql://user:password@host:port/database`.
    /// Default pool size is 10 connections.
    pub async fn connect(url: &str) -> Result<Self> {
        let pool = MySqlPoolOptions::new()
            .max_connections(10)
            .connect(url)
            .await?;
        Ok(Self { pool })
    }

    /// Opens a connection pool with a custom maximum size.
    pub async fn connect_with_pool_size(url: &str, max_connections: u32) -> Result<Self> {
        let pool = MySqlPoolOptions::new()
            .max_connections(max_connections)
            .connect(url)
            .await?;
        Ok(Self { pool })
    }

    /// Executes a statement and returns the number of rows affected.
    #[inline]
    pub async fn execute(&self, sql: &str) -> Result<u64> {
        Ok(sqlx::query(sql).execute(&self.pool).await?.rows_affected())
    }

    /// Queries the database and returns all result rows.
    #[inline]
    pub async fn query(&self, sql: &str) -> Result<Vec<Row>> {
        let rows = sqlx::query(sql).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(mysql_row_to_row).collect())
    }

    /// Queries the database and returns the first row, if any.
    #[inline]
    pub async fn query_one(&self, sql: &str) -> Result<Option<Row>> {
        let rows = sqlx::query(sql).fetch_all(&self.pool).await?;
        Ok(rows.first().map(mysql_row_to_row))
    }

    /// Executes a closure inside a database transaction.
    ///
    /// The closure receives a `&mut sqlx::Transaction<'_, MySql>`.
    pub async fn transaction<T, F>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut sqlx::Transaction<'_, sqlx::MySql>) -> Result<T>,
    {
        let mut tx = self.pool.begin().await?;
        let result = f(&mut tx)?;
        tx.commit().await?;
        Ok(result)
    }

    /// Checks whether the database server is reachable.
    pub async fn ping(&self) -> Result<bool> {
        Ok(sqlx::query("SELECT 1").execute(&self.pool).await.is_ok())
    }

    /// Returns the list of table names in the current database.
    pub async fn list_tables(&self) -> Result<Vec<String>> {
        let rows = sqlx::query("SHOW TABLES").fetch_all(&self.pool).await?;
        Ok(rows.iter().filter_map(|r| r.try_get::<String, _>(0).ok()).collect())
    }

    /// Returns the column metadata for `table`.
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
            columns: vec!["id".into(), "label".into()],
            values: vec![serde_json::json!(7), serde_json::json!("hello")],
        };
        assert_eq!(row.get_int(0), Some(7));
        assert_eq!(row.get_str(1), Some("hello"));
        assert_eq!(row.len(), 2);
    }

    #[test]
    fn test_row_null() {
        let row = Row {
            columns: vec!["x".into()],
            values: vec![serde_json::Value::Null],
        };
        assert!(row.is_null(0));
        assert!(row.get_int(0).is_none());
    }

    #[test]
    fn test_select_builder_basic() {
        let q = SelectBuilder::new("users")
            .columns(&["id", "name"])
            .where_("active = 1")
            .order_by("name", true)
            .limit(10)
            .build();
        assert_eq!(q, "SELECT id, name FROM users WHERE active = 1 ORDER BY name ASC LIMIT 10");
    }

    #[test]
    fn test_select_builder_default_star() {
        let q = SelectBuilder::new("items").where_("price > 100").build();
        assert_eq!(q, "SELECT * FROM items WHERE price > 100");
    }

    #[test]
    fn test_select_builder_offset() {
        let q = SelectBuilder::new("logs")
            .limit(50)
            .offset(100)
            .build();
        assert_eq!(q, "SELECT * FROM logs LIMIT 50 OFFSET 100");
    }

    #[test]
    fn test_row_empty() {
        let row = Row {
            columns: vec![],
            values: vec![],
        };
        assert!(row.is_empty());
    }

    #[tokio::test]
    async fn test_connect_fail() {
        let result = MySqlDb::connect("mysql://invalid:3306/nope").await;
        assert!(result.is_err());
    }
}
