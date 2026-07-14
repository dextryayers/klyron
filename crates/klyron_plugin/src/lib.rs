use anyhow::{Context, Result};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::info;
use wasmtime::{Engine, Instance, Linker, Module, Store};
use wasmtime_wasi::preview1::{self, WasiP1Ctx};
use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxBuilder, WasiView};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum PluginEvent {
    Load { name: String, version: String },
    Unload { name: String },
    Call { name: String, method: String },
}

type EventCallback = Arc<dyn Fn(PluginEvent) + Send + Sync + 'static>;

struct PluginCtx {
    table: ResourceTable,
    wasi: WasiP1Ctx,
}

impl WasiView for PluginCtx {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }

    fn ctx(&mut self) -> &mut WasiCtx {
        self.wasi.ctx()
    }
}

struct PluginInstance {
    manifest: PluginManifest,
    instance: Instance,
    store: Store<PluginCtx>,
    _wasm_hash: Vec<u8>,
    _watcher_path: Option<PathBuf>,
}

pub struct PluginManager {
    plugins: HashMap<String, PluginInstance>,
    engine: Engine,
    event_listeners: Vec<EventCallback>,
    _watcher: Option<RecommendedWatcher>,
}

impl PluginManager {
    pub fn new() -> Result<Self> {
        let mut config = wasmtime::Config::new();
        config.wasm_multi_value(true);
        config.wasm_component_model(false);
        let engine = Engine::new(&config).context("Failed to create WASM engine")?;

        Ok(Self {
            plugins: HashMap::new(),
            engine,
            event_listeners: Vec::new(),
            _watcher: None,
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

        let mut builder = WasiCtxBuilder::new();
        for perm in &manifest.permissions {
            match perm.as_str() {
                "fs_read" => {
                    builder.inherit_stdout();
                }
                "fs_write" => {
                    builder.inherit_stderr();
                }
                "net" => {
                    builder.inherit_stdio();
                }
                "env" => {
                    builder.inherit_env();
                }
                _ => {}
            }
        }

        let wasi_p1 = builder.build_p1();
        let table = ResourceTable::new();

        let mut store = Store::new(
            &self.engine,
            PluginCtx {
                table,
                wasi: wasi_p1,
            },
        );

        let mut linker: Linker<PluginCtx> = Linker::new(&self.engine);
        preview1::add_to_linker_sync(&mut linker, |ctx| &mut ctx.wasi)
            .context("Failed to add WASI to linker")?;

        let instance = linker
            .instantiate(&mut store, &module)
            .context("Failed to instantiate WASM module")?;

        let name = manifest.name.clone();

        self.plugins.insert(
            name.clone(),
            PluginInstance {
                manifest,
                instance,
                store,
                _wasm_hash: hash,
                _watcher_path: Some(PathBuf::from(path)),
            },
        );

        self.emit(PluginEvent::Load {
            name: name.clone(),
            version: "0.1.0".to_string(),
        });

        info!("Loaded plugin: {}", name);
        Ok(name)
    }

    pub fn unload(&mut self, name: &str) -> Result<()> {
        self.plugins
            .remove(name)
            .ok_or_else(|| anyhow::anyhow!("Plugin '{}' not found", name))?;

        self.emit(PluginEvent::Unload {
            name: name.to_string(),
        });

        info!("Unloaded plugin: {}", name);
        Ok(())
    }

    #[inline]
    pub fn call(&mut self, name: &str, method: &str, args: &[u8]) -> Result<Vec<u8>> {
        let plugin = self
            .plugins
            .get_mut(name)
            .ok_or_else(|| anyhow::anyhow!("Plugin '{}' not found", name))?;

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

    fn parse_manifest(path: &str) -> Result<PluginManifest> {
        let wasm_dir = Path::new(path).parent().unwrap_or(Path::new("."));
        let manifest_path = wasm_dir.join("plugin.toml");

        if manifest_path.exists() {
            let content = std::fs::read_to_string(&manifest_path)?;
            let manifest: PluginManifest = toml::from_str(&content)?;
            return Ok(manifest);
        }

        Ok(PluginManifest {
            name: Path::new(path)
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            version: "0.1.0".to_string(),
            permissions: Vec::new(),
        })
    }

    pub fn watch_hot_reload(&mut self, dir: &str) -> Result<()> {
        let path = PathBuf::from(dir);

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
}
