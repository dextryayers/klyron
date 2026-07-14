use anyhow::{Context, Result};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::info;
use wasmtime::{Engine, Instance, Linker, Module, Store, ResourceLimiter};
use wasmtime_wasi::preview1::{self, WasiP1Ctx};
use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxBuilder, WasiView, DirPerms, FilePerms};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub authors: Option<Vec<String>>,
    pub license: Option<String>,
    pub permissions: Vec<String>,
    pub dependencies: Option<Vec<PluginDependency>>,
    pub hooks: Option<PluginHooks>,
    pub sandbox: Option<SandboxConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    pub name: String,
    pub version: String,
    pub optional: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginHooks {
    pub on_load: Option<String>,
    pub on_init: Option<String>,
    pub on_start: Option<String>,
    pub on_stop: Option<String>,
    pub on_destroy: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub max_memory_bytes: Option<u64>,
    pub max_fuel: Option<u64>,
    pub allowed_domains: Option<Vec<String>>,
    pub allowed_paths: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub enum PluginEvent {
    Load { name: String, version: String },
    Unload { name: String },
    Init { name: String },
    Start { name: String },
    Stop { name: String },
    Destroy { name: String },
    Call { name: String, method: String },
    HotReload { name: String, new_hash: String },
    Error { name: String, error: String },
}

type EventCallback = Arc<dyn Fn(PluginEvent) + Send + Sync + 'static>;

struct MemoryLimiter {
    max_memory_bytes: u64,
}

impl ResourceLimiter for MemoryLimiter {
    fn memory_growing(&mut self, current: usize, desired: usize, _maximum: Option<usize>) -> Result<bool> {
        Ok((desired as u64) <= self.max_memory_bytes || (current as u64) <= self.max_memory_bytes)
    }

    fn table_growing(&mut self, _current: usize, _desired: usize, _maximum: Option<usize>) -> Result<bool> {
        Ok(true)
    }
}

struct PluginCtx {
    table: ResourceTable,
    wasi: WasiP1Ctx,
    limiter: Option<MemoryLimiter>,
}

impl WasiView for PluginCtx {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }

    fn ctx(&mut self) -> &mut WasiCtx {
        self.wasi.ctx()
    }
}

impl ResourceLimiter for PluginCtx {
    fn memory_growing(&mut self, current: usize, desired: usize, maximum: Option<usize>) -> Result<bool> {
        if let Some(ref mut limiter) = self.limiter {
            limiter.memory_growing(current, desired, maximum)
        } else {
            Ok(true)
        }
    }

    fn table_growing(&mut self, current: usize, desired: usize, maximum: Option<usize>) -> Result<bool> {
        if let Some(ref mut limiter) = self.limiter {
            limiter.table_growing(current, desired, maximum)
        } else {
            Ok(true)
        }
    }
}

struct PluginInstance {
    manifest: PluginManifest,
    instance: Instance,
    store: Store<PluginCtx>,
    wasm_path: PathBuf,
    wasm_hash: Vec<u8>,
    initialized: AtomicBool,
    started: AtomicBool,
}

impl PluginInstance {
    fn call_hook(&mut self, name: &str) -> Result<()> {
        let hook_name = match name {
            "on_load" => self.manifest.hooks.as_ref().and_then(|h| h.on_load.as_deref()),
            "on_init" => self.manifest.hooks.as_ref().and_then(|h| h.on_init.as_deref()),
            "on_start" => self.manifest.hooks.as_ref().and_then(|h| h.on_start.as_deref()),
            "on_stop" => self.manifest.hooks.as_ref().and_then(|h| h.on_stop.as_deref()),
            "on_destroy" => self.manifest.hooks.as_ref().and_then(|h| h.on_destroy.as_deref()),
            _ => None,
        };

        if let Some(hook) = hook_name {
            if let Some(func) = self.instance.get_func(&mut self.store, hook) {
                let typed = func.typed::<(), ()>(&self.store)
                    .map_err(|e| anyhow::anyhow!("Failed to type hook '{}': {}", hook, e))?;
                typed.call(&mut self.store, ())
                    .map_err(|e| anyhow::anyhow!("Hook '{}' failed: {}", hook, e))?;
            }
        }
        Ok(())
    }
}

pub struct PluginManager {
    plugins: HashMap<String, PluginInstance>,
    engine: Engine,
    event_listeners: Vec<EventCallback>,
    _watcher: Option<RecommendedWatcher>,
    dependencies: HashMap<String, Vec<String>>,
}

