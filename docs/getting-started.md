# Getting Started

## Installation

### From Source (Recommended)

Klyron is written in Rust and published on crates.io. Install it with Cargo:

```bash
cargo install klyron
```

This requires Rust 1.70+ and a working Cargo toolchain. If you don't have Rust installed, visit [rustup.rs](https://rustup.rs).

### Pre-built Binaries

Pre-compiled binaries are available on the [GitHub Releases](https://github.com/anomalyco/klyron/releases) page for Linux, macOS, and Windows. Download the archive for your platform, extract it, and place the `klyron` binary in your `PATH`.

### Platform Support

| Platform | Status |
|---|---|
| Linux (x86_64) | Tier 1 |
| macOS (x86_64, ARM64) | Tier 1 |
| Windows (x86_64) | Tier 2 |
| Linux (ARM64) | Tier 2 |

## Your First Project

Create a new project with the `create` command:

```bash
klyron create react my-app
cd my-app
klyron dev
```

This scaffolds a React project with TypeScript, Vite-like configuration, and HMR out of the box. The dev server starts on `http://localhost:5173`.

### Other Templates

```bash
klyron create vanilla-ts my-app     # Plain TypeScript
klyron create vue my-app             # Vue 3
klyron create svelte my-app          # Svelte
klyron create laravel my-app         # Laravel + Vite
klyron create laravel-react my-app   # Laravel + Inertia + React
klyron create library my-app         # Library (ESM/CJS dual output)
```

## Basic Commands Overview

| Command | Description |
|---|---|
| `klyron run <file>` | Execute a script (JS, TS, PHP) |
| `klyron dev` | Start dev server with HMR |
| `klyron build` | Build for production |
| `klyron install <pkg>` | Install a package |
| `klyron test` | Run tests |
| `klyron create <template> <dir>` | Scaffold a new project |
| `klyron workspace <cmd>` | Manage workspaces |
| `klyron publish` | Publish a package |
| `klyron add <adapter>` | Install a framework adapter |

## Configuration

Klyron uses a `klyron.json` or `klyron.jsonc` file in the project root. Here is a reference configuration:

```jsonc
{
  "$schema": "https://klyron.dev/schema.json",
  "engine": "v8",
  "compilerOptions": {
    "target": "esnext",
    "module": "esnext",
    "jsx": "react-jsx",
    "strict": true
  },
  "devServer": {
    "port": 5173,
    "host": "localhost",
    "hmr": true
  },
  "build": {
    "outDir": "dist",
    "minify": true,
    "sourcemap": true,
    "target": ["es2020", "chrome89", "firefox89"]
  },
  "install": {
    "registry": "https://registry.klyron.dev",
    "lockfile": true
  },
  "workspaces": ["packages/*"]
}
```

Configuration fields can also be passed via CLI flags, which take precedence over `klyron.json`.
