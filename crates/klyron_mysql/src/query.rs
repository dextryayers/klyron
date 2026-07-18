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

    pub fn columns(&mut self, cols: &[&str]) -> &mut Self {
        self.columns = cols.iter().map(|c| c.to_string()).collect();
        self
    }

    pub fn where_(&mut self, condition: &str) -> &mut Self {
        self.conditions.push(condition.to_string());
        self
    }

    pub fn order_by(&mut self, col: &str, asc: bool) -> &mut Self {
        self.order_by = Some((col.to_string(), asc));
        self
    }

    pub fn limit(&mut self, limit: u64) -> &mut Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(&mut self, offset: u64) -> &mut Self {
        self.offset = Some(offset);
        self
    }

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

#[derive(Debug, Clone)]
pub struct InsertBuilder {
    table: String,
    columns: Vec<String>,
    values: Vec<Vec<String>>,
}

impl InsertBuilder {
    pub fn new(table: &str) -> Self {
        Self {
            table: table.to_string(),
            columns: vec![],
            values: vec![],
        }
    }

    pub fn columns(&mut self, cols: &[&str]) -> &mut Self {
        self.columns = cols.iter().map(|c| c.to_string()).collect();
        self
    }

    pub fn values(&mut self, vals: &[&str]) -> &mut Self {
        self.values.push(vals.iter().map(|v| v.to_string()).collect());
        self
    }

    pub fn build(&self) -> String {
        let cols = self.columns.join(", ");
        let placeholders: Vec<String> = self.values.iter()
            .map(|row| format!("({})", row.iter().map(|v| format!("'{}'", v.replace('\'', "''"))).collect::<Vec<_>>().join(", ")))
            .collect();
        format!("INSERT INTO {} ({}) VALUES {}", self.table, cols, placeholders.join(", "))
    }
}
