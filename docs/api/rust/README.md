# Rust API Reference

This directory contains the Rust API documentation for Klyron's internal crates.

## Crate Overview

| Crate | Description |
|---|---|
| `klyron-core` | Core types, traits, and runtime primitives |
| `klyron-engine` | Multi-engine abstraction layer (V8, Boa, QuickJS, JSC) |
| `klyron-package` | Package resolution, installation, and registry client |
| `klyron-bundler` | Bundler, minifier, and codegen pipeline |
| `klyron-test` | Test runner, assertion library, and reporters |
| `klyron-adapter` | Framework adapter interface and built-in adapters |
| `klyron-plugin` | WASM plugin host ABI and lifecycle management |

## Generated Docs

To generate and open the Rust API documentation locally:

```bash
cargo doc --workspace --no-deps --open
```

## Key Modules

### klyron-engine

```rust
// Engine trait implemented by all backends
pub trait Engine {
    fn name(&self) -> &str;
    fn evaluate(&self, source: &str, options: EvaluateOptions) -> Result<Value>;
    fn execute_module(&self, specifier: &str) -> Result<Module>;
}

// Engine factory
pub fn create_engine(config: &EngineConfig) -> Result<Box<dyn Engine>>;
```

### klyron-core

```rust
// Permission set
pub struct Permissions {
    pub net: Vec<String>,
    pub read: Vec<String>,
    pub write: Vec<String>,
    pub env: bool,
    pub run: bool,
    pub ffi: bool,
}

// Module resolution
pub trait ModuleLoader {
    fn resolve(&self, specifier: &str, referrer: &str) -> Result<Module>;
    fn load(&self, module: &Module) -> Result<ModuleSource>;
}
```

## SDK for Plugin Development

The `klyron-plugin-sdk` crate provides macros and types for building WASM plugins:

```toml
[dependencies]
klyron-plugin-sdk = "0.1"
```

```rust
use klyron_plugin_sdk::*;

#[klyron_plugin]
mod my_plugin {
    #[hook(on_load)]
    fn on_load(config: PluginConfig) -> Result<(), PluginError> {
        Ok(())
    }
}
```

For more details, see [Creating Plugins](../guides/creating-plugins.md).
