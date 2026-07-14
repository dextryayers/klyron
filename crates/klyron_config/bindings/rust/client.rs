//! Client for klyron_config
use crate::types::ConfigConfig;

pub struct ConfigClient {
    config: ConfigConfig,
}

impl ConfigClient {
    pub fn new(config: ConfigConfig) -> Self {
        Self { config }
    }
    pub fn execute(&self) -> crate::errors::Result<()> {
        Ok(())
    }
}
