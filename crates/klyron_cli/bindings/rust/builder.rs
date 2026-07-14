//! Builder pattern for klyron_cli

use crate::types::Klyron::CliConfig;

#[derive(Debug, Default)]
pub struct Klyron::CliBuilder {
    config: Option<Klyron::CliConfig>,
    verbose: bool,
}

impl Klyron::CliBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(mut self, config: Klyron::CliConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn build(self) -> anyhow::Result<Klyron::CliInstance> {
        let config = self.config.unwrap_or_default();
        Ok(Klyron::CliInstance { config, verbose: self.verbose })
    }
}

#[derive(Debug)]
pub struct Klyron::CliInstance {
    pub config: Klyron::CliConfig,
    pub verbose: bool,
}
