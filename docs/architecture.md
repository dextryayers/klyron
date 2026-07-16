# System Architecture

Klyron is a polyglot runtime and developer platform designed with a modular, layered architecture.

## High-level Architecture

```
+-----------------------------------------------------------+
|                      CLI (klyron binary)                   |
+-----------------------------------------------------------+
|                     Command Dispatcher                      |
|  run | dev | build | install | test | create | publish     |
+-----------------------------------------------------------+
|                    Orchestration Layer                      |
|  Project Detection | Config Resolution | Lifecycle Hooks   |
+-----------------------------------------------------------+
|  +---------------------+  +------------------------------+  |
|  |   Engine Layer      |  |   Package Manager            |  |
|  |                     |  |                              |  |
|  |  +----+ +----+      |  |  Resolution | Download       |  |
|  |  | V8 | |Boa |      |  |  Store      | Lockfile       |  |
|  |  +----+ +----+      |  |  Workspaces | Registry       |  |
|  |  +----+ +----+      |  +------------------------------+  |
|  |  |QJS | |JSC |      |  +------------------------------+  |
|  |  +----+ +----+      |  |   Build System               |  |
|  +---------------------+  |                              |  |
|  +---------------------+  |  Bundling  | Minification    |  |
|  |   Adapter System    |  |  Codegen   | Sourcemaps      |  |
|  |                     |  |  WASM      | Asset Pipeline  |  |
|  | Laravel | Rails     |  +------------------------------+  |
|  | Django  | Next.js   |  +------------------------------+  |
|  +---------------------+  |   Test Runner                |  |
|  +---------------------+  |                              |  |
|  |   Plugin System     |  |  Assertions | Coverage       |  |
|  |                     |  |  Reporters  | Snapshots      |  |
|  | WASM ABI | Hooks    |  +------------------------------+  |
|  +---------------------+                                   |
+-----------------------------------------------------------+
|                     Security Sandbox                        |
|  Permissions | Capabilities | TUF | SRI | Signatures       |
+-----------------------------------------------------------+
```

## Engine Layer

Klyron supports multiple JavaScript engines, selectable per project or per file:

| Engine | Language | Use Case |
|---|---|---|
| **V8** | C++ | Default engine, full compatibility, best performance |
| **Boa** | Rust | Embedded systems, custom runtimes, no C++ dependency |
| **QuickJS** | C | Fast startup, resource-constrained environments, CLI tools |
| **JavaScriptCore** | C++ | WebKit-compatible, iOS/tvOS targets |

The engine layer provides a unified API for:

- Module loading and resolution (ESM, CJS)
- Runtime API (console, timers, fetch, Web APIs)
- GC integration and memory management
- Inspector/debugging protocol (Chrome DevTools Protocol)

Engine selection is automatic based on file extension (`--engine` flag overrides):

```bash
klyron run --engine boa script.js
klyron run --engine quickjs tiny.js
```

## Package Manager

The package manager subsystem handles dependency resolution and installation:

- **Content-addressable store**: Packages are stored by content hash, deduplicating across projects
- **Symlink-based node_modules**: pnpm-compatible structure with symlinks to the store
- **Workspace protocol**: `workspace:*` references for monorepo packages
- **Lockfile**: Deterministic installs via `klyron.lock` (JSON format)
- **Multi-registry support**: Primary Klyron registry + npm registry fallback

## Adapter System

Adapters provide framework-specific integrations:

```
Adapter Interface
├── detect(project_dir) -> bool       # Can this adapter handle this project?
├── configure(config) -> Result       # Modify klyron config for this framework
├── dev_server_config() -> DevConfig  # Dev server settings (HMR, proxy)
├── build_config() -> BuildConfig     # Build settings
└── commands() -> Vec<Command>         # Framework-specific CLI commands
```

Built-in adapters:

- **Laravel**: Vite integration, Artisan proxy, Blade compilation, Inertia SSR
- **Rails**: Propshaft integration, importmap handling, Turbo/Hotwire
- **Django**: Whitenoise static files, Daphne ASGI, template compilation
- **Next.js**: React server components, App Router, API routes

## Security Sandbox

The security sandbox enforces resource access permissions:

```
+---------------------------+
| User Script               |
+---------------------------+
        |
        v
+---------------------------+
| Permission Gatekeeper     |
| --allow-net? --allow-read?|
+---------------------------+
        |
        v
+---------------------------+
| OS System Calls            |
+---------------------------+
```

| Permission | Flag | Default |
|---|---|---|
| Network access | `--allow-net` | Denied |
| File read | `--allow-read` | Denied |
| File write | `--allow-write` | Denied |
| Environment vars | `--allow-env` | Denied |
| Process spawning | `--allow-run` | Denied |
| FFI access | `--allow-ffi` | Denied |

## CLI Command Dispatcher

The CLI entry point parses commands and dispatches to subsystems:

```
klyron <command> [options] [args]
```

Commands are registered in a central registry and can be extended by adapters and plugins. Each command implements a trait:

```rust
trait Command {
    fn name(&self) -> &str;
    fn run(&self, args: &Args) -> Result<(), ExitCode>;
}
```

## Plugin System (WASM)

Plugins are WebAssembly modules loaded at runtime. They communicate via a host ABI:

- **Plugin Manager**: Loads, caches, and instantiates WASM modules
- **Hook Dispatcher**: Calls plugin lifecycle hooks at defined points
- **Permission Guard**: Restricts plugin access to declared capabilities
- **Host Imports**: Provides logging, config access, I/O functions to plugins

## Data Flow: `klyron dev`

```
klyron dev
  |
  v
Project detection (read klyron.json, package.json, framework files)
  |
  v
Engine initialization (V8 by default)
  |
  v
Package resolution (resolve deps from klyron.lock)
  |
  v
Adapter detection (Laravel? React? Vue?)
  |
  v
Dev server startup
  |-- HTTP server (port 5173)
  |-- WebSocket server (HMR)
  |-- File watcher (chokidar-like)
  |-- Framework proxy (e.g., Artisan on :8000)
  |
  v
Source compilation (esbuild/swc-based)
  |-- JS/TS transpilation
  |-- CSS processing
  |-- Asset hashing
  |
  v
File change -> HMR update -> Browser reload
```
