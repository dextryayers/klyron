//! Builder pattern for klyron_postgres

use crate::types::Klyron::PostgresConfig;

#[derive(Debug, Default)]
pub struct Klyron::PostgresBuilder {
    config: Option<Klyron::PostgresConfig>,
    verbose: bool,
}

impl Klyron::PostgresBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(mut self, config: Klyron::PostgresConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn build(self) -> anyhow::Result<Klyron::PostgresInstance> {
        let config = self.config.unwrap_or_default();
        Ok(Klyron::PostgresInstance { config, verbose: self.verbose })
    }
}

#[derive(Debug)]
pub struct Klyron::PostgresInstance {
    pub config: Klyron::PostgresConfig,
    pub verbose: bool,
}
