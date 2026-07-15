use anyhow::{Context, Result};
use dashmap::DashMap;
use notify::{Config, Event, EventKind, RecursiveMode, Watcher};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tracing::{info, warn};

pub mod manifest;
pub mod registry;
pub mod runtime;
pub mod sandbox;
pub mod wasm;

pub use manifest::{
    Permission, PluginHook, PluginLifecycle, PluginManifest,
};
pub use registry::PluginRegistry;
pub use runtime::PluginRuntime;
pub use sandbox::PluginSandbox;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestContext {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub query: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseContext {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventContext {
    pub event_type: String,
    pub source: String,
    pub payload: serde_json::Value,
    pub timestamp: String,
}

#[async_trait::async_trait]
pub trait PluginTrait: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn manifest(&self) -> &PluginManifest;

    async fn init(&mut self, config: &serde_json::Value) -> Result<()>;
    async fn start(&mut self) -> Result<()>;
    async fn stop(&mut self) -> Result<()>;

    async fn handle_request(&self, ctx: RequestContext) -> Result<ResponseContext>;
    async fn handle_event(&self, ctx: EventContext) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    FsRead,
    FsWrite,
    NetConnect,
    NetListen,
    EnvRead,
    EnvWrite,
    ProcessSpawn,
    All,
}

impl Permission {
    pub fn as_str(&self) -> &'static str {
        match self {
            Permission::FsRead => "fs_read",
            Permission::FsWrite => "fs_write",
            Permission::NetConnect => "net_connect",
            Permission::NetListen => "net_listen",
            Permission::EnvRead => "env_read",
            Permission::EnvWrite => "env_write",
            Permission::ProcessSpawn => "process_spawn",
            Permission::All => "all",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "fs_read" => Some(Permission::FsRead),
            "fs_write" => Some(Permission::FsWrite),
            "net_connect" => Some(Permission::NetConnect),
            "net_listen" => Some(Permission::NetListen),
            "env_read" => Some(Permission::EnvRead),
            "env_write" => Some(Permission::EnvWrite),
            "process_spawn" => Some(Permission::ProcessSpawn),
            "all" => Some(Permission::All),
            _ => None,
        }
    }

    pub fn all() -> Vec<Permission> {
        vec![
            Permission::FsRead, Permission::FsWrite,
            Permission::NetConnect, Permission::NetListen,
            Permission::EnvRead, Permission::EnvWrite,
            Permission::ProcessSpawn, Permission::All,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PluginLifecycle {
    Loading,
    Active,
    Paused,
    Error,
    Unloaded,
}

impl PluginLifecycle {
    pub fn as_str(&self) -> &'static str {
        match self {
            PluginLifecycle::Loading => "LOADING",
            PluginLifecycle::Active => "ACTIVE",
            PluginLifecycle::Paused => "PAUSED",
            PluginLifecycle::Error => "ERROR",
            PluginLifecycle::Unloaded => "UNLOADED",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PluginHook {
    PreInstall,
    PostInstall,
    PreBuild,
    PostBuild,
    PreDev,
    PostDev,
    PreRequest,
    PostRequest,
}

impl PluginHook {
    pub fn as_str(&self) -> &'static str {
        match self {
            PluginHook::PreInstall => "PRE_INSTALL",
            PluginHook::PostInstall => "POST_INSTALL",
            PluginHook::PreBuild => "PRE_BUILD",
            PluginHook::PostBuild => "POST_BUILD",
            PluginHook::PreDev => "PRE_DEV",
            PluginHook::PostDev => "POST_DEV",
            PluginHook::PreRequest => "PRE_REQUEST",
            PluginHook::PostRequest => "POST_REQUEST",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "PRE_INSTALL" | "pre_install" => Some(PluginHook::PreInstall),
            "POST_INSTALL" | "post_install" => Some(PluginHook::PostInstall),
            "PRE_BUILD" | "pre_build" => Some(PluginHook::PreBuild),
            "POST_BUILD" | "post_build" => Some(PluginHook::PostBuild),
            "PRE_DEV" | "pre_dev" => Some(PluginHook::PreDev),
            "POST_DEV" | "post_dev" => Some(PluginHook::PostDev),
            "PRE_REQUEST" | "pre_request" => Some(PluginHook::PreRequest),
            "POST_REQUEST" | "post_request" => Some(PluginHook::PostRequest),
            _ => None,
        }
    }

    pub fn all() -> Vec<PluginHook> {
        vec![
            PluginHook::PreInstall, PluginHook::PostInstall,
            PluginHook::PreBuild, PluginHook::PostBuild,
            PluginHook::PreDev, PluginHook::PostDev,
            PluginHook::PreRequest, PluginHook::PostRequest,
        ]
    }
}

pub fn find_config(dir: &Path) -> Option<PathBuf> {
    let candidates = ["klyron.json", "klyron.toml", "klyron.config.ts", "klyron.config.js"];
    let mut current = Some(dir);
    while let Some(d) = current {
        for name in &candidates {
            let candidate = d.join(name);
            if candidate.exists() {
                return Some(candidate);
            }
        }
        current = d.parent();
    }
    None
}

pub fn load_plugin_config(dir: &Path) -> Result<serde_json::Value> {
    let config_path = find_config(dir)
        .ok_or_else(|| anyhow::anyhow!("No config file found in {}", dir.display()))?;
    let content = std::fs::read_to_string(&config_path)?;
    match config_path.extension().and_then(|e| e.to_str()) {
        Some("json") => Ok(serde_json::from_str(&content)?),
        Some("toml") => {
            let value: toml::Value = toml::from_str(&content)?;
            let json_str = serde_json::to_string(&value)?;
            Ok(serde_json::from_str(&json_str)?)
        }
        _ => Ok(serde_json::from_str(&content)?),
    }
}

pub struct HotReloader {
    watcher: notify::RecommendedWatcher,
    shutdown_tx: mpsc::Sender<()>,
}

impl HotReloader {
    pub fn new<F>(plugins_dir: &Path, on_change: F) -> Result<Self>
    where
        F: Fn(String) + Send + 'static,
    {
        let (tx, mut rx) = mpsc::channel::<()>(16);

        let on_change = Arc::new(on_change);
        let plugins_dir = plugins_dir.to_path_buf();

        let mut watcher = notify::RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                        for path in event.paths {
                            if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                                on_change(name.to_string());
                            }
                        }
                    }
                }
            },
            Config::default(),
        )?;

        watcher.watch(&plugins_dir, RecursiveMode::NonRecursive)?;

        Ok(Self {
            watcher,
            shutdown_tx: tx,
        })
    }

    pub fn shutdown(self) {
        drop(self.shutdown_tx);
    }
}

