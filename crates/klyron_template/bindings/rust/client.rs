//! Client for klyron_template
use crate::types::TemplateConfig;

pub struct TemplateClient {
    config: TemplateConfig,
}

impl TemplateClient {
    pub fn new(config: TemplateConfig) -> Self {
        Self { config }
    }
    pub fn execute(&self) -> crate::errors::Result<()> {
        Ok(())
    }
}
