# Plugin Metadata Format

Plugins in the Klyron registry use a `klyron-plugin.toml` manifest file alongside the compiled `.wasm` binary.

## Manifest Structure

```toml
[plugin]
name = "my-plugin"
version = "0.1.0"
description = "Description of the plugin"
authors = ["Author Name <email@example.com>"]
license = "MIT"

[permissions]
allow_net = ["*.example.com"]
allow_fs = ["/tmp", "./data"]
allow_env = ["HOME", "PATH"]
allow_stdio = true

[dependencies]
dep1 = ">=0.1.0"
dep2 = { version = "1.0", optional = true }

[hooks]
on_load = "plugin_init"
on_init = "setup"
on_start = "start"
on_stop = "shutdown"
on_destroy = "cleanup"

[sandbox]
max_memory_bytes = 67108864
max_fuel = 1000000
```

## Field Reference

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `plugin.name` | string | yes | Unique plugin identifier |
| `plugin.version` | semver | yes | Plugin version |
| `plugin.description` | string | no | Human-readable description |
| `plugin.authors` | string[] | no | Author list |
| `plugin.license` | string | no | SPDX license identifier |
| `permissions.allow_net` | string[] | no | Allowed network domains/URLs |
| `permissions.allow_fs` | string[] | no | Allowed filesystem paths |
| `permissions.allow_env` | string[] | no | Allowed environment variables |
| `permissions.allow_stdio` | bool | no | Allow stdin/stdout/stderr |
| `dependencies` | table | no | Plugin dependency map |
| `hooks` | table | no | Lifecycle hook function names |
| `sandbox.max_memory_bytes` | int | no | Maximum WASM memory in bytes |
| `sandbox.max_fuel` | int | no | Maximum fuel/instructions |

## Package Format

Each plugin package is a gzipped tarball (`.tar.gz`) containing:

```
my-plugin-0.1.0/
  plugin.wasm
  klyron-plugin.toml
  README.md (optional)
  LICENSE (optional)
```

## Registry Index

The registry index is stored at `registry.klyron.dev/plugins/<name>/<version>/` with:
- `manifest.json` - Package metadata
- `download` - Binary download endpoint
- `checksums` - SHA-256 checksums
