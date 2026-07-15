use anyhow::{Context, Result};
use wasmtime::{Engine, Linker, Module, Store};
use wasmtime_wasi::preview1::{self, WasiP1Ctx};
use wasmtime_wasi::{ResourceTable, WasiCtxBuilder, WasiView};
use std::path::Path;
use crate::manifest::PluginManifest;

pub struct WasmRuntime {
    engine: Engine,
}

impl WasmRuntime {
    pub fn new() -> Result<Self> {
        let mut config = wasmtime::Config::new();
        config.wasm_multi_value(true);
        config.consume_fuel(true);
        let engine = Engine::new(&config).context("Failed to create WASM engine")?;
        Ok(Self { engine })
    }

    pub fn engine(&self) -> &Engine {
        &self.engine
    }
}

impl Default for WasmRuntime {
    fn default() -> Self {
        Self::new().expect("Failed to create default WasmRuntime")
    }
}

pub struct WasmModule {
    module: Module,
    engine: Engine,
}

impl WasmModule {
    pub fn new(engine: &Engine, wasm_bytes: &[u8]) -> Result<Self> {
        let module = Module::new(engine, wasm_bytes)
            .context("Failed to compile WASM module")?;
        Ok(Self {
            module,
            engine: engine.clone(),
        })
    }

    pub fn module(&self) -> &Module {
        &self.module
    }
}

pub struct WasmInstance {
    instance: wasmtime::Instance,
    store: Store<WasmStoreData>,
}

struct WasmStoreData {
    table: ResourceTable,
    wasi: WasiP1Ctx,
}

impl WasiView for WasmStoreData {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
    fn ctx(&mut self) -> &mut wasmtime_wasi::WasiCtx {
        self.wasi.ctx()
    }
}

impl WasmInstance {
    pub fn new(
        engine: &Engine,
        module: &WasmModule,
        manifest: &PluginManifest,
        allowed_paths: &[String],
    ) -> Result<Self> {
        let mut builder = WasiCtxBuilder::new();
        for p in allowed_paths {
            let _ = builder.preopened_dir(
                Path::new(p), p,
                wasmtime_wasi::DirPerms::all(),
                wasmtime_wasi::FilePerms::all(),
            );
        }

        for perm in &manifest.permissions {
            match perm.as_str() {
                "stdio" => { builder.inherit_stdout().inherit_stderr(); }
                "env" | "env_read" => { builder.inherit_env(); }
                _ => {}
            }
        }

        let wasi_p1 = builder.build_p1();
        let table = ResourceTable::new();
        let store = Store::new(engine, WasmStoreData { table, wasi: wasi_p1 });

        let mut linker: Linker<WasmStoreData> = Linker::new(engine);
        preview1::add_to_linker_sync(&mut linker, |ctx| &mut ctx.wasi)
            .context("Failed to add WASI to linker")?;

        let instance = linker
            .instantiate(&mut store, &module.module)
            .context("Failed to instantiate WASM module")?;

        Ok(Self { instance, store })
    }

    pub fn instance(&self) -> &wasmtime::Instance {
        &self.instance
    }

    pub fn store(&mut self) -> &mut Store<WasmStoreData> {
        &mut self.store
    }

    pub fn call_function(&mut self, name: &str, args: &[u8]) -> Result<Vec<u8>> {
        let func = self.instance
            .get_func(&mut self.store, name)
            .ok_or_else(|| anyhow::anyhow!("Function '{}' not exported", name))?;

        let memory = self.instance
            .get_memory(&mut self.store, "memory")
            .ok_or_else(|| anyhow::anyhow!("No memory export"))?;

        if !args.is_empty() {
            let data = memory.data_mut(&mut self.store);
            let end = args.len().min(data.len());
            data[..end].copy_from_slice(&args[..end]);
        }

        let typed = func
            .typed::<(i32, i32), i32>(&self.store)
            .map_err(|e| anyhow::anyhow!("Failed to type function: {}", e))?;

        let result_ptr = typed
            .call(&mut self.store, (0, args.len() as i32))
            .map_err(|e| anyhow::anyhow!("Function call failed: {}", e))?;

        let data = memory.data(&self.store);
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

        Ok(result)
    }
}

pub fn compile_wasm(engine: &Engine, bytes: &[u8]) -> Result<Module> {
    Module::new(engine, bytes).context("Failed to compile WASM")
}

pub fn validate_wasm(bytes: &[u8]) -> Result<()> {
    let mut config = wasmtime::Config::new();
    config.wasm_multi_value(true);
    let engine = Engine::new(&config)?;
    Module::new(&engine, bytes)?;
    Ok(())
}
