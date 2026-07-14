//! Builder pattern for klyron_mysql

use crate::types::Klyron::MysqlConfig;

#[derive(Debug, Default)]
pub struct Klyron::MysqlBuilder {
    config: Option<Klyron::MysqlConfig>,
    verbose: bool,
}

impl Klyron::MysqlBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(mut self, config: Klyron::MysqlConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn build(self) -> anyhow::Result<Klyron::MysqlInstance> {
        let config = self.config.unwrap_or_default();
        Ok(Klyron::MysqlInstance { config, verbose: self.verbose })
    }
}

#[derive(Debug)]
pub struct Klyron::MysqlInstance {
    pub config: Klyron::MysqlConfig,
    pub verbose: bool,
}
