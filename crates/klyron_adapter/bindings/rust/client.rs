//! Client for klyron_adapter
use crate::types::AdapterConfig;

pub struct AdapterClient {
    config: AdapterConfig,
}

impl AdapterClient {
    pub fn new(config: AdapterConfig) -> Self {
        Self { config }
    }
    pub fn execute(&self) -> crate::errors::Result<()> {
        Ok(())
    }
}
