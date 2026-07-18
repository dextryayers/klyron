use sqlx::mysql::MySqlRow;
use sqlx::{Column, Row as SqlxRow, TypeInfo};

#[derive(Debug, Clone)]
pub struct Row {
    pub columns: Vec<String>,
    pub values: Vec<serde_json::Value>,
}

impl Row {
    #[inline]
    pub fn get(&self, index: usize) -> Option<&serde_json::Value> {
        self.values.get(index)
    }

    #[inline]
    pub fn get_str(&self, index: usize) -> Option<&str> {
        self.values.get(index)?.as_str()
    }

    #[inline]
    pub fn get_int(&self, index: usize) -> Option<i64> {
        self.values.get(index)?.as_i64()
    }

    #[inline]
    pub fn get_float(&self, index: usize) -> Option<f64> {
        self.values.get(index)?.as_f64()
    }

    #[inline]
    pub fn is_null(&self, index: usize) -> bool {
        self.values.get(index).map_or(true, serde_json::Value::is_null)
    }

    #[inline]
    pub fn columns(&self) -> &[String] {
        &self.columns
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

pub fn mysql_row_to_row(row: &MySqlRow) -> Row {
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
