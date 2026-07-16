pub mod manifest;
pub mod hooks;
pub mod runtime;
pub mod sandbox;

use anyhow::{Context, Result};
use chrono::Utc;
use manifest::{default_compat, PluginInfo, PluginMarketplaceEntry, KLYRON_API_VERSION};
use hooks::{HookHandler, HookRegistry, HookResult};
use runtime::{hash_wasm, verify_compatibility, LoadedPlugin, PluginLoadResult, PluginRuntime};
use sandbox::{Sandbox, SandboxLimits, SandboxTestHarness};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tracing::{info, warn};

pub use manifest::{HookPhase, PluginDependency, PluginManifest, SandboxConfig};

const PLUGINS_DIR_NAME: &str = "plugins";

#[derive(Debug, Clone)]
pub enum PluginEvent {
    Installed { name: String, version: String },
    Removed { name: String },
    Enabled { name: String },
    Disabled { name: String },
    HookExecuted { name: String, phase: String, duration_ms: u64 },
    HookFailed { name: String, phase: String, error: String },
    RolledBack { name: String, phase: String },
    Updated { name: String, old_version: String, new_version: String },
    Error { name: String, error: String },
}

pub struct PluginRegistry {
    plugins: HashMap<String, LoadedPlugin>,
    runtime: PluginRuntime,
    hook_registry: HookRegistry,
    sandbox: Arc<Sandbox>,
    plugins_dir: PathBuf,
    listeners: Vec<Arc<dyn Fn(PluginEvent) + Send + Sync>>,
    installed_info: HashMap<String, PluginInfo>,
    rollback_stack: Vec<RollbackEntry>,
    no_rollback: bool,
}

enum RollbackEntry {
    Install {
        name: String,
        wasm_path: PathBuf,
        manifest_path: PathBuf,
    },
    Enable {
        name: String,
        was_enabled: bool,
    },
}

impl PluginRegistry {
    pub fn new() -> Result<Self> {
        let sandbox = Arc::new(Sandbox::with_defaults());
        let runtime = PluginRuntime::new(Some(sandbox.clone()))?;

        let plugins_dir = dirs::home_dir()
            .map(|p| p.join(".klyron").join(PLUGINS_DIR_NAME))
            .unwrap_or_else(|| PathBuf::from("/tmp/klyron-plugins"));

        Ok(Self {
            plugins: HashMap::new(),
            runtime,
            hook_registry: HookRegistry::new(),
            sandbox,
            plugins_dir,
            listeners: Vec::new(),
            installed_info: HashMap::new(),
            rollback_stack: Vec::new(),
            no_rollback: false,
        })
    }

    pub fn with_plugins_dir(mut self, dir: PathBuf) -> Self {
        self.plugins_dir = dir;
        self
    }

    pub fn with_no_rollback(mut self) -> Self {
        self.no_rollback = true;
        self
    }

    pub fn with_limits(mut self, max_fuel: u64, max_memory: u64, max_cpu_ms: u64) -> Self {
        let limits = SandboxLimits {
            max_memory_bytes: max_memory,
            max_fuel,
            max_cpu_ms,
            ..SandboxLimits::default()
        };
        self.sandbox = Arc::new(Sandbox::new(limits));
        self.runtime = PluginRuntime::new(Some(self.sandbox.clone()))
            .expect("Failed to create runtime with custom limits");
        self
    }

    pub fn on_event<F>(&mut self, callback: F)
    where
        F: Fn(PluginEvent) + Send + Sync + 'static,
    {
        self.listeners.push(Arc::new(callback));
    }

    fn emit(&self, event: PluginEvent) {
        for listener in &self.listeners {
            listener(event.clone());
        }
    }

    // ── Dependency Graph ────────────────────────────────────────────────

    pub fn resolve_dependency_order(&self) -> Result<Vec<String>> {
        let mut visited = HashSet::new();
        let mut in_stack = HashSet::new();
        let mut order = Vec::new();

        let names: Vec<String> = self.plugins.keys().cloned().collect();
        for name in &names {
            if !visited.contains(name) {
                self.dfs_topological(name, &mut visited, &mut in_stack, &mut order)?;
            }
        }

        Ok(order)
    }

