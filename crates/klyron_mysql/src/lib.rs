pub mod error;
pub mod migration;
pub mod pool;
pub mod query;
pub mod row;
pub mod transaction;

pub use error::{MySqlError, Result};
pub use migration::Migrator;
pub use pool::MySqlDb;
pub use query::SelectBuilder;
pub use row::Row;
pub use transaction::Transaction;

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

    #[test]
    fn test_insert_builder() {
        let q = query::InsertBuilder::new("users")
            .columns(&["name", "email"])
            .values(&["'alice'", "'a@b.com'"])
            .build();
        assert!(q.contains("INSERT INTO users"));
    }

    #[test]
    fn test_migration_table_name() {
        let m = Migrator::with_table("_my_migrations");
        assert_eq!(m.table, "_my_migrations");
    }

    #[tokio::test]
    async fn test_connect_fail() {
        let result = MySqlDb::connect("mysql://invalid:3306/nope").await;
        assert!(result.is_err());
    }
}
