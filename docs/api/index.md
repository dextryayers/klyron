# Klyron API Documentation

Klyron is composed of multiple Rust crates. Below is the complete API reference for each crate.

## Core Crates

| Crate | Description | Status |
|-------|-------------|--------|
| [klyron_cli](../klyron_cli/index.html) | CLI entry point, command dispatch, engine helpers | Production |
| [klyron_engine](../klyron_engine/index.html) | JavaScript engine abstraction (Boa, QuickJS, JSC, V8) | Production |
| [klyron_pm](../klyron_pm/index.html) | Package manager (npm-compatible: install, resolve, lockfile, publish) | Production |
| [klyron_plugin](../klyron_plugin/index.html) | WASM-based plugin system with hooks and sandboxing | Production |

## Runtime & Execution

| Crate | Description |
|-------|-------------|
| [klyron_core](../klyron_core/index.html) | Core runtime with sandbox, permissions, TypeScript support |
| [klyron_loader](../klyron_loader/index.html) | Module resolution and loading |
| [klyron_bundler](../klyron_bundler/index.html) | JavaScript/TypeScript bundler |
| [klyron_transpiler](../klyron_transpiler/index.html) | TypeScript/JSX transpiler |
| [klyron_shell](../klyron_shell/index.html) | Interactive REPL shell |

## Developer Experience

| Crate | Description |
|-------|-------------|
| [klyron_debugger](../klyron_debugger/index.html) | Chrome DevTools Protocol debugger |
| [klyron_lsp](../klyron_lsp/index.html) | Language Server Protocol implementation |
| [klyron_ai](../klyron_ai/index.html) | AI-powered code generation (commit messages, fixes) |
| [klyron_config](../klyron_config/index.html) | Configuration management (JSON, TOML, YAML) |

## HTTP & Web

| Crate | Description |
|-------|-------------|
| [klyron_http](../klyron_http/index.html) | HTTP client and server |
| [klyron_watcher](../klyron_watcher/index.html) | File system watcher |

## Package Management

| Crate | Description |
|-------|-------------|
| [klyron_registry](../klyron_registry/index.html) | Registry client (npm, PyPI, RubyGems, etc.) |
| [klyron_workspace](../klyron_workspace/index.html) | Monorepo/workspace management |
| [klyron_compat](../klyron_compat/index.html) | Node.js/npm compatibility checker |

## Framework Adapters

| Crate | Description |
|-------|-------------|
| [klyron_adapter](../klyron_adapter/index.html) | Framework adapter system (43 frameworks) |

## Build & Deploy

| Crate | Description |
|-------|-------------|
| [klyron_deploy](../klyron_deploy/index.html) | Deployment (Vercel, Netlify, Cloudflare, Railway, Fly) |
| [klyron_docker](../klyron_docker/index.html) | Docker integration |
| [klyron_napi](../klyron_napi/index.html) | N-API native module support |

## QA & Tooling

| Crate | Description |
|-------|-------------|
| [klyron_test](../klyron_test/index.html) | Test runner |
| [klyron_bench](../klyron_bench/index.html) | Benchmarking framework (statistical analysis) |
| [klyron_linter](../klyron_linter/index.html) | Code linter |
| [klyron_formatter](../klyron_formatter/index.html) | Code formatter |

## Infrastructure

| Crate | Description |
|-------|-------------|
| [klyron_cache](../klyron_cache/index.html) | Concurrent cache with TTL, LFU, eviction |
| [klyron_crypto](../klyron_crypto/index.html) | Cryptography utilities (bcrypt, argon2, cron) |
| [klyron_template](../klyron_template/index.html) | Project scaffolding templates |

## Extensions (deno_core)

| Crate | Description |
|-------|-------------|
| klyron-ext-console | Console API |
| klyron-ext-timers | setTimeout/setInterval |
| klyron-ext-fs | File system operations |
| klyron-ext-net | Networking |
| klyron-ext-http | HTTP client |
| klyron-ext-crypto | Web Crypto API |
| klyron-ext-web | Web APIs |
| klyron-ext-html | HTML rendering |
| klyron-ext-node | Node.js compatibility |
| klyron-ext-klyron | Klyron-specific APIs |
| klyron-ext-ffi | Foreign Function Interface |
| klyron-ext-ws | WebSocket support |

## CLI Commands

```
klyron --help
```

Full documentation is available at [docs.klyron.dev](https://docs.klyron.dev) (coming soon).

To generate local docs:

```bash
cargo doc --open
```
