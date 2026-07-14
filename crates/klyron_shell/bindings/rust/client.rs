//! Client for klyron_shell
use crate::types::ShellConfig;

pub struct ShellClient {
    config: ShellConfig,
}

impl ShellClient {
    pub fn new(config: ShellConfig) -> Self {
        Self { config }
    }
    pub fn execute(&self) -> crate::errors::Result<()> {
        Ok(())
    }
}