    fn dfs_topological(
        &self,
        name: &str,
        visited: &mut HashSet<String>,
        in_stack: &mut HashSet<String>,
        order: &mut Vec<String>,
    ) -> Result<()> {
        visited.insert(name.to_string());
        in_stack.insert(name.to_string());

        let deps = self
            .plugins
            .get(name)
            .and_then(|p| p.manifest.dependencies.clone())
            .unwrap_or_default();

        for dep in &deps {
            if in_stack.contains(&dep.name) {
                anyhow::bail!(
                    "Dependency cycle detected: {} -> {}",
                    name,
                    dep.name
                );
            }
            if !visited.contains(&dep.name) {
                if self.plugins.contains_key(&dep.name) {
                    self.dfs_topological(&dep.name, visited, in_stack, order)?;
                } else if !dep.optional.unwrap_or(false) {
                    anyhow::bail!(
                        "Required dependency '{}' for plugin '{}' is not installed",
                        dep.name,
                        name
                    );
                }
            }
        }

        in_stack.remove(name);
        order.push(name.to_string());
        Ok(())
    }

    pub fn detect_cycles(&self) -> Vec<String> {
        let mut cycles = Vec::new();
        let names: Vec<String> = self.plugins.keys().cloned().collect();

        for name in &names {
            let mut visited = HashSet::new();
            let mut stack = Vec::new();
            if self.detect_cycle_recursive(name, &mut visited, &mut stack) {
                cycles.push(name.clone());
            }
        }

        cycles
    }

    fn detect_cycle_recursive(
        &self,
        name: &str,
        visited: &mut HashSet<String>,
        stack: &mut Vec<String>,
    ) -> bool {
        if stack.contains(&name.to_string()) {
            return true;
        }
        if visited.contains(name) {
            return false;
        }

        visited.insert(name.to_string());
        stack.push(name.to_string());

        let deps = self
            .plugins
            .get(name)
            .and_then(|p| p.manifest.dependencies.clone())
            .unwrap_or_default();

        for dep in &deps {
            if self.plugins.contains_key(&dep.name) {
                if self.detect_cycle_recursive(&dep.name, visited, stack) {
                    return true;
                }
            }
        }

        stack.pop();
        false
    }

    // ── Install / Load / Unload ─────────────────────────────────────────

    pub fn install(&mut self, source: &Path, force: bool) -> Result<PluginLoadResult> {
        let wasm_bytes = std::fs::read(source)
            .with_context(|| format!("Failed to read plugin file: {}", source.display()))?;

        let manifest = self.parse_manifest(source)?;
        let compat = verify_compatibility(&manifest, force)?;
        let wasm_hash = hash_wasm(&wasm_bytes);
        let size_bytes = wasm_bytes.len() as u64;

        std::fs::create_dir_all(&self.plugins_dir)?;

        let plugin_dir = self.plugins_dir.join(&manifest.name);
        std::fs::create_dir_all(&plugin_dir)?;

        let wasm_dest = plugin_dir.join(format!("{}.wasm", manifest.name));
        std::fs::copy(source, &wasm_dest)?;

        let manifest_path = plugin_dir.join("klyron-plugin.json");
        let manifest_json = serde_json::to_string_pretty(&manifest)?;
        std::fs::write(&manifest_path, &manifest_json)?;

        let load_result = PluginLoadResult {
            name: manifest.name.clone(),
            version: manifest.version.clone(),
            manifest: manifest.clone(),
            wasm_hash: hex::encode(&wasm_hash),
            compat: compat.clone(),
            is_compatible: true,
            size_bytes,
            load_duration_ms: 0,
        };

        let info = PluginInfo {
            manifest: manifest.clone(),
            enabled: true,
            install_path: plugin_dir.to_string_lossy().to_string(),
            installed_at: Utc::now().to_rfc3339(),
            wasm_hash: hex::encode(&wasm_hash),
            size_bytes,
            compat: compat.clone(),
        };
        self.installed_info.insert(manifest.name.clone(), info);

        if !self.no_rollback {
            self.rollback_stack.push(RollbackEntry::Install {
                name: manifest.name.clone(),
                wasm_path: wasm_dest.clone(),
                manifest_path,
            });
        }

        let (instance, store) = self.runtime.instantiate(&wasm_bytes, &manifest)?;

        let loaded = LoadedPlugin {
            manifest: manifest.clone(),
            instance,
            store,
            wasm_path: wasm_dest,
            wasm_hash,
            compat,
            enabled: true,
            load_time: Instant::now(),
        };

        self.plugins.insert(manifest.name.clone(), loaded);

        self.emit(PluginEvent::Installed {
            name: manifest.name.clone(),
            version: manifest.version.clone(),
        });

        info!("Installed plugin: {} v{}", manifest.name, manifest.version);
        Ok(load_result)
    }

