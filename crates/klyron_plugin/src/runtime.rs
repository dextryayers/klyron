use crate::manifest::{PluginCompat, PluginManifest, KLYRON_API_VERSION};
use crate::sandbox::Sandbox;
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
}

pub struct RuntimeCtx {
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
    pub fn new(sandbox: Option<Arc<Sandbox>>) -> Result<Self> {
        let mut config = wasmtime::Config::new();
        config.wasm_multi_value(true);
        config.wasm_component_model(false);
        config.consume_fuel(true);

        if let Some(ref s) = sandbox {
            config.max_wasm_stack(s.limits.max_cpu_ms as usize * 4096);
        }

        let engine = Engine::new(&config).context("Failed to create WASM engine")?;
        Ok(Self { engine })
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
                        Path::new(p),
                        p,
                        DirPerms::all(),
                        FilePerms::all(),
                    );
                }
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

        let mut store = Store::new(
            &self.engine,
            RuntimeCtx {
                table,
                wasi: wasi_p1,
            },
        );

        let mut linker: Linker<RuntimeCtx> = Linker::new(&self.engine);
        preview1::add_to_linker_sync(&mut linker, |ctx| &mut ctx.wasi)
            .context("Failed to add WASI to linker")?;

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
        let func = instance
            .get_func(&mut *store, hook_name)
            .ok_or_else(|| anyhow::anyhow!("Hook '{}' not exported by plugin", hook_name))?;

        let memory = instance
            .get_memory(&mut *store, "memory")
            .ok_or_else(|| anyhow::anyhow!("Plugin does not export 'memory'"))?;

        if !context.is_empty() {
            let data = memory.data_mut(&mut *store);
            let end = context.len().min(data.len());
            data[..end].copy_from_slice(&context[..end]);
        }

        let typed = func
            .typed::<(i32, i32), i32>(&mut *store)
            .map_err(|e| anyhow::anyhow!("Failed to type hook function: {}", e))?;

        let result_ptr = typed
            .call(&mut *store, (0, context.len() as i32))
            .map_err(|e| anyhow::anyhow!("Hook '{}' call failed: {}", hook_name, e))?;

        let data = memory.data(&*store);
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

    #[allow(unused)]
    pub fn call_function(
        &self,
        instance: &Instance,
        store: &mut Store<RuntimeCtx>,
        func_name: &str,
        args: &[u8],
    ) -> Result<Vec<u8>> {
        let func = instance
            .get_func(&mut *store, func_name)
            .ok_or_else(|| anyhow::anyhow!("Function '{}' not found", func_name))?;

        let memory = instance
            .get_memory(&mut *store, "memory")
            .ok_or_else(|| anyhow::anyhow!("No memory export"))?;

        if !args.is_empty() {
            let data = memory.data_mut(&mut *store);
            let end = args.len().min(data.len());
            data[..end].copy_from_slice(&args[..end]);
        }

        let typed = func
            .typed::<(i32, i32), i32>(&mut *store)
            .map_err(|e| anyhow::anyhow!("Failed to type function: {}", e))?;

        let result_ptr = typed
            .call(&mut *store, (0, args.len() as i32))
            .map_err(|e| anyhow::anyhow!("Function call failed: {}", e))?;

        let data = memory.data(&*store);
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

impl Default for PluginRuntime {
    fn default() -> Self {
        Self::new(None).expect("Failed to create default PluginRuntime")
    }
}

pub struct LoadedPlugin {
    pub manifest: PluginManifest,
    pub instance: Instance,
    pub store: Store<RuntimeCtx>,
    pub wasm_path: PathBuf,
    pub wasm_hash: Vec<u8>,
    pub compat: PluginCompat,
    pub enabled: bool,
    pub load_time: Instant,
}

pub struct PluginLoadResult {
    pub name: String,
    pub version: String,
    pub manifest: PluginManifest,
    pub wasm_hash: String,
    pub compat: PluginCompat,
    pub is_compatible: bool,
    pub size_bytes: u64,
    pub load_duration_ms: u64,
}

pub fn check_api_compatibility(manifest: &PluginManifest) -> PluginCompat {
    let api_ver = manifest
        .klyron_api
        .clone()
        .unwrap_or_else(|| KLYRON_API_VERSION.to_string());

    let min_version = api_ver.clone();
    let max_version = api_ver;

    PluginCompat {
        min_version,
        max_version,
    }
}

pub fn verify_compatibility(manifest: &PluginManifest, force: bool) -> Result<PluginCompat> {
    let compat = check_api_compatibility(manifest);

    let api_parts: Vec<u32> = KLYRON_API_VERSION
        .split('.')
        .filter_map(|p| p.parse().ok())
        .collect();
    let min_parts: Vec<u32> = compat
        .min_version
        .split('.')
        .filter_map(|p| p.parse().ok())
        .collect();
    let max_parts: Vec<u32> = compat
        .max_version
        .split('.')
        .filter_map(|p| p.parse().ok())
        .collect();

    let api_major = api_parts.first().copied().unwrap_or(0);
    let min_major = min_parts.first().copied().unwrap_or(0);
    let max_major = max_parts.first().copied().unwrap_or(0);

    if api_major < min_major || api_major > max_major {
        let msg = format!(
            "Plugin API version {} is outside required range [{}, {}]",
            KLYRON_API_VERSION, compat.min_version, compat.max_version
        );
        if force {
            warn!("{} - forcing load anyway", msg);
        } else {
            anyhow::bail!("{}", msg);
        }
    }

    Ok(compat)
}

pub fn hash_wasm(bytes: &[u8]) -> Vec<u8> {
    Sha256::digest(bytes).to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_creation() {
        let runtime = PluginRuntime::new(None).unwrap();
        let _ = runtime.engine();
    }

    #[test]
    fn test_runtime_with_sandbox() {
        let sandbox = Arc::new(crate::sandbox::Sandbox::with_defaults());
        let runtime = PluginRuntime::new(Some(sandbox)).unwrap();
        let _ = runtime.engine();
    }

    #[test]
    fn test_runtime_default() {
        let runtime = PluginRuntime::default();
        let _ = runtime.engine();
    }

    #[test]
    fn test_hash_empty() {
        let hash = hash_wasm(b"");
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_hash_deterministic() {
        let data = b"hello wasm world";
        let h1 = hash_wasm(data);
        let h2 = hash_wasm(data);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_hash_different_inputs() {
        let h1 = hash_wasm(b"foo");
        let h2 = hash_wasm(b"bar");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_check_api_compatibility_default() {
        let manifest = PluginManifest {
            name: "test".into(),
            version: "1.0.0".into(),
            ..Default::default()
        };
        let compat = check_api_compatibility(&manifest);
        assert_eq!(compat.min_version, KLYRON_API_VERSION);
        assert_eq!(compat.max_version, KLYRON_API_VERSION);
    }

    #[test]
    fn test_check_api_compatibility_custom() {
        let manifest = PluginManifest {
            name: "test".into(),
            version: "1.0.0".into(),
            klyron_api: Some("2.0.0".into()),
            ..Default::default()
        };
        let compat = check_api_compatibility(&manifest);
        assert_eq!(compat.min_version, "2.0.0");
        assert_eq!(compat.max_version, "2.0.0");
    }

    #[test]
    fn test_verify_compatibility_matching() {
        let manifest = PluginManifest {
            name: "test".into(),
            version: "1.0.0".into(),
            klyron_api: Some(KLYRON_API_VERSION.into()),
            ..Default::default()
        };
        let compat = verify_compatibility(&manifest, false).unwrap();
        assert_eq!(compat.min_version, KLYRON_API_VERSION);
    }

    #[test]
    fn test_verify_compatibility_mismatch() {
        let manifest = PluginManifest {
            name: "test".into(),
            version: "1.0.0".into(),
            klyron_api: Some("99.0.0".into()),
            ..Default::default()
        };
        let result = verify_compatibility(&manifest, false);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("outside required range"));
    }

    #[test]
    fn test_verify_compatibility_force() {
        let manifest = PluginManifest {
            name: "test".into(),
            version: "1.0.0".into(),
            klyron_api: Some("99.0.0".into()),
            ..Default::default()
        };
        let compat = verify_compatibility(&manifest, true).unwrap();
        assert_eq!(compat.min_version, "99.0.0");
    }

    #[test]
    fn test_verify_compatibility_lower_major() {
        let manifest = PluginManifest {
            name: "test".into(),
            version: "1.0.0".into(),
            klyron_api: Some("0.1.0".into()),
            ..Default::default()
        };
        let result = verify_compatibility(&manifest, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_default_compat_struct() {
        let compat = crate::manifest::default_compat();
        assert_eq!(compat.min_version, "1.0.0");
        assert_eq!(compat.max_version, "1.0.0");
    }

}
