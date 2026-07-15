use super::{Runtime, ServerlessFunction, ServerlessFunctionConfig};
use std::path::Path;
use std::collections::HashMap;

pub struct WasmRuntime {
    pub wasi: bool,
    pub memory_limit_mb: u32,
}

impl WasmRuntime {
    pub fn new() -> Self {
        Self {
            wasi: true,
            memory_limit_mb: 128,
        }
    }

    pub fn compile_wat(wat_source: &str) -> anyhow::Result<Vec<u8>> {
        let wat = wat::parse_str(wat_source)
            .map_err(|e| anyhow::anyhow!("WAT parse error: {e}"))?;
        Ok(wat)
    }

    pub fn generate_rust_wasm_template(name: &str) -> String {
        format!(
            r#"// Klyron WASM Serverless Function: {name}
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn handle(request_json: &str) -> String {{
    let response = serde_json::json!({{
        "ok": true,
        "function": "{name}",
        "message": "Hello from WASM!",
        "ts": chrono::Utc::now().to_rfc3339(),
    }});
    response.to_string()
}}
"#,
            name = name,
        )
    }

    pub fn generate_assemblyscript_template(name: &str) -> String {
        format!(
            r#"// Klyron WASM Serverless Function: {name}
export function handle(request: string): string {{
    const response = {{
        ok: true,
        function: "{name}",
        message: "Hello from WASM!",
        timestamp: Date.now(),
    }};
    return JSON.stringify(response);
}}
"#,
            name = name,
        )
    }
}

impl Default for WasmRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerlessFunction for WasmRuntime {
    fn validate(&self, config: &ServerlessFunctionConfig) -> anyhow::Result<()> {
        if config.runtime != Runtime::Wasm {
            anyhow::bail!("WasmRuntime only supports Runtime::Wasm");
        }
        if config.handler.is_empty() {
            anyhow::bail!("handler is required for WASM functions");
        }
        Ok(())
    }

    fn generate_handler(&self, config: &ServerlessFunctionConfig, output_dir: &Path) -> anyhow::Result<std::path::PathBuf> {
        let content = Self::generate_rust_wasm_template(&config.name);
        let handler_path = output_dir.join(format!("{}.rs", config.name));
        std::fs::create_dir_all(output_dir)?;
        std::fs::write(&handler_path, &content)?;
        Ok(handler_path)
    }

    fn generate_metadata(&self, config: &ServerlessFunctionConfig) -> anyhow::Result<serde_json::Value> {
        Ok(serde_json::json!({
            "type": "wasm",
            "name": config.name,
            "memory_mb": self.memory_limit_mb,
            "wasi": self.wasi,
            "handler": config.handler,
        }))
    }

    fn bundle(&self, config: &ServerlessFunctionConfig, output_dir: &Path) -> anyhow::Result<std::path::PathBuf> {
        let wasm_path = output_dir.join(format!("{}.wasm", config.name));
        let source_path = self.generate_handler(config, output_dir)?;
        let wasm_bytes = Self::compile_wat(&std::fs::read_to_string(&source_path)?)?;
        std::fs::write(&wasm_path, &wasm_bytes)?;
        Ok(wasm_path)
    }

    fn invoke(&self, config: &ServerlessFunctionConfig, payload: &str) -> anyhow::Result<String> {
        let wasm_bytes = std::fs::read(config.handler.as_str())
            .map_err(|e| anyhow::anyhow!("Cannot read WASM module: {e}"))?;

        let mut config_wasm = wasmtime::Config::new();
        config_wasm.wasm_component_model(true);
        if self.wasi {
            config_wasm.wasm_backtrace(true);
        }
        let engine = wasmtime::Engine::new(&config_wasm)?;
        let module = wasmtime::Module::new(&engine, &wasm_bytes)?;
        let mut store = wasmtime::Store::new(&engine, ());
        let linker = wasmtime::Linker::new(&engine);
        let instance = linker.instantiate(&mut store, &module)?;
        let handle = instance.get_typed_func::<(&str,), (String,), _>(&mut store, "handle")?;
        let result = handle.call(&mut store, payload)?;
        Ok(result)
    }
}