    pub fn load(&mut self, path: &str, force: bool) -> Result<String> {
        let source = Path::new(path);
        let wasm_bytes = std::fs::read(source)
            .with_context(|| format!("Failed to read WASM file: {}", source.display()))?;

        let manifest = self.parse_manifest(source)?;
        let compat = verify_compatibility(&manifest, force)?;
        let wasm_hash = hash_wasm(&wasm_bytes);

        let (instance, store) = self.runtime.instantiate(&wasm_bytes, &manifest)?;

        let name = manifest.name.clone();
        let loaded = LoadedPlugin {
            manifest,
            instance,
            store,
            wasm_path: source.to_path_buf(),
            wasm_hash,
            compat,
            enabled: true,
            load_time: Instant::now(),
        };

        self.register_hooks_for_plugin(&loaded);

        self.plugins.insert(name.clone(), loaded);
        info!("Loaded plugin: {}", name);
        Ok(name)
    }

    pub fn unload(&mut self, name: &str) -> Result<()> {
        if let Some(plugin) = self.plugins.remove(name) {
            self.hook_registry.unregister(name);

            self.emit(PluginEvent::Removed {
                name: name.to_string(),
            });

            info!("Unloaded plugin: {}", name);
            Ok(())
        } else {
            anyhow::bail!("Plugin '{}' not found", name)
        }
    }

    pub fn remove(&mut self, name: &str) -> Result<()> {
        self.unload(name)?;

        let plugin_dir = self.plugins_dir.join(name);
        if plugin_dir.exists() {
            std::fs::remove_dir_all(&plugin_dir)?;
        }

        self.installed_info.remove(name);
        info!("Removed plugin: {}", name);
        Ok(())
    }

    fn parse_manifest(&self, wasm_path: &Path) -> Result<PluginManifest> {
        let wasm_dir = wasm_path.parent().unwrap_or(Path::new("."));

        for manifest_name in &[
            "klyron-plugin.json",
            "klyron-plugin.toml",
            "plugin.toml",
            "klyron.json",
        ] {
            let manifest_path = wasm_dir.join(manifest_name);
            if manifest_path.exists() {
                let content = std::fs::read_to_string(&manifest_path)?;
                if manifest_name.ends_with(".json") {
                    return Ok(serde_json::from_str(&content)?);
                } else {
                    return Ok(toml::from_str(&content)?);
                }
            }
        }

        Ok(PluginManifest {
            name: wasm_path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            version: "0.1.0".to_string(),
            description: None,
            authors: None,
            license: None,
            klyron_api: Some(KLYRON_API_VERSION.to_string()),
            permissions: Vec::new(),
            dependencies: None,
            hooks: None,
            sandbox: None,
        })
    }

    fn register_hooks_for_plugin(&mut self, plugin: &LoadedPlugin) {
        if let Some(ref hooks) = plugin.manifest.hooks {
            for hook_name in hooks {
                let phase = HookPhase::from_str(hook_name);
                if let Some(phase) = phase {
                    let plugin_name = plugin.manifest.name.clone();
                    let handler = Arc::new(move |name: &str, ctx: &[u8]| -> Result<Vec<u8>> {
                        if name == plugin_name {
                            Ok(ctx.to_vec())
                        } else {
                            Ok(Vec::new())
                        }
                    });
                    self.hook_registry.register(HookHandler {
                        plugin_name: plugin.manifest.name.clone(),
                        phase,
                        handler,
                    });
                }
            }
        }
    }

    // ── Hook Execution ──────────────────────────────────────────────────

    pub fn execute_hooks(&self, phase: &HookPhase, context: &[u8]) -> Vec<HookResult> {
        self.hook_registry
            .execute_phase_with_rollback(phase, context)
    }

