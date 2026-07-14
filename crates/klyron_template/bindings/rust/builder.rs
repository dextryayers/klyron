//! Builder for klyron_template
use crate::types::TemplateConfig;

pub struct TemplateBuilder {
    config: TemplateConfig,
}

impl TemplateBuilder {
    pub fn new() -> Self {
        Self { config: TemplateConfig::default() }
    }
    pub fn with_version(mut self, version: &str) -> Self {
        self.config.version = version.to_string();
        self
    }
    pub fn build(self) -> TemplateConfig {
        self.config
    }
}
