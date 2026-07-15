use crate::manifest::{PluginManifest, PluginMetadata, PluginState};
use crate::runtime::{hash_wasm, LoadedPlugin, PluginRuntime};
use crate::sandbox::{PluginSandbox, SandboxLimits};
use crate::{hash_bytes, PluginLifecycle, PluginTrait};
use anyhow::{Context, Result};
use chrono::Utc;
use dashmap::DashMap;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tracing::{info, warn};

type PluginBox = Box<dyn PluginTrait + Send + Sync>;

pub struct PluginRegistry {
    native_plugins: DashMap<String, Arc<PluginBox>>,
    wasm_plugins: DashMap<String, Arc<parking_lot::RwLock<LoadedPlugin>>>,
    plugin_states: DashMap<String, PluginState>,
    runtime: PluginRuntime,
    sandbox: Arc<PluginSandbox>,
    plugins_dir: PathBuf,
    listeners: Vec<Box<dyn Fn(PluginEvent) + Send + Sync>>,
}

#[derive(Debug, Clone)]
pub enum PluginEvent {
    Registered { name: String, version: String },
    Unregistered { name: String },
    Loaded { name: String },
    Unloaded { name: String },
    Enabled { name: String },
    Disabled { name: String },
    Error { name: String, error: String },
}

impl PluginRegistry {
    pub fn new() -> Result<Self> {
        let sandbox = Arc::new(PluginSandbox::with_defaults());
        let runtime = PluginRuntime::new(sandbox.clone())?;
        let plugins_dir = dirs::home_dir()
            .map(|p| p.join(".klyron").join("plugins"))
            .unwrap_or_else(|| PathBuf::from("/tmp/klyron-plugins"));

        Ok(Self {
            native_plugins: DashMap::new(),
            wasm_plugins: DashMap::new(),
            plugin_states: DashMap::new(),
            runtime,
            sandbox,
            plugins_dir,
            listeners: Vec::new(),
        })
    }

    pub fn with_plugins_dir(mut self, dir: PathBuf) -> Self {
        self.plugins_dir = dir;
        self
    }

    pub fn with_limits(mut self, max_fuel: u64, max_memory: u64, max_cpu_ms: u64) -> Self {
        let limits = SandboxLimits {
            max_memory_bytes: max_memory,
            max_fuel,
            max_cpu_ms,
            ..SandboxLimits::default()
        };
        self.sandbox = Arc::new(PluginSandbox::new(limits));
        self.runtime = PluginRuntime::new(self.sandbox.clone())
            .expect("Failed to create runtime with custom limits");
        self
    }

    pub fn on_event<F>(&mut self, callback: F)
    where
        F: Fn(PluginEvent) + Send + Sync + 'static,
    {
        self.listeners.push(Box::new(callback));
    }

    fn emit(&self, event: PluginEvent) {
        for listener in &self.listeners {
            listener(event.clone());
        }
    }

    pub fn register_native(&self, plugin: PluginBox) {
        let name = plugin.name().to_string();
        let version = plugin.version().to_string();
        let metadata = PluginMetadata::from_manifest(plugin.manifest());
        let state = PluginState::new(metadata);
        self.native_plugins.insert(name.clone(), Arc::new(plugin));
        self.plugin_states.insert(name.clone(), state);
        self.emit(PluginEvent::Registered { name, version });
    }

    pub fn register_wasm(
        &self,
        name: &str,
        wasm_bytes: Vec<u8>,
        manifest: PluginManifest,
    ) -> Result<()> {
        let (instance, store) = self.runtime.instantiate(&wasm_bytes, &manifest)?;
        let wasm_hash = hash_wasm(&wasm_bytes);
        let loaded = LoadedPlugin {
            manifest: manifest.clone(),
            instance,
            store,
            wasm_path: PathBuf::new(),
            wasm_hash,
            enabled: true,
            load_time: Instant::now(),
        };

        let metadata = PluginMetadata::from_manifest(&manifest);
        let mut state = PluginState::new(metadata);
        state.lifecycle = PluginLifecycle::Active;
        state.loaded_at = Some(Utc::now().to_rfc3339());

        self.wasm_plugins.insert(
            name.to_string(),
            Arc::new(parking_lot::RwLock::new(loaded)),
        );
        self.plugin_states.insert(name.to_string(), state);
        self.emit(PluginEvent::Loaded {
            name: name.to_string(),
        });
        info!("Loaded WASM plugin: {}", name);
        Ok(())
    }

