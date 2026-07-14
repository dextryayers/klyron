//! Client for klyron_compat
use crate::types::CompatConfig;

pub struct CompatClient {
    config: CompatConfig,
}

impl CompatClient {
    pub fn new(config: CompatConfig) -> Self {
        Self { config }
    }
    pub fn execute(&self) -> crate::errors::Result<()> {
        Ok(())
    }
}