impl PluginManager {
    pub fn new() -> Result<Self> {
        let mut config = wasmtime::Config::new();
        config.wasm_multi_value(true);
        config.wasm_component_model(false);
        config.consume_fuel(true);
        let engine = Engine::new(&config).context("Failed to create WASM engine")?;

        Ok(Self {
            plugins: HashMap::new(),
            engine,
            event_listeners: Vec::new(),
            _watcher: None,
            dependencies: HashMap::new(),
        })
    }

    pub fn new_with_limits(_max_fuel: u64, _max_memory: u64) -> Result<Self> {
        let mut config = wasmtime::Config::new();
        config.wasm_multi_value(true);
        config.wasm_component_model(false);
        config.consume_fuel(true);
        config.max_wasm_stack(512 * 1024);
        let engine = Engine::new(&config).context("Failed to create WASM engine")?;

        Ok(Self {
            plugins: HashMap::new(),
            engine,
            event_listeners: Vec::new(),
            _watcher: None,
            dependencies: HashMap::new(),
        })
    }

    pub fn on_event<F>(&mut self, callback: F)
    where
        F: Fn(PluginEvent) + Send + Sync + 'static,
    {
        self.event_listeners.push(Arc::new(callback));
    }

    fn emit(&self, event: PluginEvent) {
        for listener in &self.event_listeners {
            listener(event.clone());
        }
    }

    pub fn load(&mut self, path: &str) -> Result<String> {
        let wasm_bytes = std::fs::read(path)
            .with_context(|| format!("Failed to read WASM file: {}", path))?;

        let hash = Sha256::digest(&wasm_bytes).to_vec();

        let module =
            Module::new(&self.engine, &wasm_bytes).context("Failed to compile WASM module")?;

        let manifest = Self::parse_manifest(path)?;

        let sandbox = manifest.sandbox.clone().unwrap_or_default();
        let mut builder = WasiCtxBuilder::new();

        let max_fuel = sandbox.max_fuel.unwrap_or(1_000_000);

        if let Some(paths) = &sandbox.allowed_paths {
            for p in paths {
                let _ = builder.preopened_dir(
                    Path::new(p),
                    p,
                    DirPerms::all(),
                    FilePerms::all(),
                );
            }
        }

        for perm in &manifest.permissions {
            match perm.as_str() {
                "stdio" => {
                    builder.inherit_stdout().inherit_stderr();
                }
                "net" => {
                    builder.inherit_stdio();
                }
                "env" => {
                    builder.inherit_env();
                }
                "fs_read" | "fs_write" | "fs_all" => {
                    builder.inherit_stdio();
                }
                _ => {}
            }
        }

        let wasi_p1 = builder.build_p1();
        let table = ResourceTable::new();

        let limiter = sandbox.max_memory_bytes.map(|max_mem| MemoryLimiter { max_memory_bytes: max_mem });

        let mut store = Store::new(
            &self.engine,
            PluginCtx {
                table,
                wasi: wasi_p1,
                limiter,
            },
        );

        store.set_fuel(max_fuel)?;

        let mut linker: Linker<PluginCtx> = Linker::new(&self.engine);
        preview1::add_to_linker_sync(&mut linker, |ctx| &mut ctx.wasi)
            .context("Failed to add WASI to linker")?;

        let instance = linker
            .instantiate(&mut store, &module)
            .context("Failed to instantiate WASM module")?;

        let name = manifest.name.clone();

        let mut plugin = PluginInstance {
            manifest,
            instance,
            store,
            wasm_path: PathBuf::from(path),
            wasm_hash: hash,
            initialized: AtomicBool::new(false),
            started: AtomicBool::new(false),
        };

        plugin.call_hook("on_load")?;

        self.plugins.insert(name.clone(), plugin);

        if let Some(deps) = self.plugins.get(&name).and_then(|p| p.manifest.dependencies.clone()) {
            let dep_names: Vec<String> = deps.into_iter().map(|d| d.name).collect();
            self.dependencies.insert(name.clone(), dep_names);
        }

        self.emit(PluginEvent::Load {
            name: name.clone(),
            version: "0.1.0".to_string(),
        });

        info!("Loaded plugin: {}", name);
        Ok(name)
    }

    pub fn init(&mut self, name: &str) -> Result<()> {
        let plugin = self.plugins.get_mut(name)
            .ok_or_else(|| anyhow::anyhow!("Plugin '{}' not found", name))?;

        plugin.call_hook("on_init")?;
        plugin.initialized.store(true, Ordering::SeqCst);

        self.emit(PluginEvent::Init {
            name: name.to_string(),
        });

        Ok(())
    }