    pub fn unregister(&self, name: &str) -> Result<()> {
        let removed = self.native_plugins.remove(name).or_else(|| {
            self.wasm_plugins.remove(name).map(|(k, _)| (k, ()))
        });

        if removed.is_some() {
            if let Some(mut state) = self.plugin_states.get_mut(name) {
                state.lifecycle = PluginLifecycle::Unloaded;
            }
            self.emit(PluginEvent::Unregistered {
                name: name.to_string(),
            });
            info!("Unregistered plugin: {}", name);
            Ok(())
        } else {
            anyhow::bail!("Plugin '{}' not found", name)
        }
    }

    pub fn get_native(&self, name: &str) -> Option<Arc<PluginBox>> {
        self.native_plugins.get(name).map(|e| e.value().clone())
    }

    pub fn get_wasm(&self, name: &str) -> Option<Arc<parking_lot::RwLock<LoadedPlugin>>> {
        self.wasm_plugins.get(name).map(|e| e.value().clone())
    }

    pub fn get_state(&self, name: &str) -> Option<PluginState> {
        self.plugin_states.get(name).map(|e| e.value().clone())
    }

    pub fn list(&self) -> Vec<String> {
        let mut names: Vec<String> = self.native_plugins.iter().map(|e| e.key().clone()).collect();
        names.extend(self.wasm_plugins.iter().map(|e| e.key().clone()));
        names.sort();
        names
    }

    pub fn list_states(&self) -> Vec<(String, PluginState)> {
        let mut states = Vec::new();
        for entry in self.plugin_states.iter() {
            states.push((entry.key().clone(), entry.value().clone()));
        }
        states.sort_by(|a, b| a.0.cmp(&b.0));
        states
    }

    pub fn enable(&self, name: &str) -> Result<()> {
        if let Some(mut state) = self.plugin_states.get_mut(name) {
            state.config.enabled = true;
            state.lifecycle = PluginLifecycle::Active;
            self.emit(PluginEvent::Enabled {
                name: name.to_string(),
            });
            info!("Enabled plugin: {}", name);
            Ok(())
        } else {
            anyhow::bail!("Plugin '{}' not found", name)
        }
    }

    pub fn disable(&self, name: &str) -> Result<()> {
        if let Some(mut state) = self.plugin_states.get_mut(name) {
            state.config.enabled = false;
            state.lifecycle = PluginLifecycle::Paused;
            self.emit(PluginEvent::Disabled {
                name: name.to_string(),
            });
            info!("Disabled plugin: {}", name);
            Ok(())
        } else {
            anyhow::bail!("Plugin '{}' not found", name)
        }
    }

    pub fn is_enabled(&self, name: &str) -> bool {
        self.plugin_states
            .get(name)
            .map(|s| s.config.enabled)
            .unwrap_or(false)
    }

    pub fn load_all(&self) -> Vec<String> {
        let mut loaded = Vec::new();
        if !self.plugins_dir.exists() {
            return loaded;
        }
        if let Ok(entries) = std::fs::read_dir(&self.plugins_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let name = path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string());
                let name = match name {
                    Some(n) => n,
                    None => continue,
                };
                let wasm_path = path.join(format!("{}.wasm", name));
                let manifest_path = path.join("klyron-plugin.json");
                if !wasm_path.exists() || !manifest_path.exists() {
                    continue;
                }
                let content = match std::fs::read_to_string(&manifest_path) {
                    Ok(c) => c,
                    Err(_) => continue,
                };
                let manifest: PluginManifest = match serde_json::from_str(&content) {
                    Ok(m) => m,
                    Err(_) => continue,
                };
                let wasm_bytes = match std::fs::read(&wasm_path) {
                    Ok(b) => b,
                    Err(_) => continue,
                };
                if self.register_wasm(&name, wasm_bytes, manifest).is_ok() {
                    loaded.push(name);
                }
            }
        }
        loaded
    }

    pub fn count(&self) -> usize {
        self.native_plugins.len() + self.wasm_plugins.len()
    }

    pub fn plugins_dir(&self) -> &Path {
        &self.plugins_dir
    }

    pub fn runtime(&self) -> &PluginRuntime {
        &self.runtime
    }

    pub fn sandbox(&self) -> &PluginSandbox {
        &self.sandbox
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new().expect("Failed to create default PluginRegistry")
    }
}
