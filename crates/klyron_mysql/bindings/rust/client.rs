//! Client for klyron_mysql

pub struct Klyron::MysqlClient {
    endpoint: String,
}

impl Klyron::MysqlClient {
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