    pub fn execute_hooks_with_rollback(
        &mut self,
        phase: &HookPhase,
        context: &[u8],
    ) -> Vec<HookResult> {
        let results = self.hook_registry.execute_phase(phase, context);
        let has_failure = results.iter().any(|r| matches!(r, HookResult::Failure { .. }));

        if has_failure && !self.no_rollback {
            for result in &results {
                if let HookResult::Failure { plugin, error, .. } = result {
                    warn!("Rolling back hook {} for plugin {}: {}", phase.as_str(), plugin, error);
                    self.emit(PluginEvent::RolledBack {
                        name: plugin.clone(),
                        phase: phase.as_str().to_string(),
                    });
                }
                if let HookResult::Success { plugin, .. } = result {
                    self.emit(PluginEvent::RolledBack {
                        name: plugin.clone(),
                        phase: phase.as_str().to_string(),
                    });
                }
            }
        }

        for result in &results {
            match result {
                HookResult::Success { plugin, duration, .. } => {
                    self.emit(PluginEvent::HookExecuted {
                        name: plugin.clone(),
                        phase: phase.as_str().to_string(),
                        duration_ms: duration.as_millis() as u64,
                    });
                }
                HookResult::Failure { plugin, error, .. } => {
                    self.emit(PluginEvent::HookFailed {
                        name: plugin.clone(),
                        phase: phase.as_str().to_string(),
                        error: error.clone(),
                    });
                }
            }
        }

        results
    }

    // ── Query ───────────────────────────────────────────────────────────

    pub fn list(&self) -> Vec<&str> {
        self.plugins.keys().map(|s| s.as_str()).collect()
    }

    pub fn get_manifest(&self, name: &str) -> Option<&PluginManifest> {
        self.plugins.get(name).map(|p| &p.manifest)
    }

    pub fn get_info(&self, name: &str) -> Option<&PluginInfo> {
        self.installed_info.get(name)
    }

    pub fn get_all_info(&self) -> Vec<&PluginInfo> {
        self.installed_info.values().collect()
    }

    pub fn is_loaded(&self, name: &str) -> bool {
        self.plugins.contains_key(name)
    }

    pub fn is_enabled(&self, name: &str) -> bool {
        self.installed_info
            .get(name)
            .map(|i| i.enabled)
            .unwrap_or(false)
    }

    pub fn toggle(&mut self, name: &str) -> Result<bool> {
        let info = self
            .installed_info
            .get_mut(name)
            .ok_or_else(|| anyhow::anyhow!("Plugin '{}' not found", name))?;

        info.enabled = !info.enabled;
        let enabled = info.enabled;
        let name_owned = name.to_string();
        drop(info);

        if !self.no_rollback {
            self.rollback_stack.push(RollbackEntry::Enable {
                name: name_owned.clone(),
                was_enabled: !enabled,
            });
        }

        if enabled {
            self.emit(PluginEvent::Enabled {
                name: name_owned,
            });
        } else {
            self.emit(PluginEvent::Disabled {
                name: name_owned,
            });
        }

        Ok(enabled)
    }

    pub fn rollback(&mut self) -> Result<()> {
        while let Some(entry) = self.rollback_stack.pop() {
            match entry {
                RollbackEntry::Install { name, wasm_path, manifest_path } => {
                    self.plugins.remove(&name);
                    self.installed_info.remove(&name);
                    self.hook_registry.unregister(&name);
                    if wasm_path.exists() {
                        let _ = std::fs::remove_file(&wasm_path);
                    }
                    if manifest_path.exists() {
                        let _ = std::fs::remove_file(&manifest_path);
                    }
                    let plugin_dir = wasm_path.parent().unwrap_or(Path::new(""));
                    if plugin_dir.exists() {
                        let _ = std::fs::remove_dir(plugin_dir);
                    }
                    warn!("Rolled back install of plugin: {}", name);
                }
                RollbackEntry::Enable { name, was_enabled } => {
                    if let Some(info) = self.installed_info.get_mut(&name) {
                        info.enabled = was_enabled;
                    }
                    warn!("Rolled back toggle of plugin: {}", name);
                }
            }
        }
        Ok(())
    }

    // ── Update ──────────────────────────────────────────────────────────

    pub fn update(&mut self, name: &str, new_source: &Path, force: bool) -> Result<PluginLoadResult> {
        let old_info = self
            .installed_info
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Plugin '{}' is not installed", name))?;

        let old_version = old_info.manifest.version.clone();

        self.remove(name)?;
        let result = self.install(new_source, force)?;

        self.emit(PluginEvent::Updated {
            name: name.to_string(),
            old_version,
            new_version: result.version.clone(),
        });

        Ok(result)
    }

    // ── Search ──────────────────────────────────────────────────────────

    pub fn search(&self, _query: &str) -> Vec<PluginMarketplaceEntry> {
        Vec::new()
    }

