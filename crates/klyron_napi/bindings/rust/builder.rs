use crate::types::{NapiLoaderConfig, NapiModule};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct NapiLoaderBuilder {
    config: NapiLoaderConfig,
}

impl NapiLoaderBuilder {
    pub fn new() -> Self {
        Self { config: NapiLoaderConfig::default() }
    }

    pub fn module_path(mut self, path: &str) -> Self {
        self.config.module_paths.push(path.to_string());
        self
    }

    pub fn cache_enabled(mut self, enabled: bool) -> Self {
        self.config.cache_enabled = enabled;
        self
    }

    pub fn build(self) -> super::NapiLoader {
        super::NapiLoader::with_config(self.config)
    }
}

impl Default for NapiLoaderBuilder {
    fn default() -> Self { Self::new() }
}
