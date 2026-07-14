use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NapiModule {
    pub name: String,
    pub exports: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NapiLoaderConfig {
    pub module_paths: Vec<String>,
    pub cache_enabled: bool,
}

impl Default for NapiLoaderConfig {
    fn default() -> Self {
        Self {
            module_paths: vec!["node_modules".to_string()],
            cache_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NapiVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}