pub fn verify_compatibility(
    api_version: &str,
    plugin_api_version: &str,
    force: bool,
) -> Result<bool> {
    let api_parts: Vec<u32> = api_version.split('.').filter_map(|p| p.parse().ok()).collect();
    let plugin_parts: Vec<u32> = plugin_api_version.split('.').filter_map(|p| p.parse().ok()).collect();

    let api_major = api_parts.first().copied().unwrap_or(0);
    let plugin_major = plugin_parts.first().copied().unwrap_or(0);

    if api_major != plugin_major {
        let msg = format!(
            "API version mismatch: klyron={}, plugin={}",
            api_major, plugin_major
        );
        if force {
            warn!("{} - forcing load", msg);
            return Ok(true);
        }
        anyhow::bail!("{}", msg);
    }

    Ok(true)
}

pub fn hash_bytes(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    hex::encode(Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_from_str() {
        assert_eq!(Permission::from_str("fs_read"), Some(Permission::FsRead));
        assert_eq!(Permission::from_str("all"), Some(Permission::All));
        assert_eq!(Permission::from_str("invalid"), None);
    }

    #[test]
    fn test_plugin_hook_from_str() {
        assert_eq!(PluginHook::from_str("PRE_INSTALL"), Some(PluginHook::PreInstall));
        assert_eq!(PluginHook::from_str("post_build"), Some(PluginHook::PostBuild));
        assert_eq!(PluginHook::from_str("invalid"), None);
    }

    #[test]
    fn test_lifecycle_as_str() {
        assert_eq!(PluginLifecycle::Active.as_str(), "ACTIVE");
        assert_eq!(PluginLifecycle::Unloaded.as_str(), "UNLOADED");
    }

    #[test]
    fn test_verify_compatibility() {
        assert!(verify_compatibility("1.0.0", "1.5.0", false).is_ok());
        assert!(verify_compatibility("2.0.0", "1.0.0", false).is_err());
        assert!(verify_compatibility("2.0.0", "1.0.0", true).is_ok());
    }

    #[test]
    fn test_hash_bytes() {
        let hash = hash_bytes(b"hello");
        assert_eq!(hash.len(), 64);
    }
}
