//! Client for klyron_workspace
use crate::types::WorkspaceConfig;

pub struct WorkspaceClient {
    config: WorkspaceConfig,
}

impl WorkspaceClient {
    pub fn new(config: WorkspaceConfig) -> Self {
        Self { config }
    }
    pub fn execute(&self) -> crate::errors::Result<()> {
        Ok(())
    }
}
