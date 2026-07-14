//! Client for klyron_sqlite

pub struct Klyron::SqliteClient {
    endpoint: String,
}

impl Klyron::SqliteClient {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self { endpoint: endpoint.into() }
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub fn connect(&self) -> anyhow::Result<()> {
        Ok(())
    }

    pub fn ping(&self) -> anyhow::Result<bool> {
        Ok(true)
    }
}
