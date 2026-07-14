//! Client for klyron_ai

pub struct Klyron::AiClient {
    endpoint: String,
}

impl Klyron::AiClient {
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
