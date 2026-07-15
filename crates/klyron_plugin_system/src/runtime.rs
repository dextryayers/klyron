use crate::manifest::{PluginManifest, KLYRON_API_VERSION};
use crate::sandbox::PluginSandbox;
use crate::verify_compatibility;
use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tracing::warn;
use wasmtime::{Engine, Instance, Linker, Module, Store};
use wasmtime_wasi::preview1::{self, WasiP1Ctx};
use wasmtime_wasi::{DirPerms, FilePerms, ResourceTable, WasiCtxBuilder, WasiView};

pub struct PluginRuntime {
    engine: Engine,
    sandbox: Arc<PluginSandbox>,
}

#[derive(Clone)]
struct RuntimeCtx {
    table: ResourceTable,
    wasi: WasiP1Ctx,
}

impl WasiView for RuntimeCtx {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
    fn ctx(&mut self) -> &mut wasmtime_wasi::WasiCtx {
        self.wasi.ctx()
    }
}

impl PluginRuntime {
    pub fn new(sandbox: Arc<PluginSandbox>) -> Result<Self> {
        let mut config = wasmtime::Config::new();
        config.wasm_multi_value(true);
        config.wasm_component_model(false);
        config.consume_fuel(true);
        let engine = Engine::new(&config).context("Failed to create WASM engine")?;
        Ok(Self { engine, sandbox })
    }

    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    pub fn instantiate(
        &self,
        wasm_bytes: &[u8],
        manifest: &PluginManifest,
    ) -> Result<(Instance, Store<RuntimeCtx>)> {
        let module = Module::new(&self.engine, wasm_bytes)
            .context("Failed to compile WASM module")?;

        let mut builder = WasiCtxBuilder::new();

        if let Some(ref sandbox_cfg) = manifest.sandbox {
            if let Some(ref paths) = sandbox_cfg.allowed_paths {
                for p in paths {
                    let _ = builder.preopened_dir(
                        Path::new(p), p,
                        DirPerms::all(), FilePerms::all(),
                    );
                }
            }
        }

        let limits = self.sandbox.limits();
        if let Some(ref allowed_env) = limits.allowed_env {
            for var in allowed_env {
                if let Ok(val) = std::env::var(var) {
                    builder.env(var, &val);
                }
            }
        }

        for perm in &manifest.permissions {
            match perm.as_str() {
                "stdio" => { builder.inherit_stdout().inherit_stderr(); }
                "net" | "net_connect" | "net_listen" => { builder.inherit_stdio(); }
                "env" | "env_read" => { builder.inherit_env(); }
                "fs_read" | "fs_write" | "fs_all" => { builder.inherit_stdio(); }
                "all" => { builder.inherit_stdio().inherit_env(); }
                _ => {}
            }
        }

        let wasi_p1 = builder.build_p1();
        let table = ResourceTable::new();
        let mut store = Store::new(&self.engine, RuntimeCtx { table, wasi: wasi_p1 });

        let mut linker: Linker<RuntimeCtx> = Linker::new(&self.engine);
        preview1::add_to_linker_sync(&mut linker, |ctx| &mut ctx.wasi)
            .context("Failed to add WASI to linker")?;

        self.sandbox.start_execution();
        let instance = linker
            .instantiate(&mut store, &module)
            .context("Failed to instantiate WASM module")?;

        Ok((instance, store))
    }

    pub fn call_hook(
        &self,
        instance: &Instance,
        store: &mut Store<RuntimeCtx>,
        hook_name: &str,
        context: &[u8],
    ) -> Result<Vec<u8>> {
        if self.sandbox.is_timed_out() {
            anyhow::bail!("Sandbox execution timed out");
        }

        let func = instance
            .get_func(store, hook_name)
            .ok_or_else(|| anyhow::anyhow!("Hook '{}' not exported", hook_name))?;

        let memory = instance
            .get_memory(store, "memory")
            .ok_or_else(|| anyhow::anyhow!("No memory export"))?;

        if !context.is_empty() {
            let data = memory.data_mut(store);
            let end = context.len().min(data.len());
            data[..end].copy_from_slice(&context[..end]);
        }

        let typed = func
            .typed::<(i32, i32), i32>(store)
            .map_err(|e| anyhow::anyhow!("Failed to type hook: {}", e))?;

        let result_ptr = typed
            .call(store, (0, context.len() as i32))
            .map_err(|e| anyhow::anyhow!("Hook call failed: {}", e))?;

        let data = memory.data(&store);
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

        self.sandbox.consume_fuel(1);
        Ok(result)
    }

    pub fn call_function(
        &self,
        instance: &Instance,
        store: &mut Store<RuntimeCtx>,
        func_name: &str,
        args: &[u8],
    ) -> Result<Vec<u8>> {
        if self.sandbox.is_timed_out() {
            anyhow::bail!("Sandbox execution timed out");
        }

        let func = instance
            .get_func(store, func_name)
            .ok_or_else(|| anyhow::anyhow!("Function '{}' not found", func_name))?;

        let memory = instance
            .get_memory(store, "memory")
            .ok_or_else(|| anyhow::anyhow!("No memory export"))?;

        if !args.is_empty() {
            let data = memory.data_mut(store);
            let end = args.len().min(data.len());
            data[..end].copy_from_slice(&args[..end]);
        }

        let typed = func
            .typed::<(i32, i32), i32>(store)
            .map_err(|e| anyhow::anyhow!("Failed to type function: {}", e))?;

        let result_ptr = typed
            .call(store, (0, args.len() as i32))
            .map_err(|e| anyhow::anyhow!("Function call failed: {}", e))?;

        let data = memory.data(&store);
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

        self.sandbox.consume_fuel(1);
        Ok(result)
    }

    pub fn check_compatibility(&self, manifest: &PluginManifest, force: bool) -> Result<()> {
        let api = manifest.klyron_api.clone().unwrap_or_else(|| KLYRON_API_VERSION.to_string());
        verify_compatibility(KLYRON_API_VERSION, &api, force)?;
        Ok(())
    }
}

impl Default for PluginRuntime {
    fn default() -> Self {
        let sandbox = Arc::new(PluginSandbox::with_defaults());
        Self::new(sandbox).expect("Failed to create default PluginRuntime")
    }
}

pub struct LoadedPlugin {
    pub manifest: PluginManifest,
    pub instance: Instance,
    pub store: Store<RuntimeCtx>,
    pub wasm_path: PathBuf,
    pub wasm_hash: Vec<u8>,
    pub enabled: bool,
    pub load_time: Instant,
}

pub fn hash_wasm(bytes: &[u8]) -> Vec<u8> {
    Sha256::digest(bytes).to_vec()
}

pub fn check_api_compatibility(manifest: &PluginManifest) -> String {
    manifest.klyron_api.clone().unwrap_or_else(|| KLYRON_API_VERSION.to_string())
}