    pub fn start(&mut self, name: &str) -> Result<()> {
        let plugin = self.plugins.get_mut(name)
            .ok_or_else(|| anyhow::anyhow!("Plugin '{}' not found", name))?;

        plugin.call_hook("on_start")?;
        plugin.started.store(true, Ordering::SeqCst);

        self.emit(PluginEvent::Start {
            name: name.to_string(),
        });

        Ok(())
    }

    pub fn stop(&mut self, name: &str) -> Result<()> {
        let plugin = self.plugins.get_mut(name)
            .ok_or_else(|| anyhow::anyhow!("Plugin '{}' not found", name))?;

        plugin.call_hook("on_stop")?;
        plugin.started.store(false, Ordering::SeqCst);

        self.emit(PluginEvent::Stop {
            name: name.to_string(),
        });

        Ok(())
    }

    pub fn unload(&mut self, name: &str) -> Result<()> {
        let mut plugin = self.plugins.remove(name)
            .ok_or_else(|| anyhow::anyhow!("Plugin '{}' not found", name))?;

        plugin.call_hook("on_destroy")?;

        self.emit(PluginEvent::Destroy {
            name: name.to_string(),
        });

        self.dependencies.remove(name);
        info!("Unloaded plugin: {}", name);
        Ok(())
    }

    pub fn resolve_dependencies(&self, name: &str) -> Result<Vec<String>> {
        let mut resolved = Vec::new();
        let mut visited = std::collections::HashSet::new();
        self.resolve_deps_recursive(name, &mut resolved, &mut visited)?;
        Ok(resolved)
    }

    fn resolve_deps_recursive(
        &self,
        name: &str,
        resolved: &mut Vec<String>,
        visited: &mut std::collections::HashSet<String>,
    ) -> Result<()> {
        if !visited.insert(name.to_string()) {
            return Ok(());
        }

        if let Some(deps) = self.dependencies.get(name) {
            for dep in deps {
                self.resolve_deps_recursive(dep, resolved, visited)?;
            }
        }

        resolved.push(name.to_string());
        Ok(())
    }

    #[inline]
    pub fn call(&mut self, name: &str, method: &str, args: &[u8]) -> Result<Vec<u8>> {
        let plugin = self
            .plugins
            .get_mut(name)
            .ok_or_else(|| anyhow::anyhow!("Plugin '{}' not found", name))?;

        let fuel_before = plugin.store.get_fuel().unwrap_or(0);
        if fuel_before == 0 {
            anyhow::bail!("Plugin '{}' has exhausted its fuel allocation", name);
        }

        let func = plugin
            .instance
            .get_func(&mut plugin.store, method)
            .ok_or_else(|| {
                anyhow::anyhow!("Method '{}' not found in plugin '{}'", method, name)
            })?;

        let func_typed = func
            .typed::<(i32, i32), i32>(&plugin.store)
            .map_err(|e| anyhow::anyhow!("Failed to type function: {}", e))?;

        let memory = plugin
            .instance
            .get_memory(&mut plugin.store, "memory")
            .ok_or_else(|| anyhow::anyhow!("No memory export in plugin '{}'", name))?;

        if !args.is_empty() {
            let data = memory.data_mut(&mut plugin.store);
            let end = args.len().min(data.len());
            data[..end].copy_from_slice(&args[..end]);
        }

        let result_ptr = func_typed
            .call(&mut plugin.store, (0, args.len() as i32))
            .map_err(|e| anyhow::anyhow!("Plugin call failed: {}", e))?;

        let data = memory.data(&plugin.store);
        let result_len_pos = result_ptr as usize;
        let result_len = if result_len_pos + 4 <= data.len() {
            let mut len_bytes = [0u8; 4];
            len_bytes.copy_from_slice(&data[result_len_pos..result_len_pos + 4]);
            i32::from_le_bytes(len_bytes) as usize
        } else {
            0
        };

        let mut result = Vec::new();
        if result_len > 0 {
            let start = (result_ptr + 4) as usize;
            let end = (start + result_len).min(data.len());
            result.extend_from_slice(&data[start..end]);
        }

        self.emit(PluginEvent::Call {
            name: name.to_string(),
            method: method.to_string(),
        });

        Ok(result)
    }

    pub fn list(&self) -> Vec<&str> {
        self.plugins.keys().map(|s| s.as_str()).collect()
    }

    pub fn get_manifest(&self, name: &str) -> Option<&PluginManifest> {
        self.plugins.get(name).map(|p| &p.manifest)
    }

