pub mod prelude;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Plugin manifest metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub permissions: Vec<String>,
}

/// Plugin context passed to hook functions
#[derive(Debug, Clone)]
pub struct PluginContext {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
}

/// Result from a plugin hook
#[derive(Debug)]
pub struct HookResult {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

impl HookResult {
    pub fn ok(message: impl Into<String>) -> Self {
        Self { success: true, message: message.into(), data: None }
    }

    pub fn ok_with_data(message: impl Into<String>, data: serde_json::Value) -> Self {
        Self { success: true, message: message.into(), data: Some(data) }
    }

    pub fn err(message: impl Into<String>) -> Self {
        Self { success: false, message: message.into(), data: None }
    }
}

/// Runtime API for plugin access to Klyron features
pub struct RuntimeAPI {
    pub manifest: Manifest,
}

impl RuntimeAPI {
    pub fn new(manifest: Manifest) -> Self {
        Self { manifest }
    }

    pub fn log(&self, message: &str) {
        eprintln!("[klyron:{}] {}", self.manifest.name, message);
    }

    pub fn get_env(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }
}

/// Macro attribute placeholder for plugin function registration
/// Actual implementation uses the `#[klyron_plugin]` proc-macro
pub use klyron_plugin_macros::klyron_plugin;

/// Initialize a plugin from a manifest
pub fn init_plugin(name: &str) -> RuntimeAPI {
    let manifest = Manifest {
        name: name.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        description: String::new(),
        permissions: vec!["fs_read".into(), "stdout".into()],
    };
    RuntimeAPI::new(manifest)
}
