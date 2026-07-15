use serde::{Deserialize, Serialize};
use crate::{Permission, PluginHook, PluginLifecycle};

pub const KLYRON_API_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub homepage: Option<String>,
    pub license: Option<String>,
    pub klyron_api: Option<String>,
    pub permissions: Vec<String>,
    pub hooks: Option<Vec<String>>,
    pub dependencies: Option<Vec<PluginDependency>>,
    pub sandbox: Option<SandboxConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    pub name: String,
    pub version: String,
    pub optional: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub max_memory_bytes: Option<u64>,
    pub max_fuel: Option<u64>,
    pub max_cpu_ms: Option<u64>,
    pub allowed_domains: Option<Vec<String>>,
    pub allowed_paths: Option<Vec<String>>,
    pub allowed_env: Option<Vec<String>>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            max_memory_bytes: Some(64 * 1024 * 1024),
            max_fuel: Some(1_000_000),
            max_cpu_ms: Some(5000),
            allowed_domains: None,
            allowed_paths: None,
            allowed_env: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub license: Option<String>,
}

impl PluginMetadata {
    pub fn from_manifest(m: &PluginManifest) -> Self {
        Self {
            name: m.name.clone(),
            version: m.version.clone(),
            author: m.author.clone(),
            description: m.description.clone(),
            homepage: m.homepage.clone(),
            license: m.license.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub enabled: bool,
    pub permissions: Vec<Permission>,
    pub settings: serde_json::Value,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            permissions: Vec::new(),
            settings: serde_json::Value::Object(serde_json::Map::new()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPackage {
    pub name: String,
    pub version: String,
    pub source: PluginSource,
    pub integrity: String,
    pub manifest: PluginManifest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PluginSource {
    Registry { url: String, id: String },
    Local { path: String },
    Git { url: String, rev: Option<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginEntry {
    pub name: String,
    pub version: String,
    pub enabled: bool,
    pub permissions: Vec<Permission>,
    pub config: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginState {
    pub lifecycle: PluginLifecycle,
    pub config: PluginConfig,
    pub metadata: PluginMetadata,
    pub loaded_at: Option<String>,
    pub error: Option<String>,
    pub hook_count: u64,
    pub request_count: u64,
}

impl PluginState {
    pub fn new(metadata: PluginMetadata) -> Self {
        Self {
            lifecycle: PluginLifecycle::Unloaded,
            config: PluginConfig::default(),
            metadata,
            loaded_at: None,
            error: None,
            hook_count: 0,
            request_count: 0,
        }
    }
}

pub fn default_manifest(name: &str) -> PluginManifest {
    PluginManifest {
        name: name.to_string(),
        version: "0.1.0".to_string(),
        description: None,
        author: None,
        homepage: None,
        license: None,
        klyron_api: Some(KLYRON_API_VERSION.to_string()),
        permissions: Vec::new(),
        hooks: None,
        dependencies: None,
        sandbox: None,
    }
}