    pub fn refresh_installed(&mut self) -> Result<()> {
        if !self.plugins_dir.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(&self.plugins_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                if !self.installed_info.contains_key(&name) {
                    let manifest_path = path.join("klyron-plugin.json");
                    let wasm_path = path.join(format!("{}.wasm", name));

                    if manifest_path.exists() && wasm_path.exists() {
                        if let Ok(content) = std::fs::read_to_string(&manifest_path) {
                            if let Ok(manifest) = serde_json::from_str::<PluginManifest>(&content) {
                                let wasm_bytes = std::fs::read(&wasm_path).unwrap_or_default();
                                let wasm_hash = hash_wasm(&wasm_bytes);

                                let info = PluginInfo {
                                    manifest,
                                    enabled: true,
                                    install_path: path.to_string_lossy().to_string(),
                                    installed_at: "unknown".to_string(),
                                    wasm_hash: hex::encode(&wasm_hash),
                                    size_bytes: wasm_bytes.len() as u64,
                                    compat: default_compat(),
                                };
                                self.installed_info.insert(name, info);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    // ── Sandbox Testing ─────────────────────────────────────────────────

    pub fn test_plugin(
        &mut self,
        name: &str,
        wasm_bytes: Vec<u8>,
        hooks_to_test: &[&str],
    ) -> Vec<manifest::SandboxTestReport> {
        let config = self
            .installed_info
            .get(name)
            .map(|i| i.manifest.sandbox.clone().unwrap_or_default())
            .unwrap_or_default();

        let mut harness = SandboxTestHarness::new(name, wasm_bytes, config);

        let mut reports = Vec::new();
        for hook_name in hooks_to_test {
            let report = harness.run_test(hook_name, |bytes| {
                let manifest = PluginManifest {
                    name: name.to_string(),
                    version: "0.1.0".to_string(),
                    description: None,
                    authors: None,
                    license: None,
                    klyron_api: Some(KLYRON_API_VERSION.to_string()),
                    permissions: Vec::new(),
                    dependencies: None,
                    hooks: Some(hooks_to_test.iter().map(|s| s.to_string()).collect()),
                    sandbox: None,
                };
                let (instance, mut store) = self.runtime.instantiate(bytes, &manifest)?;
                self.runtime.call_hook(&instance, &mut store, hook_name, &[])
            });
            reports.push(report);
        }

        reports
    }

    // ── Hot Reload ──────────────────────────────────────────────────────

    pub fn reload(&mut self, name: &str) -> Result<()> {
        let wasm_path = self
            .plugins
            .get(name)
            .map(|p| p.wasm_path.clone())
            .ok_or_else(|| anyhow::anyhow!("Plugin '{}' not found", name))?;

        let new_bytes = std::fs::read(&wasm_path)
            .with_context(|| format!("Failed to read WASM file: {}", wasm_path.display()))?;

        let new_hash = Sha256::digest(&new_bytes).to_vec();
        let old_hash = self.plugins.get(name).map(|p| &p.wasm_hash);

        if old_hash.map_or(false, |h| *h == new_hash) {
            return Ok(());
        }

        let manifest = self.parse_manifest(&wasm_path)?;
        self.unload(name)?;

        let (instance, store) = self.runtime.instantiate(&new_bytes, &manifest)?;

        let loaded = LoadedPlugin {
            manifest,
            instance,
            store,
            wasm_path,
            wasm_hash: new_hash,
            compat: default_compat(),
            enabled: true,
            load_time: Instant::now(),
        };

        self.register_hooks_for_plugin(&loaded);
        self.plugins.insert(name.to_string(), loaded);

        info!("Hot-reloaded plugin: {}", name);
        Ok(())
    }

    pub fn total_fuel_consumed(&self, name: &str) -> Result<u64> {
        let plugin = self
            .plugins
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Plugin '{}' not found", name))?;
        let initial: u64 = plugin
            .manifest
            .sandbox
            .as_ref()
            .and_then(|s| s.max_fuel)
            .unwrap_or(1_000_000);
        let remaining = plugin.store.get_fuel().unwrap_or(0);
        Ok(initial.saturating_sub(remaining))
    }

    pub fn hook_registry(&self) -> &HookRegistry {
        &self.hook_registry
    }

    pub fn plugins_dir(&self) -> &Path {
        &self.plugins_dir
    }

    pub fn runtime(&self) -> &PluginRuntime {
        &self.runtime
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new().expect("Failed to create default PluginRegistry")
    }
}

impl Drop for PluginRegistry {
    fn drop(&mut self) {
        let _ = self.rollback();
    }
}