    pub fn reload(&mut self, name: &str) -> Result<()> {
        let wasm_path = self.plugins.get(name)
            .map(|p| p.wasm_path.clone())
            .ok_or_else(|| anyhow::anyhow!("Plugin '{}' not found", name))?;

        let new_bytes = std::fs::read(&wasm_path)
            .with_context(|| format!("Failed to read WASM file: {}", wasm_path.display()))?;

        let new_hash = Sha256::digest(&new_bytes).to_vec();
        let old_hash = self.plugins.get(name).map(|p| &p.wasm_hash);

        if old_hash.map_or(false, |h| *h == new_hash) {
            return Ok(());
        }

        let old_manifest = self.plugins.get(name).map(|p| p.manifest.name.clone())
            .unwrap_or_default();

        self.unload(name)?;
        let new_name = self.load(wasm_path.to_str().unwrap())?;

        self.emit(PluginEvent::HotReload {
            name: new_name.clone(),
            new_hash: hex::encode(&new_hash),
        });

        info!("Hot-reloaded plugin: {} (was: {})", new_name, old_manifest);
        Ok(())
    }

    fn parse_manifest(path: &str) -> Result<PluginManifest> {
        let wasm_dir = Path::new(path).parent().unwrap_or(Path::new("."));

        for manifest_name in &["klyron-plugin.toml", "plugin.toml"] {
            let manifest_path = wasm_dir.join(manifest_name);
            if manifest_path.exists() {
                let content = std::fs::read_to_string(&manifest_path)?;
                let manifest: PluginManifest = toml::from_str(&content)?;
                return Ok(manifest);
            }
        }

        Ok(PluginManifest {
            name: Path::new(path)
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            version: "0.1.0".to_string(),
            description: None,
            authors: None,
            license: None,
            permissions: Vec::new(),
            dependencies: None,
            hooks: None,
            sandbox: None,
        })
    }

    pub fn watch_hot_reload(&mut self, dir: &str) -> Result<()> {
        let path = PathBuf::from(dir);
        let _manager_ref = std::sync::Mutex::new(std::ptr::null_mut::<PluginManager>());

        let mut watcher: RecommendedWatcher =
            notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                        for wasm_path in event.paths.iter().filter(|p| {
                            p.extension().map_or(false, |ext| ext == "wasm")
                        }) {
                            if let Some(path_str) = wasm_path.to_str() {
                                info!("Hot-reload detected: {}", path_str);
                            }
                        }
                    }
                }
            })?;

        watcher.watch(&path, RecursiveMode::Recursive)?;

        self._watcher = Some(watcher);
        info!("Watching directory for hot-reload: {}", dir);

        Ok(())
    }

    pub fn watch_hot_reload_with_manager(&mut self, dir: &str) -> Result<()> {
        let path = PathBuf::from(dir);
        let plugins_dir = path.clone();
        let plugin_names: Vec<String> = self.plugins.keys().cloned().collect();

        let mut watcher: RecommendedWatcher =
            notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    if matches!(event.kind, EventKind::Modify(_)) {
                        for changed_path in event.paths.iter().filter(|p| {
                            p.extension().map_or(false, |ext| ext == "wasm")
                        }) {
                            if let Some(name) = changed_path.file_stem()
                                .and_then(|s| s.to_str())
                                .map(|s| s.to_string())
                            {
                                if plugin_names.contains(&name) {
                                    info!("Hot-reload triggered for plugin: {}", name);
                                }
                            }
                        }
                    }
                }
            })?;

        watcher.watch(&plugins_dir, RecursiveMode::Recursive)?;
        self._watcher = Some(watcher);
        info!("Watching directory for plugin hot-reload: {}", dir);
        Ok(())
    }

    pub fn total_fuel_consumed(&self, name: &str) -> Result<u64> {
        let plugin = self.plugins.get(name)
            .ok_or_else(|| anyhow::anyhow!("Plugin '{}' not found", name))?;
        let initial: u64 = plugin.manifest.sandbox.as_ref()
            .and_then(|s| s.max_fuel)
            .unwrap_or(1_000_000);
        let remaining = plugin.store.get_fuel().unwrap_or(0);
        Ok(initial.saturating_sub(remaining))
    }
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            max_memory_bytes: Some(64 * 1024 * 1024),
            max_fuel: Some(1_000_000),
            allowed_domains: None,
            allowed_paths: None,
        }
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new().expect("Failed to create default PluginManager")
    }
}
