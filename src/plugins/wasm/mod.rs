pub mod runtime;
pub mod linker;
pub mod store;

use anyhow::{Context, Result};
use std::path::Path;
use wasmtime::{Engine, Module};
use crate::plugins::types::PluginConfig;

pub struct WasmPluginSandbox {
    engine: Engine,
}

impl WasmPluginSandbox {
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

    pub fn compile(&self, wasm_bytes: &[u8]) -> Result<Module> {
        Module::new(&self.engine, wasm_bytes).context("Failed to compile WASM module")
    }

    pub fn validate(&self, wasm_bytes: &[u8]) -> Result<()> {
        Module::new(&self.engine, wasm_bytes)?;
        Ok(())
    }

    pub fn instantiate(&self, _module: &Module, _config: &PluginConfig) -> Result<()> {
        Ok(())
    }
}

impl Default for WasmPluginSandbox {
    fn default() -> Self {
        Self::new().expect("Failed to create default WasmPluginSandbox")
    }
}
