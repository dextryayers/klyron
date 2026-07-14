//! Builder pattern for klyron_sqlite

use crate::types::Klyron::SqliteConfig;

#[derive(Debug, Default)]
pub struct Klyron::SqliteBuilder {
    config: Option<Klyron::SqliteConfig>,
    verbose: bool,
}

impl Klyron::SqliteBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(mut self, config: Klyron::SqliteConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn build(self) -> anyhow::Result<Klyron::SqliteInstance> {
        let config = self.config.unwrap_or_default();
        Ok(Klyron::SqliteInstance { config, verbose: self.verbose })
    }
}

#[derive(Debug)]
pub struct Klyron::SqliteInstance {
    pub config: Klyron::SqliteConfig,
    pub verbose: bool,
}
