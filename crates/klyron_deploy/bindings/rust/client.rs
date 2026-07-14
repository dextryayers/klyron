//! Client for klyron_deploy
use crate::types::DeployConfig;

pub struct DeployClient {
    config: DeployConfig,
}

impl DeployClient {
    pub fn new(config: DeployConfig) -> Self {
        Self { config }
    }
    pub fn execute(&self) -> crate::errors::Result<()> {
        Ok(())
    }
}
