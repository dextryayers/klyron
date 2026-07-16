# Creating Plugins

Klyron's plugin system allows extending the runtime with custom functionality using WebAssembly (WASM) plugins.

## Plugin Architecture

Plugins in Klyron are compiled to WebAssembly and communicate with the host runtime through a well-defined ABI. This provides:

- **Language agnosticism**: Write plugins in Rust, Go, C, Zig, or any language that compiles to WASM
- **Sandboxing**: Plugins run in a WASM sandbox with limited host access
- **Performance**: Near-native execution speed with WASM's JIT compilation
- **Versioning**: Plugin ABI is versioned for forward compatibility

### ABI Overview

```
+-------------------+       +-------------------+
|   Klyron Host     |       |   WASM Plugin     |
|                   |       |                   |
|  PluginManager    | <---> |  exports:         |
|  HookDispatcher   |       |  klyron_plugin_*  |
|  PermissionGuard  |       |  host_imports:     |
|                   |       |  klyron_host_*    |
+-------------------+       +-------------------+
```

## Plugin Lifecycle Hooks

Plugins implement lifecycle hooks that Klyron calls at specific points:

### Available Hooks

| Hook | When Called | Purpose |
|---|---|---|
| `on_load` | Plugin loaded | Initialize state |
| `on_unload` | Plugin unloaded | Clean up resources |
| `on_file_open` | File opened | Intercept file access |
| `on_module_resolve` | Module import resolved | Custom resolution |
| `on_transform` | Source transformed | Code transformation |
| `on_build_start` | Build begins | Pre-build setup |
| `on_build_end` | Build completes | Post-build processing |
| `on_request` | HTTP request received | Middleware |
| `on_test_start` | Test suite begins | Test setup |
| `on_test_case` | Individual test | Custom assertions |

## Creating a Plugin

### Prerequisites

- Rust toolchain with `wasm32-wasi` target: `rustup target add wasm32-wasi`
- WASM plugin SDK: `klyron install --global klyron-plugin-sdk`

### Step 1: Scaffold

```bash
klyron create plugin my-plugin
cd my-plugin
```

This creates a plugin project with:

```
my-plugin/
├── src/
│   └── lib.rs
├── Cargo.toml
├── klyron.json
└── README.md
```

### Step 2: Implement hooks

```rust
use klyron_plugin_sdk::*;

#[klyron_plugin]
mod my_plugin {
    #[hook(on_load)]
    fn load(config: PluginConfig) -> Result<(), PluginError> {
        log("Plugin loaded successfully");
        Ok(())
    }

    #[hook(on_transform)]
    fn transform(source: &str, path: &str) -> Result<String, PluginError> {
        // Example: Strip console.log calls in production
        let transformed = source.replace("console.log(", "// console.log(");
        Ok(transformed)
    }

    #[hook(on_request)]
    fn request(req: HttpRequest) -> Result<HttpResponse, PluginError> {
        // Example: Add security headers
        let mut resp = HttpResponse::new();
        resp.headers.insert(
            "X-Content-Type-Options".into(),
            "nosniff".into(),
        );
        resp.headers.insert(
            "X-Frame-Options".into(),
            "DENY".into(),
        );
        Ok(resp)
    }
}
```

### Step 3: Build

```bash
klyron build
```

This compiles the plugin to a `.wasm` file in the `dist/` directory.

### Step 4: Test locally

```bash
klyron plugin load --path dist/my_plugin.wasm
klyron dev --plugin my_plugin
```

### Step 5: Publish

```bash
klyron publish
```

## Host API

Plugins can call host functions:

```rust
use klyron_plugin_sdk::*;

#[hook(on_file_open)]
fn file_open(path: &str) -> Result<FileAction, PluginError> {
    // Log the access
    host_log(&format!("File opened: {}", path));

    // Get config values
    let allowed_dir: String = host_config_get("allowed_directories")?;

    // Check host permissions
    if !path.starts_with(&allowed_dir) {
        return Ok(FileAction::Deny("Access denied".into()));
    }

    Ok(FileAction::Allow)
}
```

### Available Host Functions

| Function | Description |
|---|---|
| `host_log(message)` | Write to Klyron's log |
| `host_config_get(key)` | Read plugin configuration |
| `host_http_fetch(url)` | Make HTTP requests |
| `host_read_file(path)` | Read file (permissions apply) |
| `host_write_file(path, data)` | Write file (permissions apply) |
| `host_resolve_module(specifier)` | Resolve module specifier |
| `host_emit_event(name, data)` | Emit custom events |

## Plugin Configuration

Configure plugins in `klyron.json`:

```jsonc
{
  "plugins": {
    "my-plugin": {
      "source": "dist/my_plugin.wasm",
      "permissions": ["read", "log"],
      "config": {
        "allowed_directories": ["/app/src"],
        "strip_logs": true
      },
      "hooks": ["on_load", "on_transform", "on_request"]
    }
  }
}
```

## Plugin Permissions

Plugins have their own permission model separate from user scripts:

| Permission | Description |
|---|---|
| `log` | Write to host log |
| `read` | Read files |
| `write` | Write files |
| `network` | Make HTTP requests |
| `module_resolve` | Resolve module specifiers |
| `emit_event` | Emit custom events |
| `host_config` | Read host configuration |

## Publishing Plugins

Plugins are published to the Klyron registry like regular packages:

```bash
klyron publish
klyron install my-plugin
klyron dev --plugin my-plugin
```

## Example Plugin Walkthrough

### Log Request Duration Plugin

```rust
use klyron_plugin_sdk::*;
use std::time::Instant;

#[klyron_plugin]
mod timing_plugin {
    thread_local! {
        static START: std::cell::RefCell<Option<Instant>> =
            std::cell::RefCell::new(None);
    }

    #[hook(on_request)]
    fn request_start(req: HttpRequest) -> Result<HttpResponse, PluginError> {
        START.with(|start| {
            *start.borrow_mut() = Some(Instant::now());
        });
        // Return None to continue normal processing
        Ok(HttpResponse::default())
    }

    #[hook(on_build_end)]
    fn request_end(result: &BuildResult) -> Result<(), PluginError> {
        START.with(|start| {
            if let Some(duration) = start.borrow_mut().take() {
                host_log(&format!(
                    "Build completed in {:?}",
                    duration
                ));
            }
        });
        Ok(())
    }
}
```

Build and test:

```bash
klyron build
klyron build --plugin timing_plugin
```
