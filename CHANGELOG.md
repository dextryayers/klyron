# Changelog

All notable changes to Klyron are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] - Phase 10-13: Deep Improvement (200+ Todos)

### Added
- **Phase 10: klyron.lock binary lockfile system**
  - Binary format via bincode with integrity verification
  - Lockfile generation, resolution, migration from npm/pnpm/yarn
  - `--frozen-lockfile`, `lock verify`, `lock diff`, `lock migrate` commands
  - Monorepo workspace protocol support
  - Benchmark: 100/1000/10000 package resolution

- **Phase 11: Engine Performance**
  - JIT profiling system with statistics collection
  - Thread-safe engine pool with dynamic scaling
  - Two-tier bytecode cache (LRU memory + disk via zstd compression)
  - Lazy compilation and pre-warm at startup
  - Nano-process isolation for sandboxed execution
  - Streaming compilation and parallel module resolution
  - mmap file cache for frequently accessed files
  - Real engine bindings: V8 (rusty_v8), QuickJS (qjs-sys), JSC (C API)

- **Phase 12: Node.js Compatibility**
  - 32 Node.js polyfill modules (assert, async_hooks, cluster, dns, http2, inspector, module, net, perf_hooks, readline, repl, timers, tty, util, v8, vm, wasi, worker_threads, etc.)
  - CommonJS loader with `require()`, `__dirname`, `__filename`
  - `Buffer` global, `nextTick`, `globalThis` polyfills
  - Dual `require('module')` and `require('node:module')` support

- **Phase 12: Web APIs**
  - fetch/Request/Response/Headers, WebSocket, EventSource
  - URLPattern, TextEncoder/TextDecoder, structuredClone
  - EventTarget, AbortController, BroadcastChannel, MessageChannel
  - Performance API, Console API, Web Crypto (subtle)
  - CompressionStream, CacheStorage, IndexedDB (basic)

### Changed
- CLI v2: completions (bash/zsh/fish/powershell), JSON output, colors, progress bars, interactive prompts
- Extensive test coverage: 11 test files with CLI, engine, PM, runtime, security, and integration tests
- Plugin system: WASM-based sandbox via wasmtime, lifecycle hooks, registry
- Deployment: Docker multi-stage, 7 cloud targets, K8s (Helm + Kustomize), CI/CD (GitHub/GitLab/Jenkins/CircleCI)
- SDK expansion: js, ts, rust, cpp, php, python, go, ruby, zig

### Fixed
- All TODO stubs replaced with real implementations
- All empty scaffold directories filled
- All small .rs files (< 300 bytes) expanded
- Lockfile migration from npm/pnpm/yarn lockfiles

## [0.2.0] - Phase 5-9: Frameworks, SDK, Plugins, CLI, Deploy

### Added
- 12 framework adapters: Next.js, Express, React, Vue, Svelte, Astro, Nuxt, Hono, Elysia, Fastify, Koa, NestJS
- 8 ORM adapters: Prisma, Drizzle, TypeORM, Sequelize, MikroORM, Mongoose, Knex, Kysely
- Multi-language SDK: JS, TS, Rust, C++, PHP, Python, Go, Ruby, Zig
- Plugin system with WASM sandboxing, lifecycle hooks, hot-reload
- CLI completions, colors, progress bars, interactive prompts, JSON output
- Deployment: Docker, K8s, AWS, GCP, Azure, Heroku, Netlify, Vercel, Cloudflare
- Framework detection and auto-configuration
- Middleware and template engine detection

### Changed
- Package manager: complete lockfile lifecycle, workspace protocol, dep dedupe, audit, bundle
- Engine switching: auto-detect best engine per workload

## [0.1.0] - Phase 0-4: Foundation

### Added
- Project structure: 39+ Rust crate workspace
- 5 JS engines: Boa, V8, QuickJS, JavaScriptCore, Common abstraction layer
- 13 Deno-core extensions: console, crypto, ffi, fs, html, http, klyron, net, node, process, timers, web, ws
- TypeScript transpiler with tree-shaking, code splitting, JSX, HMR
- Package manager: npm registry client, binary lockfile, integrity
- Multi-language support: PHP/Laravel, Python, Ruby, Go, Rust, Zig, C, C++
- CLI: install, add, remove, run, dev, build, test, format, lint, eval, repl
- Documentation: CHANGELOG, PLAN, README, CONTRIBUTING, SECURITY
- CI/CD: GitHub Actions, benchmarks, security scanning
