# Klyron Architecture

## Overview

Klyron is a universal polyglot JS runtime + web developer multitool, organized as a Rust workspace of 39+ crates with 5 JS engines (Boa, V8, QuickJS, JavaScriptCore), 13 Deno-core extensions, 32 Node.js polyfills, 22+ Web API globals, 12 framework adapters, 8 ORM adapters, and 5 SDK languages.

## Crate Dependency Graph

```
klyron-cli  (binary entrypoint)
   |
   +-- klyron-engine      (JS engine abstraction + V8/QuickJS/JSC/Boa bindings)
   |     +-- engines/boa      (pure Rust JS engine)
   |     +-- engines/v8       (rusty_v8 bindings)
   |     +-- engines/quickjs  (qjs-sys bindings)
   |     +-- engines/jsc      (JavaScriptCore C API)
   |
   +-- klyron-pm           (package manager: install, lock, publish, audit)
   |     +-- lockfile      (klyron.lock binary format via bincode)
   |     +-- registry      (npm registry client)
   |     +-- audit         (vulnerability scanning)
   |     +-- bundle        (dependency bundling)
   |
   +-- klyron-plugin-system  (WASM-based plugin sandbox)
   |
   +-- klyron-framework   (detect, scaffold, build 12+ frameworks)
   |     +-- adapters/    (next, express, react, vue, svelte, astro, nuxt, hono, elysia, fastify, koa, nest)
   |     +-- adapters/orm (prisma, drizzle, typeorm, sequelize, mikroorm, mongoose, knex, kysely)
   |
   +-- klyron-deploy      (Docker, K8s, serverless, cloud target deployment)
   |
   +-- klyron-bench       (performance benchmarks)
   |
   +-- src/ext/           (13 Deno-core extensions: console, crypto, ffi, fs, html, http, klyron, net, node, process, timers, web, ws)
   |
   +-- runtime/js/        (32 Node.js polyfills, globals.js with 40+ Web APIs)
   |
   +-- sdk/               (js, ts, rust, cpp, php, python, go, ruby, zig)
```

## Engine Abstraction Layer

```
EngineTrait
   |
   +-- BoaEngine     (interpreted, pure Rust, debuggable)
   +-- V8Engine      (JIT-compiled, highest perf, via rusty_v8)
   +-- QuickJSEngine (lightweight, embeddable, via qjs-sys)
   +-- JSCEngine     (macOS-optimized, via JavaScriptCore C API)
   |
   +-- EnginePool    (thread-safe pool, auto-switching by workload)
   +-- BytecodeCache (LRU + disk two-tier cache with zstd compression)
   +-- LazyCompiler  (defer compilation until first execution)
   +-- PreWarmCache  (pre-compile common modules at startup)
   +-- NanoProcess   (isolate each eval in separate OS process)
```

Engine auto-switching logic:
- **Hot path** (frequent execution) → V8
- **Short scripts** (< 10ms) → QuickJS
- **Batch processing** → JSC
- **Debug/dev** → Boa

## Extension System Architecture

```
ExtensionManager  (in klyron runtime)
   |
   +-- ConsoleExt    (console.log/warn/error, formatting)
   +-- CryptoExt     (crypto module, Web Crypto API)
   +-- FfiExt        (FFI to native libraries)
   +-- FsExt         (file system operations)
   +-- HtmlExt       (HTML parsing/rendering)
   +-- HttpExt       (HTTP client/server)
   +-- KlyronExt     (Klyron.* globals)
   +-- NetExt        (TCP/UDP networking)
   +-- NodeExt       (Node.js compatibility layer)
   +-- ProcessExt    (process management)
   +-- TimersExt     (setTimeout, setInterval, setImmediate)
   +-- WebExt        (fetch, WebSocket, etc.)
   +-- WsExt         (WebSocket protocol)
```

## Node.js Compatibility Layer

32 polyfill modules in `runtime/js/` mapped via `globals.js`:

| Module | File | Status |
|--------|------|--------|
| assert | node_assert.js | ✅ |
| async_hooks | node_async_hooks.js | ✅ |
| buffer | node_buffer.js | ✅ |
| child_process | node_child_process.js | ✅ |
| cluster | node_cluster.js | ✅ |
| crypto | node_crypto.js | ✅ |
| diagnostics_channel | node_diagnostics_channel.js | ✅ |
| dns | node_dns.js | ✅ |
| events | node_events.js | ✅ |
| fs | node_fs.js | ✅ |
| http | node_http.js | ✅ |
| http2 | node_http2.js | ✅ |
| https | node_https.js | ✅ |
| inspector | node_inspector.js | ✅ |
| module | node_module.js | ✅ |
| net | node_net.js | ✅ |
| os | node_os.js | ✅ |
| path | node_path.js | ✅ |
| perf_hooks | node_perf_hooks.js | ✅ |
| querystring | node_querystring.js | ✅ |
| readline | node_readline.js | ✅ |
| repl | node_repl.js | ✅ |
| stream | node_stream.js | ✅ |
| string_decoder | node_string_decoder.js | ✅ |
| timers | node_timers.js | ✅ |
| trace_events | node_trace_events.js | ✅ |
| tty | node_tty.js | ✅ |
| util | node_util.js | ✅ |
| v8 | node_v8.js | ✅ |
| vm | node_vm.js | ✅ |
| wasi | node_wasi.js | ✅ |
| worker_threads | node_worker_threads.js | ✅ |

## Package Manager (klyron-pm)

23 modules handling:
- **Registry**: npm registry client with caching, authentication, mirror support
- **Lockfile**: Binary format via bincode (klyron.lock), integrity verification, migration from npm/pnpm/yarn
- **Audit**: Vulnerability scanning with severity levels (CRITICAL/HIGH/MEDIUM/LOW)
- **Bundle**: Dependency bundling with minification, sourcemaps
- **Dedupe**: Tree deduplication, hoisting analysis
- **Workspace**: Monorepo workspace support with per-member operations
- **Scripts**: Lifecycle hook execution (pre/post hooks), circular detection
- **Security**: TUF integration, signing, provenance, rate limiting

## CLI Commands

26 command modules in `src/cli/src/commands/`:
ai, artisan, bench, build, cache, check, compat, db, deploy, dev, docker, format, helpers, lint, napi, orm, plugin, pm, registry, runtime, scaffold, scripts, test, utils, workspace

15 supporting modules in `crates/klyron_cli/src/`:
color, colors, completions, config, doctor, error, interactive, logger, output, progress, scaffold_inline, signal, telemetry, upgrade, lib

## Deployment System

31 files across 7 categories:
- **Docker**: Multi-stage, dev, prod Dockerfiles
- **Cloud**: AWS, GCP, Azure, Heroku, Netlify, Vercel, Cloudflare
- **Serverless**: WASM-based, edge computing
- **Kubernetes**: Manifests, Helm charts, Kustomize overlays
- **CI**: GitHub Actions, GitLab CI, Jenkins, CircleCI
- **Monitoring**: Health checks, metrics (Prometheus), logging, alerting
- **Config**: Deployment configuration with validation

## SDK Languages

9 SDK packages:
- JavaScript (js), TypeScript (ts), Rust, C++, PHP, Python, Go, Ruby, Zig
