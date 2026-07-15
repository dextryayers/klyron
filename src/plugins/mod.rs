pub mod registry;
pub mod types;
pub mod manager;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use types::{PluginConfig, PluginMetadata};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub enabled: bool,
    pub config: PluginConfig,
    pub metadata: PluginMetadata,
    pub install_path: Option<PathBuf>,
}

pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn metadata(&self) -> &PluginMetadata;
    fn initialize(&mut self, config: &PluginConfig) -> Result<()>;
    fn cleanup(&mut self) -> Result<()>;
}
