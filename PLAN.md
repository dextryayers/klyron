# Klyron — Master Plan Arsitektur & Roadmap

**Versi**: 2.0  
**Status**: Planning  
**Target**: Production-ready runtime setara Bun/Deno untuk ekosistem JS, TS, PHP, Python, Ruby, Go, Rust, Zig, C, C++

---

## Daftar Isi

1. [Visi & Misi](#1-visi--misi)
2. [Struktur Repository](#2-struktur-repository)
3. [Arsitektur Crate](#3-arsitektur-crate)
4. [JavaScript Engine Strategy](#4-javascript-engine-strategy)
5. [Polyglot Engine Protocol](#5-polyglot-engine-protocol)
6. [Framework Adapters](#6-framework-adapters)
7. [Laravel Ecosystem](#7-laravel-ecosystem)
8. [ORM Layer](#8-orm-layer)
9. [Template & Scaffold System](#9-template--scaffold-system)
10. [Registry & Package Management](#10-registry--package-management)
11. [SDK Multi-Bahasa](#11-sdk-multi-bahasa)
12. [CLI Command Reference](#12-cli-command-reference)
13. [API Contracts](#13-api-contracts)
14. [Config Format (klyron.toml)](#14-config-format-klyrontoml)
15. [Performance Targets](#15-performance-targets)
16. [Roadmap & Timeline](#16-roadmap--timeline)
17. [Contributing Guide](#17-contributing-guide)

---

## 1. Visi & Misi

### Visi

Klyron adalah universal polyglot runtime yang menyatukan seluruh ekosistem
pengembangan web — JavaScript, TypeScript, PHP/Laravel, Python, Ruby, Go, Rust,
Zig, C, dan C++ — dalam satu toolchain yang koheren, cepat, dan modern.

### Misi

1. **Satu CLI untuk semua bahasa** — `klyron run`, `klyron dev`, `klyron build`
   bekerja di proyek apapun tanpa konfigurasi manual.
2. **First-class Laravel support** — Laravel bukan second-class citizen. Semua
   fitur artisan, blade, horizon, telescope, livewire, inertia bekerja penuh.
3. **Node.js / Bun compatible** — Jalankan project Node.js atau Bun tanpa
   perubahan kode. `require()`, `node_modules`, built-in modules semua didukung.
4. **Framework agnostic** — Next.js, Nuxt, SvelteKit, Astro, SolidStart, Qwik,
   Angular, Remix — semuanya di-scaffold, di-build, di-dev dengan satu command.
5. **Multi-registry package management** — npm, PyPI, RubyGems, crates.io,
   Packagist, Go proxy — satu CLI untuk semua.
6. **Performance native** — Rust core, V8/QuickJS/Boa engine, zero-cost
   abstractions, snapshot startup.

---

## 2. Struktur Repository

```
klyron/
│
├── .github/
│   └── workflows/
│       ├── ci.yml                      ← Rust CI (test, lint, build)
│       ├── release.yml                 ← Release pipeline (crates.io + npm)
│       ├── benchmark.yml               ← Benchmark regression
│       └── docs.yml                    ← Documentation deploy
│
├── assets/
│   ├── logo.svg
│   ├── icon.png
│   └── banner.png
│
├── crates/
│   ├── klyron_cli/                     ← CLI entry point, command routing
│   ├── klyron_runtime/                 ← Core runtime: event loop, v8, extensions
│   ├── klyron_engine/                  ← Polyglot engine trait + bridges
│   ├── klyron_loader/                  ← Module resolution (ESM/CJS/node_modules)
│   ├── klyron_pm/                      ← Package management (npm/pip/gem/cargo/go)
│   ├── klyron_bundler/                 ← Bundling abstraction (esbuild/vite/rollup)
│   ├── klyron_transpiler/              ← TS/JSX transpiler (swc/esbuild/tsc)
│   ├── klyron_watcher/                 ← File watching + HMR (chokidar/notify)
│   ├── klyron_config/                  ← Config management (klyron.toml + klyron.config.ts)
│   ├── klyron_test/                    ← Test runner abstraction
│   ├── klyron_linter/                  ← Linter abstraction
│   ├── klyron_formatter/               ← Formatter abstraction
│   ├── klyron_process/                 ← Child process spawn/manage
│   ├── klyron_logger/                  ← Logging & diagnostic reporting
│   ├── klyron_telemetry/               ← Telemetry & metrics collection
│   ├── klyron_updater/                 ← Self-update mechanism
│   ├── klyron_shell/                   ← REPL + shell subsystem
│   ├── klyron_template/                ← Template engine for scaffold system
│   ├── klyron_napi/                    ← Node-API (NAPI) binding layer
│   ├── klyron_node/                    ← Node.js compat layer (require, process, Buffer)
│   ├── klyron_web/                     ← Web API (fetch, URL, Headers, Request/Response)
│   ├── klyron_http/                    ← HTTP server + client
│   ├── klyron_fs/                      ← File system ops
│   ├── klyron_dns/                     ← DNS resolution
│   ├── klyron_crypto/                  ← Crypto ops (hash, encrypt, random)
│   ├── klyron_sqlite/                  ← SQLite binding
│   ├── klyron_postgres/                ← PostgreSQL binding
│   ├── klyron_mysql/                   ← MySQL binding
│   ├── klyron_cache/                   ← Cache layer (memory, redis, memcached)
│   ├── klyron_plugin/                  ← Plugin system (wasm, hot-reload)
│   ├── klyron_workspace/               ← Monorepo workspace management
│   ├── klyron_registry/                ← Registry client (npm/PyPI/Gems/Cargo/Packagist)
│   ├── klyron_bench/                   ← Benchmarking framework
│   ├── klyron_compat/                  ← Node.js compatibility checker
│   ├── klyron_docker/                  ← Docker integration (generate, build, run)
│   ├── klyron_deploy/                  ← Deployment abstraction (Vercel, CF, Railway, Fly)
│   ├── klyron_ai/                      ← AI features (generate, optimize, review, docs, test, migrate)
│   └── klyron_utils/                   ← Shared utilities
│
├── engines/                            ← JS engine source implementations
│   ├── v8/                             ← V8 via deno_core + cppgc (C++/Rust)
│   ├── boa/                            ← Boa crate wrapper (Rust native)
│   ├── quickjs/                        ← QuickJS C binding
│   └── jsc/                            ← JavaScriptCore (future)
│
├── runtime/                            ← JS/TS runtime bootstrap files
│   ├── js/
│   │   ├── globals.js                  ← globalThis polyfills
│   │   ├── console.js                  ← console.* implementation
│   │   ├── timers.js                   ← setTimeout/setInterval/setImmediate
│   │   ├── fetch.js                    ← fetch/Request/Response
│   │   ├── buffer.js                   ← Buffer implementation
│   │   ├── streams.js                  ← Web Streams API
│   │   ├── crypto.js                   ← Web Crypto API
│   │   ├── encoding.js                 ← TextEncoder/TextDecoder
│   │   ├── url.js                      ← URL/URLSearchParams
│   │   ├── events.js                   ← EventTarget/EventEmitter
│   │   ├── abort.js                    ← AbortController/AbortSignal
│   │   └── klyron.js                   ← Klyron-specific APIs
│   ├── ts/
│   │   ├── compiler.ts                 ← TypeScript compiler wrapper
│   │   ├── types.d.ts                  ← Type definitions for runtime
│   │   └── transform.ts                ← Custom transforms
│   └── snapshots/
│       └── snapshot.bin                ← V8 startup snapshot
│
├── adapters/                           ← Framework adapter implementations
│   ├── react/                          ← React SSR/SSG adapter
│   ├── vue/                            ← Vue 3 SSR adapter
│   ├── astro/                          ← Astro adapter
│   ├── next/                           ← Next.js adapter
│   ├── nuxt/                           ← Nuxt 3 adapter
│   ├── svelte/                         ← Svelte adapter
│   ├── sveltekit/                      ← SvelteKit adapter
│   ├── solid/                          ← SolidJS adapter
│   ├── solidstart/                     ← SolidStart adapter
│   ├── qwik/                           ← Qwik City adapter
│   ├── angular/                        ← Angular Universal adapter
│   ├── remix/                          ← Remix adapter
│   ├── preact/                         ← Preact adapter
│   ├── lit/                            ← Lit adapter
│   ├── express/                        ← Express.js adapter
│   ├── fastify/                        ← Fastify adapter
│   ├── nest/                           ← NestJS adapter
│   ├── hono/                           ← Hono adapter
│   ├── koa/                            ← Koa adapter
│   ├── hapi/                           ← Hapi.js adapter
│   ├── adonis/                         ← AdonisJS adapter
│   ├── laravel/                        ← Laravel adapter
│   ├── django/                         ← Django adapter
│   ├── rails/                          ← Rails adapter
│   ├── trpc/                           ← tRPC adapter
│   └── graphql/                        ← GraphQL adapter
│
├── scaffolds/                          ← Scaffold templates (consolidated)
│   ├── frontend/                       ← Full project templates (JS/TS frameworks)
│   │   ├── react/                      ← React + Vite + Router
│   │   ├── vue/                        ← Vue 3 + Pinia + Router
│   │   ├── astro/                      ← Astro + Content Collections
│   │   ├── next/                       ← Next.js App Router
│   │   ├── nuxt/                       ← Nuxt 3 + Modules
│   │   ├── svelte/                     ← Svelte + Vite
│   │   ├── sveltekit/                  ← SvelteKit + Tailwind
│   │   ├── solid/                      ← SolidJS + Vite
│   │   ├── solidstart/                 ← SolidStart
│   │   ├── qwik/                       ← Qwik City
│   │   ├── angular/                    ← Angular Standalone
│   │   ├── remix/                      ← Remix + Vite
│   │   └── preact/                     ← Preact + Vite + Signals
│   │
│   ├── backend/                        ← Full project templates (server frameworks)
│   │   ├── express/                    ← Express.js + MVC
│   │   ├── fastify/                    ← Fastify + Plugins
│   │   ├── nest/                       ← NestJS + Modules
│   │   ├── hono/                       ← Hono + Zod
│   │   ├── koa/                        ← Koa + Middleware
│   │   ├── hapi/                       ← Hapi.js + Plugins
│   │   ├── adonis/                     ← AdonisJS + Lucid
│   │   ├── trpc/                       ← tRPC + React Query
│   │   └── graphql/                    ← GraphQL Yoga + Pothos
│   │
│   ├── laravel/                        ← Laravel stacks
│   │   ├── blade/                      ← Blade-only + Tailwind
│   │   ├── inertia-react/              ← Inertia + React + Tailwind
│   │   ├── inertia-vue/                ← Inertia + Vue + Tailwind
│   │   ├── livewire/                   ← Livewire 3 + Volt
│   │   ├── react/                      ← React SPA backend
│   │   ├── vue/                        ← Vue SPA backend
│   │   ├── next/                       ← Next.js BFF + Laravel API
│   │   ├── astro/                      ← Astro + Laravel API
│   │   └── api/                        ← API-only + Sanctum
│   │
│   ├── polyglot/                       ← Non-JS framework templates
│   │   ├── django/                     ← Django + DRF + Celery
│   │   ├── rails/                      ← Rails + Hotwire + Stimulus
│   │   ├── gin/                        ← Go Gin + GORM
│   │   ├── fiber/                      ← Go Fiber + Ent
│   │   ├── echo/                       ← Go Echo + GORM
│   │   ├── actix/                      ← Rust Actix-web + SQLx
│   │   ├── axum/                       ← Rust Axum + SeaORM
│   │   ├── rocket/                     ← Rust Rocket + Diesel
│   │   ├── leptos/                     ← Rust Leptos + CSR/SSR
│   │   ├── tauri/                      ← Tauri desktop + Svelte
│   │   ├── fastapi/                    ← Python FastAPI + SQLAlchemy
│   │   └── flask/                      ← Python Flask + SQLAlchemy
│   │
│   └── components/                     ← Single-file generators
│       ├── react/                      ← Component/page templates
│       ├── vue/                        ← Component/page templates
│       ├── next/                       ← Page/route templates
│       ├── express/                    ← Route/middleware templates
│       └── nest/                       ← Module/controller templates
│
├── orm/                                ← ORM adapters & schema tools
│   ├── prisma/                         ← Prisma schema + client
│   ├── drizzle/                        ← Drizzle ORM
│   ├── sequelize/                      ← Sequelize models
│   ├── mikroorm/                       ← MikroORM entities
│   ├── typeorm/                        ← TypeORM entities
│   ├── mongoose/                       ← Mongoose schemas
│   ├── kysely/                         ← Kysely dialects
│   └── knex/                           ← Knex migrations
│
├── php/                                ← PHP ecosystem packages
│   ├── laravel/
│   │   ├── v10/                        ← Laravel 10 support
│   │   ├── v11/                        ← Laravel 11 support
│   │   ├── v12/                        ← Laravel 12 support
│   │   └── v13/                        ← Laravel 13 support
│   ├── symfony/                        ← Symfony skeleton
│   ├── codeigniter/                    ← CodeIgniter 4
│   └── wordpress/                      ← WordPress plugin scaffold
│
├── compatibility/                      ← Polyfills & compat layers
│   ├── node/                           ← Node.js polyfills (require, process, Buffer, fs, path, http, crypto, stream)
│   ├── web/                            ← Web API polyfills (fetch, URL, streams, encoding, events, abort)
│   ├── php/                            ← PHP function polyfills
│   └── python/                         ← Python compat shims
│
├── examples/                           ← Example projects
│   ├── 01-hello-world/                 ← Minimal JS/TS
│   ├── 02-http-server/                 ← HTTP server
│   ├── 03-react-app/                   ← React SPA
│   ├── 04-laravel-app/                 ← Laravel fullstack
│   ├── 05-next-app/                    ← Next.js app
│   ├── 06-fullstack/                   ← Fullstack (API + SPA)
│   ├── 07-microservices/               ← Multi-language services
│   └── 08-desktop/                     ← Tauri desktop
│
├── sdk/                                ← Multi-language SDK
│   ├── js/                             ← @klyron/sdk (npm)
│   │   ├── src/
│   │   ├── package.json
│   │   └── README.md
│   ├── ts/                             ← @klyron/types (npm)
│   │   ├── src/
│   │   ├── package.json
│   │   └── README.md
│   ├── rust/                           ← klyron-sdk (crates.io)
│   │   ├── src/
│   │   ├── Cargo.toml
│   │   └── README.md
│   ├── cpp/                            ← klyron-sdk (C++ headers)
│   │   ├── include/
│   │   └── README.md
│   └── php/                            ← klyron/sdk (Packagist)
│       ├── src/
│       ├── composer.json
│       └── README.md
│
├── registry/                           ← Package registry
│   ├── packages/                       ← Local package cache
│   ├── mirrors/                        ← Mirror configs
│   └── metadata/                       ← Package metadata cache
│
├── scripts/                            ← Build/dev/release helper scripts
│   ├── build.sh                        ← Build Klyron
│   ├── test.sh                         ← Run test suite
│   ├── release.sh                      ← Release pipeline
│   └── benchmark.sh                    ← Run benchmarks
│
├── tests/                              ← Integration tests
│   ├── engines/                        ← Engine integration tests
│   ├── scaffolds/                      ← Scaffold output tests
│   ├── runtime/                        ← Runtime behavior tests
│   ├── cli/                            ← CLI e2e tests
│   ├── adapters/                       ← Adapter integration tests
│   └── e2e/                            ← End-to-end tests
│
├── Cargo.toml                          ← Workspace root
├── Cargo.lock
├── package.json                        ← npm package (JS distribution)
├── klyron.toml                         ← Default project config
├── klyron.config.ts                    ← TypeScript config (optional)
├── README.md
├── ROADMAP.md
├── CONTRIBUTING.md
├── CHANGELOG.md
├── CODE_OF_CONDUCT.md
├── SECURITY.md
├── LICENSE
├── .gitignore
├── .editorconfig
├── .env.example
└── rust-toolchain.toml
```

---

## 3. Arsitektur Crate

### 3.1 Klyron CLI (`crates/klyron_cli`)

**Entry point utama.** Semua perintah diregistrasi di sini.
**Depends on:** semua crate lainnya.

```rust
// CLI Structure
pub enum Commands {
    // Runtime
    Run(RunArgs),
    Eval(EvalArgs),
    Repl,
    Shell,

    // Development
    Dev(DevArgs),

    // Build
    Build(BuildArgs),

    // Package management
    Install(InstallArgs),
    Add(AddArgs),
    Remove(RemoveArgs),
    Uninstall(RemoveArgs),
    Update,
    Upgrade,
    Outdated,
    Audit,
    Doctor(DoctorArgs),
    Dedupe,

    // Package.json scripts
    Start,
    TestCommand(TestCliArgs),
    Lint(LintArgs),
    Format(FormatArgs),
    RunScript(RunScriptArgs),

    // Workspace / Monorepo
    Workspace(WorkspaceArgs),

    // Framework scaffold (frontend)
    CreateReact(ScaffoldArgs),
    CreateVue(ScaffoldArgs),
    CreateAstro(ScaffoldArgs),
    CreateNext(ScaffoldArgs),
    CreateNuxt(ScaffoldArgs),
    CreateSveltekit(ScaffoldArgs),
    CreateSolid(ScaffoldArgs),
    CreateQwik(ScaffoldArgs),
    CreateAngular(ScaffoldArgs),
    CreateRemix(ScaffoldArgs),

    // Framework scaffold (backend)
    CreateExpress(ScaffoldArgs),
    CreateFastify(ScaffoldArgs),
    CreateNest(ScaffoldArgs),
    CreateHono(ScaffoldArgs),
    CreateKoa(ScaffoldArgs),
    CreateHapi(ScaffoldArgs),
    CreateAdonis(ScaffoldArgs),

    // Laravel scaffold
    CreateLaravelReact(ScaffoldArgs),
    CreateLaravelVue(ScaffoldArgs),
    CreateLaravelInertiaReact(ScaffoldArgs),
    CreateLaravelInertiaVue(ScaffoldArgs),
    CreateLaravelLivewire(ScaffoldArgs),
    CreateLaravelNext(ScaffoldArgs),
    CreateLaravelAstro(ScaffoldArgs),
    CreateLaravelApi(ScaffoldArgs),

    // Database
    Db(DbCommand),

    // ORM Compatibility
    Prisma(PrismaArgs),
    Drizzle(DrizzleArgs),

    // Testing
    Test(TestArgs),

    // Benchmark
    Bench(BenchArgs),

    // Linter / Formatter / Type Check
    Lint(LintArgs),
    Format(FormatArgs),
    Check(CheckArgs),

    // Plugin System
    Plugin(PluginArgs),

    // Registry
    Publish(PublishArgs),
    Unpublish(UnpublishArgs),
    Login(LoginArgs),
    Logout(LogoutArgs),
    Whoami,
    Search(SearchArgs),
    Info(InfoArgs),

    // Cache
    Cache(CacheArgs),

    // Node Compatibility
    Compat(CompatArgs),

    // Native Modules
    Napi(NapiArgs),

    // Docker
    Docker(DockerArgs),

    // Deployment
    Deploy(DeployArgs),

    // Project Utilities
    Init,
    Doctor(DoctorArgs),
    Info,
    Version,
    Telemetry(TelemetryArgs),
    Config(ConfigArgs),
    Clean,

    // AI (Future / Enterprise)
    Ai(AiArgs),

    // Laravel specific
    Artisan(ArtisanArgs),
    Composer(ComposerArgs),
    Blade(BladeArgs),
    Tinker,

    // Polyglot engines
    Cc(EngineArgs),
    Cxx(EngineArgs),
    Ts(EngineArgs),
    Js(EngineArgs),
    Php(EngineArgs),
    Py(EngineArgs),
    Rb(EngineArgs),
    Go(EngineArgs),
    Rs(EngineArgs),
    Zig(EngineArgs),

    // Utility
    Completions(CompletionsArgs),
    Serve(ServeArgs),
    Bundle(BundleArgs),

    Help,
}
```

### 3.2 Klyron Runtime (`crates/klyron_runtime`)

**Core runtime engine.** Menangani event loop, V8 isolate, module system,
extension system, permission system.

```rust
pub struct KlyronRuntime {
    v8_isolate: v8::Isolate,
    event_loop: tokio::runtime::Runtime,
    module_loader: Arc<dyn ModuleLoader>,
    extensions: Vec<Box<dyn Extension>>,
    permissions: PermissionSet,
}

impl KlyronRuntime {
    pub fn new(config: RuntimeConfig) -> Result<Self>;
    pub fn eval(&self, code: &str) -> Result<Value>;
    pub fn execute_script(&self, filename: &str, source: &str) -> Result<Value>;
    pub fn execute_module(&self, path: &Path) -> Result<Value>;
    pub fn register_extension(&mut self, ext: Box<dyn Extension>);
    pub fn snapshot(&self) -> Vec<u8>;
    pub fn restore(snapshot: &[u8]) -> Result<Self>;
}

pub struct RuntimeConfig {
    pub js_engine: JsEngineKind,    // V8 | QuickJS | Boa | JSC
    pub ts_transpiler: TsEngineKind, // SWC | ESBuild | TSC
    pub enable_typescript: bool,
    pub permissions: PermissionSet,
    pub extensions: Vec<Box<dyn Extension>>,
    pub snapshot: Option<Vec<u8>>,
    pub module_resolution: ModuleConfig,
}

pub enum JsEngineKind { V8, QuickJS, Boa, JSC }
pub enum TsEngineKind { Swc, Esbuild, Tsc }
```

### 3.3 Klyron Engine (`crates/klyron_engine`)

**Abstraksi polyglot engine.** Semua engine (C, C++, PHP, Python, Ruby, Go,
Rust, Zig) mengimplementasi trait yang sama.

```rust
#[async_trait]
pub trait Engine: Send + Sync {
    fn name(&self) -> &'static str;
    fn language(&self) -> &'static str;

    // Execution
    async fn exec(&mut self, input: EngineInput) -> EngineResult;
    async fn eval(&mut self, code: &str) -> EngineResult;
    async fn run_file(&mut self, path: &str) -> EngineResult;

    // Project tools
    async fn dev(&mut self, project: &str, port: Option<u16>) -> EngineResult;
    async fn build(&mut self, project: &str, release: bool) -> EngineResult;
    async fn test(&mut self, project: &str, filter: Option<&str>) -> EngineResult;
    async fn lint(&mut self, code: &str) -> EngineResult;
    async fn format(&mut self, code: &str) -> EngineResult;

    // Project management
    async fn install(&mut self, project: &str) -> EngineResult;
    async fn add(&mut self, packages: &[String], dev: bool) -> EngineResult;
    async fn remove(&mut self, packages: &[String]) -> EngineResult;

    // Lifecycle
    async fn ping(&mut self) -> EngineResult;
    fn reset(&mut self);
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EngineInput {
    pub action: String,
    pub code: Option<String>,
    pub args: Option<String>,
    pub files: Option<Vec<FileSpec>>,
    pub filename: Option<String>,
    pub project: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EngineResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub result: String,
    pub diagnostics: Option<Vec<Diagnostic>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Diagnostic {
    pub file: Option<String>,
    pub line: Option<usize>,
    pub col: Option<usize>,
    pub message: String,
    pub severity: DiagnosticSeverity,
}

pub enum DiagnosticSeverity { Error, Warning, Info, Hint }
```

#### Engine Actions

| Action | C | C++ | TS | JS | PHP | Py | Rb | Go | Rs | Zig |
|--------|---|---|---|---|-----|----|----|----|----|-----|
| exec   | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| eval   | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| compile| ✅ | ✅ | ✅ | - | - | - | - | - | ✅ | ✅ |
| check  | - | - | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| lint   | - | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| format | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| test   | - | - | - | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| file   | - | - | ✅ | ✅ | ✅ | ✅ | ✅ | - | - | - |
| build  | - | - | ✅ | ✅ | - | - | - | ✅ | ✅ | ✅ |
| dev    | - | - | ✅ | ✅ | ✅ | - | ✅ | ✅ | ✅ | ✅ |
| ping   | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |

### 3.4 Klyron Loader (`crates/klyron_loader`)

**Module resolution system.** Menangani import/require, node_modules,
npm packages, import maps, ESM/CJS detection.

```rust
pub struct ModuleLoader {
    resolution_strategy: ResolutionStrategy,
    registry_client: Arc<RegistryClient>,
    cache: Arc<ModuleCache>,
}

impl ModuleLoader {
    pub fn resolve(&self, specifier: &str, referrer: &Path) -> Result<ModuleInfo>;
    pub fn load(&self, info: &ModuleInfo) -> Result<ModuleSource>;
    pub fn resolve_package(&self, name: &str, version: &VersionReq) -> Result<PackageInfo>;
}

pub struct ModuleInfo {
    pub kind: ModuleKind,       // ESM | CJS | JSON | Wasm | Asset
    pub path: PathBuf,
    pub package: Option<PackageInfo>,
}

pub enum ModuleKind { Esm, Cjs, Json, Wasm, Asset, Ts, Tsx, Jsx }

pub enum ResolutionStrategy {
    NodeModules,       // npm-style (default)
    ImportMaps,        // Deno-style
    Bare,              // Direct paths only
}
```

### 3.5 Klyron PM (`crates/klyron_pm`)

**Package management.** Universal package manager yang mendukung multiple
registry backend.

```rust
pub struct PackageManager {
    registries: Vec<Box<dyn RegistryBackend>>,
    lockfile: LockFile,
    cache: PackageCache,
}

#[async_trait]
pub trait RegistryBackend: Send + Sync {
    fn name(&self) -> &'static str;
    fn supports(&self, package: &str) -> bool;
    async fn resolve(&self, name: &str, version: &VersionReq) -> Result<Vec<PackageVersion>>;
    async fn download(&self, pkg: &PackageVersion, dest: &Path) -> Result<()>;
    async fn publish(&self, pkg: &PackageManifest, tarball: &Path) -> Result<()>;
}

pub struct NpmRegistry { base_url: String, token: Option<String> }
pub struct PyPIRegistry { base_url: String }
pub struct RubyGemsRegistry { base_url: String }
pub struct CargoRegistry { index_url: String }
pub struct GoProxyRegistry { base_url: String }
pub struct PackagistRegistry { base_url: String }

impl PackageManager {
    pub fn add(&self, packages: &[PackageReq], dev: bool) -> Result<()>;
    pub fn install(&self) -> Result<()>;
    pub fn remove(&self, packages: &[String]) -> Result<()>;
    pub fn update(&self, packages: &[String]) -> Result<()>;
    pub fn outdated(&self) -> Result<Vec<OutdatedPackage>>;
}
```

### 3.6 Crates Lainnya — Ringkasan API

| Crate | Struct Utama | Method Kunci |
|-------|-------------|--------------|
| `klyron_bundler` | `Bundler` | `bundle(entry, output, config)` |
| `klyron_transpiler` | `Transpiler` | `transpile(source, filename, opts)` |
| `klyron_watcher` | `FileWatcher` | `watch(paths, cb, opts)`, `debounce(ms)`, `close()` |
| `klyron_config` | `ConfigManager` | `load(path)`, `merge(cli, file, defaults)`, `validate()` |
| `klyron_test` | `TestRunner` | `run(dir, filter, opts)` |
| `klyron_linter` | `Linter` | `lint(code, filename)`, `fix(code)` |
| `klyron_formatter` | `Formatter` | `format(code, filename)` |
| `klyron_process` | `ProcessManager` | `spawn(cmd, args)`, `exec(cmd)`, `kill(pid)` |
| `klyron_logger` | `Logger` | `info/warn/error/debug(msg)`, `diagnostic(d)` |
| `klyron_telemetry` | `Telemetry` | `event(name, props)`, `metric(name, value)`, `flush()` |
| `klyron_updater` | `Updater` | `check_version()`, `download()`, `install()` |
| `klyron_shell` | `ShellRepl` | `start()`, `eval(line)`, `history()`, `autocomplete()` |
| `klyron_template` | `TemplateEngine` | `render(tpl, vars)`, `scaffold(template, dir, vars)` |
| `klyron_napi` | `NapiLoader` | `load_module(path)`, `call_function(...)` |
| `klyron_node` | `NodeCompat` | `polyfill_require()`, `polyfill_process()` |
| `klyron_web` | `WebApi` | `fetch(req)`, `WebSocket(url)` |
| `klyron_http` | `HttpServer` | `serve(handler, addr)`, `request(req)` |
| `klyron_fs` | `FileSystem` | `read/write/copy/move/remove/watch` |
| `klyron_dns` | `DnsResolver` | `resolve(hostname, record_type)` |
| `klyron_crypto` | `CryptoProvider` | `hash/encrypt/decrypt/sign/verify/random` |
| `klyron_sqlite` | `SqliteDb` | `open/query/exec/transaction` |
| `klyron_postgres` | `PostgresDb` | `connect/query/exec/pool` |
| `klyron_mysql` | `MySqlDb` | `connect/query/exec/pool` |
| `klyron_cache` | `CacheManager` | `get/set/delete/clear/tags` |
| `klyron_plugin` | `PluginManager` | `load/unload/reload/events` |
| `klyron_workspace` | `Workspace` | `init/add/remove/exec/scripts` |
| `klyron_registry` | `RegistryClient` | `search/info/download/publish/login` |
| `klyron_bench` | `BenchmarkRunner` | `run(label, fn, iterations)` |
| `klyron_compat` | `CompatChecker` | `check(framework, version)` |
| `klyron_docker` | `DockerManager` | `generate/build/run/compose` |
| `klyron_deploy` | `Deployment` | `deploy(platform, dir, opts)` |
| `klyron_ai` | `AiEngine` | `generate/optimize/review/docs/test/migrate` |
| `klyron_utils` | `UtilFunctions` | `path/semver/hash/shell/url/json` |

---

## 4. JavaScript Engine Strategy

### 4.1 Engine Matrix

| Engine | Bahasa | Runtime | Performa | Memory | Bundle | Status |
|--------|--------|---------|----------|--------|--------|--------|
| **V8** | C++ | Production | Sangat Cepat | Tinggi | Besar | ✅ Active (default) |
| **Boa** | Rust | Experimental | Sedang | Rendah | Kecil | 🔧 Development |
| **QuickJS** | C | Production | Cepat | Sangat Rendah | Sangat Kecil | 🔧 Development |
| **JSC** | C++ | Future | Sangat Cepat | Tinggi | Besar | ⬜ Planned |

### 4.2 Konfigurasi

```toml
[engine]
js = "v8"              # default, fallback: boa → quickjs
ts = "swc"             # default, fallback: esbuild → tsc

[engine.v8]
snapshot = true
max_heap_size = "512mb"
initial_heap = "64mb"

[engine.boa]
features = ["js", "ts"]
optimization_level = "aggressive"
```

### 4.3 Engine Selection Logic

```
klyron run main.ts
  │
  ├── is TS? ──YES──→ klyron_transpiler::transpile(main.ts) → JS
  │
  ├── JS Engine Selection:
  │   ├── klyron.toml [engine.js] = "v8"       → V8 (via deno_core)
  │   ├── klyron.toml [engine.js] = "boa"      → Boa (Rust native)
  │   ├── klyron.toml [engine.js] = "quickjs"  → QuickJS (C binding)
  │   └── klyron.toml [engine.js] = "jsc"      → JSC (C++ binding)
  │
  ├── Permission Check:
  │   ├── Filesystem? → check allow_read/allow_write
  │   ├── Network?    → check allow_net
  │   ├── Env?        → check allow_env
  │   └── Subprocess? → check allow_run
  │
  └── Execute → output → stdout/stderr
```

---

## 5. Polyglot Engine Protocol

### 5.1 Wire Protocol

Semua engine berkomunikasi via **stdin/stdout JSON-line protocol**.

```
▶ Request (stdin):
{"action":"exec","code":"print('hello')","args":"--optimize=2","files":[{"name":"lib.rs","content":"..."}],"filename":"main.py"}

◀ Response (stdout):
{"stdout":"hello\n","stderr":"","exit_code":0,"result":"ok","diagnostics":[]}
```

### 5.2 Engine Specification Per Bahasa

#### C Engine (`engine.c`)
- **Compiled**: Ya (gcc/clang ke binary)
- **Compiler**: `cc` (gcc, clang, tcc)
- **Standard default**: gnu11
- **Optimasi default**: -O2 -Wall -Wextra -Werror
- **Library default**: -lm -pthread
- **Actions**: exec, eval, compile, format, ping
- **Args flags**: --std=, -O, -W, -l, -I, -D, -g, -fsanitize=
- **Timeout**: 120s compile, 30s run

#### C++ Engine (`engine.cpp`)
- **Compiled**: Ya (g++/clang++ ke binary)
- **Compiler**: g++ (clang++, c++)
- **Standard default**: c++20
- **Ekstensi**: .cpp, .cxx, .cc, .hpp, .hxx, .ixx
- **Actions**: exec, eval, compile, lint, format, ping
- **Fitur**: Coroutine detection, modules (-fmodules), LTO (-flto)
- **Args flags**: --std=, -O, -W, -l, -I, -D, -g, -fsanitize=, -flto, -fmodules

#### TypeScript Engine (`engine.ts`)
- **Runtime**: Node.js via `npx tsx`
- **Transpiler**: Internal TS stripper + fallback esbuild/Deno
- **Actions**: exec, eval, compile, check, file, lint, format, build, dev, watch, ping
- **Fitur**: tsconfig parsing, declaration emit, composite projects, path mapping, watch mode
- **Format**: Prettier via npx
- **Lint**: ESLint via npx

#### JavaScript Engine (`engine.js`)
- **Runtime**: Node.js
- **ESM/CJS**: Auto-detect via file extension (.mjs/.cjs/.js)
- **Actions**: exec, eval, file, lint, format, check, build, test, ping
- **Format**: Prettier via npx
- **Lint**: ESLint via npx

#### PHP Engine (`engine.php`)
- **Runtime**: PHP interpreter
- **Actions**: exec, eval, file, check, lint, format, test, ping
- **Blade**: Compiler built-in (regex-based)
- **Artisan**: 50+ artisan make:commands, horizon, telescope, sail, pulse
- **Composer**: Wrapper penuh
- **Format**: php-cs-fixer / Laravel Pint
- **Lint**: php -l
- **Test**: PHPUnit / Pest

#### Python Engine (`engine.py`)
- **Runtime**: python3
- **Actions**: exec, eval, file, check, lint, format, test, ping
- **Django**: manage.py commands
- **Package manager**: pip, pipenv, poetry
- **Format**: black (default), autopep8, yapf
- **Lint**: flake8 (default), ruff, pylint
- **Test**: pytest (default), unittest

#### Ruby Engine (`engine.rb`)
- **Runtime**: ruby
- **Actions**: exec, eval, file, check, lint, format, test, ping
- **Rails**: runner, console, rake tasks
- **Bundler**: install, update, exec
- **Format**: rubocop -A
- **Lint**: rubocop
- **Test**: RSpec (default), Minitest

#### Go Engine (`engine.go`)
- **Compiled**: Ya (go build)
- **Runtime**: go toolchain
- **Actions**: exec, eval, check, build, test, lint, format, ping
- **Fitur**: Multi-file, go mod init, go test, go get, go fmt, go vet
- **Format**: gofmt
- **Lint**: go vet, staticcheck

#### Rust Engine (`engine.rs`)
- **Compiled**: Ya (rustc)
- **Runtime**: rustc + cargo
- **Actions**: exec, eval, check, build, test, fmt, clippy, scaffold, ping
- **Fitur**: Multi-file, TempDir, 1MB output truncation, Drop cleanup
- **Scaffold templates**: actix-web, axum, rocket, cli, lib, lambda, tauri, leptos, yew, dioxus, warp, tide, poem
- **Format**: rustfmt
- **Lint**: clippy

#### Zig Engine (`engine.zig`)
- **Compiled**: Ya (zig build-exe / zig build)
- **Runtime**: zig toolchain
- **Actions**: exec, eval, build, test, fmt, ping
- **Fitur**: Multi-file, zig build, zig test, zig fmt, arena allocator

---

## 6. Framework Adapters

### 6.1 Adapter Contract

Setiap adapter mengimplementasi trait berikut:

```rust
#[async_trait]
pub trait FrameworkAdapter: Send + Sync {
    fn name(&self) -> &'static str;
    fn detect(&self, dir: &Path) -> bool;

    // Project lifecycle
    async fn dev(&self, dir: &Path, port: Option<u16>) -> Result<()>;
    async fn build(&self, dir: &Path, opts: BuildOptions) -> Result<()>;
    async fn test(&self, dir: &Path, filter: Option<&str>) -> Result<()>;
    async fn lint(&self, dir: &Path) -> Result<()>;
    async fn format(&self, dir: &Path) -> Result<()>;

    // Scaffolding
    async fn scaffold(&self, name: &str, dir: &Path, options: ScaffoldOptions) -> Result<()>;
}
```

### 6.2 Adapter Matrix

| Framework | detect() | dev() | build() | test() | lint() | format() | scaffold() |
|-----------|----------|-------|---------|--------|--------|----------|------------|
| React | `vite.config.*` | `vite` | `vite build` | `vitest` | `eslint` | `prettier` | ✅ 18 files |
| Next.js | `next.config.*` | `next dev` | `next build` | `vitest` | `next lint` | `prettier` | ✅ 22 files |
| Vue | `vite.config.*` | `vite` | `vite build` | `vitest` | `eslint` | `prettier` | ✅ 18 files |
| Nuxt | `nuxt.config.*` | `nuxt dev` | `nuxt build` | `vitest` | `eslint` | `prettier` | ✅ 18 files |
| Astro | `astro.config.*` | `astro dev` | `astro build` | `astro check` | `eslint` | `prettier` | ✅ 16 files |
| Svelte | `vite.config.*` | `vite` | `vite build` | `vitest` | `eslint` | `prettier` | ✅ |
| SvelteKit | `svelte.config.*` | `vite dev` | `vite build` | `vitest` | `eslint` | `prettier` | ✅ 18 files |
| Solid | `vite.config.*` | `vite` | `vite build` | `vitest` | `eslint` | `prettier` | ✅ |
| SolidStart | `solid.config.*` | `solid start` | `solid build` | `vitest` | `eslint` | `prettier` | ⬜ |
| Qwik | `qwik.config.*` | `qwik dev` | `qwik build` | `vitest` | `eslint` | `prettier` | ⬜ |
| Angular | `angular.json` | `ng serve` | `ng build` | `ng test` | `ng lint` | `prettier` | ⬜ |
| Remix | `remix.config.*` | `remix dev` | `remix build` | `vitest` | `eslint` | `prettier` | ✅ 17 files |
| Preact | `vite.config.*` | `vite` | `vite build` | `vitest` | `eslint` | `prettier` | ✅ |
| Lit | `vite.config.*` | `vite` | `vite build` | `web-test` | `eslint` | `prettier` | ✅ |
| Express | `package.json` | `node --watch` | - | `jest` | `eslint` | `prettier` | ✅ |
| Fastify | `package.json` | `node --watch` | - | `tap` | `eslint` | `prettier` | ✅ |
| NestJS | `nest-cli.json` | `nest start` | `nest build` | `jest` | `eslint` | `prettier` | ⬜ |
| Hono | `package.json` | `node --watch` | - | `vitest` | `eslint` | `prettier` | ⬜ |
| Koa | `package.json` | `node --watch` | - | `jest` | `eslint` | `prettier` | ⬜ |
| Hapi | `package.json` | `node --watch` | - | `jest` | `eslint` | `prettier` | ⬜ |
| AdonisJS | `adonisrc.*` | `node ace serve` | `node ace build` | `japa` | `eslint` | `prettier` | ✅ |
| tRPC | `package.json` | `node --watch` | - | `vitest` | `eslint` | `prettier` | ⬜ |
| GraphQL | `package.json` | `node --watch` | - | `jest` | `eslint` | `prettier` | ⬜ |
| Laravel | `artisan` | `artisan serve` | `artisan build` | `phpunit` | `pint` | `pint` | ✅ 40 files |
| Django | `manage.py` | `manage.py runserver` | - | `pytest` | `ruff` | `black` | ✅ 24 files |
| Rails | `bin/rails` | `rails server` | `rails assets` | `rspec` | `rubocop` | `rubocop -A` | ✅ 32 files |

### 6.3 Template File Count

| Template | Files | Dir | Status |
|----------|-------|-----|--------|
| Next.js | 22 | 8 | ✅ Production |
| React | 18 | 5 | ✅ Production |
| Vue | 18 | 6 | ✅ Production |
| SvelteKit | 18 | 5 | ✅ Production |
| Nuxt | 18 | 6 | ✅ Production |
| Remix | 17 | 5 | ✅ Production |
| Solid | 12 | 3 | ✅ Standard |
| Preact | 10 | 3 | ✅ Standard |
| Lit | 10 | 2 | ✅ Standard |
| Astro | 16 | 5 | ✅ Standard |
| Qwik | 14 | 4 | ⬜ |
| Angular | 20 | 7 | ⬜ |
| Express | 8 | 2 | ✅ Standard |
| Fastify | 8 | 2 | ✅ Standard |
| NestJS | 15 | 5 | ⬜ |
| Hono | 8 | 2 | ⬜ |
| AdonisJS | 14 | 4 | ✅ Standard |
| Laravel | 40 | 19 | ✅ Production |
| Django | 24 | 8 | ✅ Production |
| Rails | 32 | 12 | ✅ Production |
| Go Gin | 4 | 2 | ✅ Standard |
| Go Fiber | 3 | 1 | ✅ Standard |
| Go Echo | 3 | 1 | ✅ Standard |
| Actix | 3 | 2 | ✅ Standard |
| Axum | 3 | 2 | ✅ Standard |
| Rocket | 3 | 2 | ✅ Standard |
| Leptos | 4 | 2 | ✅ Standard |
| Tauri | 5 | 3 | ✅ Standard |
| Gin | 4 | 2 | ✅ Standard |
| FastAPI | 10 | 3 | ✅ Standard |
| Flask | 10 | 3 | ✅ Standard |

---

## 7. Laravel Ecosystem

### 7.1 Version Support

| Versi | PHP Min | Support | Fitur Kunci |
|-------|---------|---------|-------------|
| 10 | 8.1 | ✅ | LTS, Native type declarations, Process layer |
| 11 | 8.2 | ✅ | No more Http Kernel,简化 bootstrap, SQLite by default |
| 12 | 8.3 | ⬜ | Reverb, Health routing, Graceful encryption |
| 13 | 8.4 | ⬜ | TBD |

### 7.2 Stack Matrix

```
┌──────────────────────────────────────────────────────────────┐
│                    Laravel Application                       │
├──────────────────────────────────────────────────────────────┤
│  Frontend Stack:                                             │
│  ┌─────────┬──────────┬────────┬────────┬────────┬────────┐ │
│  │ Blade   │ Inertia  │ Inertia│ Livewire│ React  │ Vue    │ │
│  │ Tailwind│ +React   │ +Vue   │ +Volt  │ SPA    │ SPA    │ │
│  └─────────┴──────────┴────────┴────────┴────────┴────────┘ │
├──────────────────────────────────────────────────────────────┤
│  API / BFF:                                                  │
│  ┌──────────┬──────────┬──────────┬──────────┬─────────────┐ │
│  │ REST API │ Next.js  │ Astro    │ GraphQL  │ Custom SSR  │ │
│  │ Sanctum  │ BFF      │ +Laravel │ +Lighthouse│ +Vite     │ │
│  └──────────┴──────────┴──────────┴──────────┴─────────────┘ │
├──────────────────────────────────────────────────────────────┤
│  Tools:                                                      │
│  ┌────────┬─────────┬──────────┬────────┬────────┬─────────┐│
│  │Horizon │Telescope│  Sail    │ Pulse  │Pennant │ Reverb  ││
│  │Queue   │ Debug   │ Docker   │Health  │Features│WebSocket││
│  └────────┴─────────┴──────────┴────────┴────────┴─────────┘│
└──────────────────────────────────────────────────────────────┘
```

### 7.3 Artisan Command Wrapper

Semua command artisan diakses via `klyron artisan <command>`:

```bash
# Generate commands
klyron artisan make:controller PostController --api
klyron artisan make:model Post -m
klyron artisan make:migration create_posts_table
klyron artisan make:seeder PostSeeder
klyron artisan make:factory PostFactory
klyron artisan make:resource PostResource
klyron artisan make:request StorePostRequest
klyron artisan make:mail WelcomeMail
klyron artisan make:notification PostCreated
klyron artisan make:listener SendWelcomeEmail
klyron artisan make:event PostCreated
klyron artisan make:job ProcessPost
klyron artisan make:command ImportPosts
klyron artisan make:rule ValidSlug
klyron artisan make:cast UserIdCast
klyron artisan make:channel PostChannel
klyron artisan make:observer PostObserver
klyron artisan make:policy PostPolicy
klyron artisan make:provider PostServiceProvider
klyron artisan make:action CreatePost    # Laravel 11+
klyron artisan make:enum PostStatus      # Laravel 11+
klyron artisan make:class PostFormatter   # Laravel 11+

# Database
klyron artisan migrate
klyron artisan migrate:fresh --seed
klyron artisan migrate:rollback --step=3
klyron artisan db:seed --class=DatabaseSeeder
klyron artisan db:wipe --drop-types

# Cache
klyron artisan cache:clear
klyron artisan config:clear
klyron artisan route:clear
klyron artisan view:clear
klyron artisan optimize:clear
klyron artisan optimize

# Queue
klyron artisan queue:work --queue=high,low
klyron artisan queue:listen
klyron artisan queue:restart
klyron artisan queue:failed
klyron artisan queue:retry all

# Horizon
klyron artisan horizon
klyron artisan horizon:install
klyron artisan horizon:pause
klyron artisan horizon:continue
klyron artisan horizon:terminate
klyron artisan horizon:status
klyron artisan horizon:snapshot

# Telescope
klyron artisan telescope:install
klyron artisan telescope:prune --hours=48
klyron artisan telescope:clear

# Sail
klyron sail up -d
klyron sail down
klyron sail artisan migrate
klyron sail npm run dev

# Schedule
klyron artisan schedule:run
klyron artisan schedule:list
klyron artisan schedule:work        # Laravel 11+

# Route
klyron artisan route:list
klyron artisan route:cache
klyron artisan route:clear

# Storage
klyron artisan storage:link
klyron artisan storage:unlink
```

### 7.4 Laravel Scaffold Structure

```
my-laravel-app/
├── app/
│   ├── Http/
│   │   ├── Controllers/
│   │   │   ├── Controller.php
│   │   │   ├── AuthController.php
│   │   │   ├── DashboardController.php
│   │   │   └── PostController.php
│   │   └── Livewire/
│   │       └── Counter.php
│   ├── Models/
│   │   ├── User.php          (HasRoles, HasApiTokens)
│   │   └── Post.php
│   ├── Providers/
│   │   ├── AppServiceProvider.php
│   │   └── TelescopeServiceProvider.php
│   └── Http/Middleware/
│       └── AdminMiddleware.php
├── bootstrap/
│   └── app.php
├── config/
│   ├── app.php
│   ├── cache.php
│   ├── database.php          (pgsql, mysql, sqlite)
│   ├── sanctum.php
│   ├── permission.php
│   └── telescope.php
├── database/
│   ├── migrations/
│   │   ├── 2014_10_12_000000_create_users_table.php
│   │   ├── 2014_10_12_100000_create_password_resets_table.php
│   │   └── 2024_01_01_000001_create_posts_table.php
│   ├── seeders/
│   │   ├── DatabaseSeeder.php
│   │   └── UserSeeder.php
│   └── factories/
│       ├── UserFactory.php
│       └── PostFactory.php
├── resources/
│   ├── views/
│   │   ├── layouts/
│   │   │   └── app.blade.php
│   │   ├── welcome.blade.php
│   │   ├── dashboard.blade.php
│   │   ├── auth/
│   │   │   └── login.blade.php
│   │   └── livewire/
│   │       └── counter.blade.php
│   └── css/
│       └── app.css
├── routes/
│   ├── web.php
│   └── api.php
├── public/
│   ├── index.php
│   └── .htaccess
├── storage/
│   └── logs/
│       └── .gitkeep
├── .env
├── .env.example
├── .gitignore
├── artisan
├── composer.json
├── docker-compose.yml
├── Dockerfile
├── package.json
├── phpunit.xml
├── vite.config.js
├── tailwind.config.js
└── postcss.config.js
```

---

## 8. ORM Layer

### 8.1 Supported ORMs

| ORM | Bahasa | Dialek DB | Schema | Migrations | Seeding | Status |
|-----|--------|-----------|--------|------------|---------|--------|
| Prisma | TS/JS | postgres, mysql, sqlite, sqlserver, mongodb | `schema.prisma` | `prisma migrate` | `prisma seed` | ⬜ |
| Drizzle | TS/JS | postgres, mysql, sqlite, turso | `drizzle.config.ts` | `drizzle-kit` | manual | ⬜ |
| TypeORM | TS/JS | postgres, mysql, sqlite, sqlserver, mongodb | Entity decorators | `typeorm migration` | manual | ⬜ |
| MikroORM | TS/JS | postgres, mysql, sqlite, mongodb | Entity decorators | `mikro-orm migration` | manual | ⬜ |
| Sequelize | JS | postgres, mysql, sqlite, sqlserver, mariadb | Model definitions | `sequelize migration` | `sequelize seed` | ⬜ |
| Mongoose | JS | mongodb | Schema definitions | manual | manual | ⬜ |
| Kysely | TS/JS | postgres, mysql, sqlite | TypeScript types | manual | manual | ⬜ |
| Knex | JS | postgres, mysql, sqlite, sqlserver | Migration files | `knex migrate` | `knex seed` | ⬜ |

### 8.2 ORM Integration Commands

```bash
klyron db init                    # Init ORM (auto-detect)
klyron db migrate                 # Run migrations
klyron db migrate:make  Name      # Create migration
klyron db seed                    # Run seeders
klyron db seed:make  Name         # Create seeder
klyron db rollback                # Rollback last batch
klyron db status                  # Migration status
klyron db studio                  # Open DB studio (Prisma/Drizzle)

# With specific ORM
klyron db --orm=prisma migrate
klyron db --orm=drizzle generate
klyron db --orm=typeorm migration:run
```

### 8.3 Auto-Detection

```
Detect ORM by config files:
  schema.prisma              → Prisma
  drizzle.config.ts          → Drizzle
  ormconfig.json             → TypeORM
  mikro-orm.config.ts        → MikroORM
  .sequelizerc               → Sequelize
  knexfile.ts                → Knex
  mongoose (in package.json) → Mongoose
  kysely (in package.json)   → Kysely
```

---

## 9. Template & Scaffold System

### 9.1 Template Format

Setiap template adalah folder dengan file-file lengkap:

```
scaffolds/laravel/inertia-react/
├── meta.toml                              ← Template metadata
├── vars.toml                              ← Variable definitions
├── files/
│   ├── composer.json                      ← File dengan variables
│   ├── package.json
│   ├── .env.example
│   ├── routes/web.php
│   └── ... (all project files)
└── hooks/
    ├── pre-scaffold.sh                    ← Before hooks
    └── post-scaffold.sh                   ← After hooks (e.g. composer install)
```

### 9.2 Variable Substitution

```toml
# vars.toml
[variables]
name = { type = "string", prompt = "Project name", default = "my-app" }
stack = { type = "select", prompt = "Stack", options = ["blade", "livewire", "inertia"] }
db = { type = "select", prompt = "Database", options = ["sqlite", "mysql", "pgsql"], default = "sqlite" }
port = { type = "number", prompt = "Dev port", default = 3000 }
```

Variable dalam file ditulis dengan `{{ name }}` atau `{{ name | upper }}`.

### 9.3 Template Registry

```
~/.klyron/scaffolds/                       ← Local templates
  ├── community/                           ← Community templates
  └── official/                            ← Official templates

Registry URL:
  https://registry.klyron.dev/scaffolds/   ← Official registry
```

### 9.4 Custom Template CLI

```bash
klyron create my-app --template=laravel/inertia-react
klyron create my-app --template=github:user/repo
klyron create my-app --template=./path/to/template
klyron template list
klyron template install laravel/inertia-react
klyron template publish ./my-template
```

---

## 10. Registry & Package Management

### 10.1 Supported Registries

| Registry | URL Default | Package Format | Auth |
|----------|-------------|----------------|------|
| npm | `registry.npmjs.org` | `.tgz` | Token (Bearer) |
| PyPI | `pypi.org` | `.tar.gz`/`.whl` | Token |
| RubyGems | `rubygems.org` | `.gem` | API key |
| crates.io | `crates.io` | `.crate` | Token |
| Go Proxy | `proxy.golang.org` | `.zip` | None |
| Packagist | `repo.packagist.org` | `.zip` | Token |
| Custom | Kustom | Kustom | Kustom |

### 10.2 Package Management Commands

```bash
# Add packages
klyron add react                    # npm
klyron add laravel/framework        # composer
klyron add django                   # pip
klyron add rails                    # gem
klyron add serde                    # cargo
klyron add github.com/gin-gonic/gin # go
klyron add react --dev              # dev dependency

# Install
klyron install                      # Auto-detect and install all deps

# Remove
klyron remove axios                 # Remove from any registry

# Other
klyron outdated                     # Check outdated packages
klyron update                       # Update all packages
klyron audit                        # Security audit
klyron login [registry]             # Login to registry
klyron publish                      # Publish package
klyron unpublish                    # Unpublish package
```

### 10.3 Multi-Project Context

```bash
# In a Laravel project with Node frontend:
klyron add laravel/breeze           # → composer require laravel/breeze
klyron add @inertiajs/inertia-react # → npm install @inertiajs/inertia-react
klyron install                       # → composer install && npm install
```

---

## 11. SDK Multi-Bahasa

### 11.1 SDK API

Each SDK exposes these capabilities:

```typescript
// TypeScript SDK (@klyron/types)
interface KlyronRuntime {
  // System
  version: string;
  os: 'linux' | 'macos' | 'windows';
  arch: 'x64' | 'arm64';

  // Env
  env: { get(key: string): string | undefined; set(key: string, value: string): void };
  exit(code?: number): never;

  // FS
  fs: {
    read(path: string): Promise<Uint8Array>;
    write(path: string, data: Uint8Array): Promise<void>;
    exists(path: string): boolean;
    mkdir(path: string, recursive?: boolean): void;
    remove(path: string): void;
    watch(path: string, cb: (event: FsEvent) => void): () => void;
  };

  // HTTP
  http: {
    fetch(request: Request | string, init?: RequestInit): Promise<Response>;
    serve(handler: (req: Request) => Response | Promise<Response>, port?: number): void;
  };

  // Crypto
  crypto: {
    hash(algorithm: string, data: Uint8Array): Uint8Array;
    randomBytes(size: number): Uint8Array;
    uuid(): string;
  };

  // Process
  process: {
    spawn(cmd: string, args?: string[]): ChildProcess;
    exec(cmd: string): Promise<{ stdout: string; stderr: string; exitCode: number }>;
  };

  // Database
  db: {
    query(sql: string, params?: any[]): Promise<QueryResult>;
    transaction<T>(fn: (tx: Transaction) => Promise<T>): Promise<T>;
  };

  // Cache
  cache: {
    get<T>(key: string): Promise<T | undefined>;
    set<T>(key: string, value: T, ttl?: number): Promise<void>;
    delete(key: string): Promise<void>;
  };

  // Metrics & Tracing
  metrics: {
    counter(name: string, value?: number): void;
    gauge(name: string, value: number): void;
    timing(name: string, duration: number): void;
  };
}

declare const klyron: KlyronRuntime;
```

### 11.2 SDK Per Bahasa

| Bahasa | Package Name | Distribusi |
|--------|-------------|------------|
| JavaScript | `@klyron/sdk` | npm |
| TypeScript | `@klyron/types` | npm |
| Rust | `klyron-sdk` | crates.io |
| C++ | `klyron/sdk` | Header-only, vcpkg |
| PHP | `klyron/sdk` | Packagist |

---

## 12. CLI Command Reference

### 12.1 Command Tree

```
klyron
│
├── Runtime
│   ├── run <file> [args]...            Execute file (app.js, app.ts, app.tsx, app.jsx)
│   ├── repl                            Start interactive REPL
│   ├── eval <code>                     Evaluate expression (e.g. `console.log('hello')`)
│   └── shell                           Start system shell
│
├── Development
│   ├── dev [entry] [--watch] [--hot]   Start dev server with HMR
│   │   [--host] [--port]
│   └── dev src/index.ts                Dev server with custom entry
│
├── Build
│   ├── build [entry] [--minify]        Build project
│   │   [--sourcemap] [--target browser|node|edge|lambda]
│   └── build src/index.ts              Build with custom entry
│
├── Package Manager
│   ├── install [packages]...           Install all or specific packages
│   ├── add <packages>...               Alias for install
│   ├── remove <packages>...            Remove packages
│   ├── uninstall <packages>...         Alias for remove
│   ├── update                          Update all dependencies
│   ├── upgrade                         Upgrade Klyron itself
│   ├── outdated                        Check outdated packages
│   ├── audit                           Security audit
│   ├── doctor                          Package health check
│   └── dedupe                          Deduplicate dependencies
│
├── Package.json Scripts
│   ├── start                           Run `npm start`
│   ├── test                            Run `npm test`
│   ├── lint                            Run `npm run lint`
│   ├── format                          Run `npm run format`
│   └── run <script>                    Run any npm script
│
├── Workspace / Monorepo
│   ├── workspace init                  Initialize workspace
│   ├── workspace list                  List workspace members
│   ├── workspace add <name>            Add workspace member
│   ├── workspace remove <name>         Remove workspace member
│   └── workspace run <script>          Run script across all members
│
├── Framework Generator (Frontend)
│   ├── create react <name>             Scaffold React + Vite + Router
│   ├── create vue <name>               Scaffold Vue 3 + Pinia + Router
│   ├── create astro <name>             Scaffold Astro + Content Collections
│   ├── create next <name>              Scaffold Next.js App Router
│   ├── create nuxt <name>              Scaffold Nuxt 3 + Modules
│   ├── create sveltekit <name>         Scaffold SvelteKit + Tailwind
│   ├── create solid <name>             Scaffold SolidJS + Vite
│   ├── create qwik <name>              Scaffold Qwik City
│   ├── create angular <name>           Scaffold Angular Standalone
│   └── create remix <name>             Scaffold Remix + Vite
│
├── Framework Generator (Backend)
│   ├── create express <name>           Scaffold Express.js + MVC
│   ├── create fastify <name>           Scaffold Fastify + Plugins
│   ├── create nest <name>              Scaffold NestJS + Modules
│   ├── create hono <name>              Scaffold Hono + Zod
│   ├── create koa <name>               Scaffold Koa + Middleware
│   ├── create hapi <name>              Scaffold Hapi.js + Plugins
│   └── create adonis <name>            Scaffold AdonisJS + Lucid
│
├── Laravel Integration
│   ├── create laravel-react <name>     Laravel + React SPA + Tailwind
│   ├── create laravel-vue <name>       Laravel + Vue SPA + Tailwind
│   ├── create laravel-inertia-react    Laravel + Inertia + React + Tailwind
│   ├── create laravel-inertia-vue      Laravel + Inertia + Vue + Tailwind
│   ├── create laravel-livewire <name>  Laravel + Livewire 3 + Volt
│   ├── create laravel-next <name>      Laravel + Next.js BFF + API
│   ├── create laravel-astro <name>     Laravel + Astro + API
│   └── create laravel-api <name>       Laravel API-only + Sanctum
│
├── Database
│   ├── db init                         Initialize ORM (auto-detect)
│   ├── db generate                     Generate ORM client/types
│   ├── db migrate                      Run pending migrations
│   ├── db push                         Push schema to database
│   ├── db pull                         Pull schema from database
│   ├── db seed                         Run database seeders
│   ├── db reset                        Reset database (drop + migrate + seed)
│   └── db studio                       Open database studio (Prisma/Drizzle)
│
├── ORM Compatibility
│   ├── prisma generate                 Prisma client generation
│   ├── prisma migrate                  Prisma migrations
│   ├── prisma studio                   Prisma Studio
│   ├── prisma db push                  Prisma schema push
│   ├── drizzle generate                Drizzle schema generation
│   ├── drizzle migrate                 Drizzle migrations
│   └── drizzle studio                  Drizzle Studio
│
├── Testing
│   ├── test                            Run all tests
│   ├── test watch                      Run tests in watch mode
│   ├── test coverage                   Run tests with coverage
│   ├── test ui                         Open test UI (vitest)
│   ├── test e2e                        Run end-to-end tests
│   ├── test unit                       Run unit tests
│   └── test integration                Run integration tests
│
├── Benchmark
│   ├── bench                           Run all benchmarks
│   ├── bench runtime                   Benchmark runtime performance
│   ├── bench http                      Benchmark HTTP throughput
│   ├── bench memory                    Benchmark memory usage
│   └── bench startup                   Benchmark cold/warm startup
│
├── Linter
│   ├── lint                            Lint project (auto-detect)
│   ├── lint <dir>                      Lint specific directory
│   └── lint fix                        Lint with auto-fix
│
├── Formatter
│   ├── format                          Format project (auto-detect)
│   ├── format <dir>                    Format specific directory
│   └── format --write                  Format and write in-place
│
├── Type Checking
│   ├── check                           Run all checks
│   ├── check types                     TypeScript type checking
│   └── check project                   Full project validation
│
├── Plugin System
│   ├── plugin install <name>           Install plugin
│   ├── plugin remove <name>            Remove plugin
│   ├── plugin list                     List installed plugins
│   ├── plugin update                   Update all plugins
│   └── plugin create <name>            Scaffold new plugin
│
├── Registry
│   ├── publish                         Publish current package
│   ├── unpublish <name>                Unpublish package
│   ├── login [registry]                Login to registry
│   ├── logout [registry]               Logout from registry
│   ├── whoami                          Show current user
│   ├── search <query>                  Search packages
│   └── info <package>                  Show package info
│
├── Cache
│   ├── cache clean                     Clear all cache
│   ├── cache prune                     Prune expired cache
│   └── cache info                      Show cache stats
│
├── Node Compatibility
│   ├── compat check                    Check Node.js compatibility
│   ├── compat react                    Check React compat
│   ├── compat next                     Check Next.js compat
│   ├── compat astro                    Check Astro compat
│   ├── compat nest                     Check NestJS compat
│   └── compat prisma                   Check Prisma compat
│
├── Native Modules
│   ├── napi build                      Build N-API native module
│   ├── napi generate                   Generate N-API bindings
│   └── napi test                       Test N-API native module
│
├── Docker
│   ├── docker init                     Generate Dockerfile + docker-compose
│   ├── docker build                    Build Docker image
│   └── docker run                      Run Docker container
│
├── Deployment
│   ├── deploy vercel                   Deploy to Vercel
│   ├── deploy cloudflare               Deploy to Cloudflare Workers/Pages
│   ├── deploy railway                  Deploy to Railway
│   ├── deploy fly                      Deploy to Fly.io
│   └── deploy docker                   Deploy via Docker
│
├── Project Utilities
│   ├── init                            Initialize klyron.toml
│   ├── upgrade                         Upgrade Klyron binary
│   ├── doctor                          Full system health check
│   ├── info                            System information
│   ├── version                         Show version
│   ├── telemetry                       Toggle telemetry
│   ├── config                          Edit klyron.toml
│   └── clean                           Clean build artifacts
│
├── AI (Future / Enterprise)
│   ├── ai generate                     AI code generation
│   ├── ai optimize                     AI code optimization
│   ├── ai review                       AI code review
│   ├── ai docs                         AI documentation generation
│   ├── ai test                         AI test generation
│   └── ai migrate                      AI code migration
│
└── help [command]                   Show help for any command
```

### 12.2 Command Categories Summary

| Category | Count | Key Commands |
|----------|-------|-------------|
| Runtime | 4 | run, repl, eval, shell |
| Development | 2 | dev |
| Build | 2 | build |
| Package Manager | 10 | install, add, remove, uninstall, update, upgrade, outdated, audit, doctor, dedupe |
| npm Scripts | 5 | start, test, lint, format, run |
| Workspace | 5 | workspace init/list/add/remove/run |
| Scaffold Frontend | 10 | create react/vue/astro/next/nuxt/sveltekit/solid/qwik/angular/remix |
| Scaffold Backend | 7 | create express/fastify/nest/hono/koa/hapi/adonis |
| Laravel | 8 | create laravel-react/vue/inertia-react/inertia-vue/livewire/next/astro/api |
| Database | 8 | db init/generate/migrate/push/pull/seed/reset/studio |
| ORM Compat | 7 | prisma generate/migrate/studio/db push, drizzle generate/migrate/studio |
| Testing | 7 | test (watch/coverage/ui/e2e/unit/integration) |
| Benchmark | 5 | bench (runtime/http/memory/startup) |
| Linter | 3 | lint |
| Formatter | 3 | format |
| Type Check | 3 | check |
| Plugin | 5 | plugin install/remove/list/update/create |
| Registry | 7 | publish/unpublish/login/logout/whoami/search/info |
| Cache | 3 | cache clean/prune/info |
| Node Compat | 6 | compat check/react/next/astro/nest/prisma |
| NAPI | 3 | napi build/generate/test |
| Docker | 3 | docker init/build/run |
| Deploy | 5 | deploy vercel/cloudflare/railway/fly/docker |
| Project Utils | 8 | init/upgrade/doctor/info/version/telemetry/config/clean |
| AI (Future) | 6 | ai generate/optimize/review/docs/test/migrate |
| **Total** | **~140** | |

### 12.3 Most Frequently Used Commands

Daily driver commands — Klyron feels like Bun + Deno + pnpm + Vite + Prisma CLI + Laravel Installer + Cargo combined:

| Category | Command | Purpose |
|----------|---------|---------|
| Scaffold | `klyron create react` | Create React app |
| Scaffold | `klyron create next` | Create Next.js app |
| Scaffold | `klyron create astro` | Create Astro app |
| Scaffold | `klyron create nest` | Create NestJS app |
| Scaffold | `klyron create laravel-react` | Create Laravel + React app |
| Scaffold | `klyron create laravel-vue` | Create Laravel + Vue app |
| Package | `klyron install` | Install all dependencies |
| Package | `klyron add react` | Add a package |
| Dev | `klyron dev` | Start dev server |
| Build | `klyron build` | Build project |
| Test | `klyron test` | Run tests |
| Lint | `klyron lint` | Lint project |
| Format | `klyron format` | Format project |
| Deploy | `klyron deploy vercel` | Deploy to Vercel |

### 12.4 Global Flags

```
--verbose       Verbose output
--quiet         Suppress non-error output
--no-color      Disable colored output
--json          Output in JSON format
--config <path> Path to klyron.toml
```

---

## 13. API Contracts

### 13.1 Engine Communication

```
▶ Request:
{
  "action": "exec",
  "code": "print('hello')",
  "args": "--flag=value",
  "files": [{ "name": "lib.py", "content": "..." }],
  "filename": "main.py",
  "project": "/path/to/project",
  "stdin": "input data"
}

◀ Response:
{
  "stdout": "hello\n",
  "stderr": "",
  "exit_code": 0,
  "result": "ok",
  "diagnostics": [
    {
      "file": "main.py",
      "line": 5,
      "col": 12,
      "message": "undefined variable 'x'",
      "severity": "warning"
    }
  ]
}
```

### 13.2 Scaffold Generation Response

```
▶ Request:
{
  "action": "scaffold",
  "args": "next",
  "code": "my-app"
}

◀ Response:
{
  "stdout": "",
  "stderr": "",
  "exit_code": 0,
  "result": "{\"type\":\"nextjs\",\"files\":[{\"name\":\"package.json\",\"content\":\"{...}\"},...]}"
}
```

### 13.3 Project Detection

```
Input: /path/to/project
Output:
  "node"     ← has package.json
  "laravel"  ← has composer.json with laravel/framework
  "rust"     ← has Cargo.toml
  "python"   ← has setup.py, pyproject.toml, or requirements.txt
  "ruby"     ← has Gemfile
  "go"       ← has go.mod
  "zig"      ← has build.zig
  "deno"     ← has deno.json or deno.jsonc
  "make"     ← has Makefile
  "unknown"
```

---

## 14. Config Format (klyron.toml)

```toml
[project]
name = "my-app"
version = "0.1.0"
description = "My awesome project"
type = "auto"                        # auto | laravel | node | rust | python | ruby | go | zig

[engine]
js = "v8"                            # v8 | boa | quickjs | jsc
ts = "swc"                           # swc | esbuild | tsc
python = "system"                    # system | venv | conda
php = "system"                       # system | docker | artisan

[engine.transpiler]
target = "esnext"
jsx = "automatic"
decorators = "legacy"                # legacy | 2023-05
sourcemap = true
minify = false

[permissions]
allow_read = ["./src", "./public"]
allow_write = ["./storage", "./dist"]
allow_net = ["localhost:*"]
allow_env = ["APP_ENV", "DB_*"]
allow_run = ["node", "npm", "php", "composer"]
allow_read_all = false
allow_write_all = false
allow_net_all = false
allow_env_all = false
allow_ffi = false
prompt = false                        # Prompt for unlisted permissions
audit = true                          # Log permission usage

[dev]
host = "127.0.0.1"
port = 3000
hmr = true
open = false
https = false

[build]
out_dir = "dist"
minify = true
sourcemap = true
clean = true
target = "esnext"
format = "esm"                       # esm | cjs | iife

[test]
runner = "auto"                      # auto | vitest | jest | phpunit | pest | pytest | rspec
coverage = false
timeout = 30000                      # ms
parallel = true

[lint]
runner = "auto"                      # auto | eslint | biome | pint | ruff | rubocop | clippy
fix = false
fail_on_warning = false

[format]
runner = "auto"                      # auto | prettier | biome | pint | black | gofmt | rustfmt
line_width = 100
indent = "space"                     # space | tab
indent_size = 2

[registry]
default = "npm"
npm = { registry = "https://registry.npmjs.org" }
composer = { registry = "https://repo.packagist.org" }
pip = { registry = "https://pypi.org" }
gem = { registry = "https://rubygems.org" }
cargo = { registry = "https://crates.io" }

[cache]
enabled = true
strategy = "memory"                  # memory | sqlite | redis
ttl = 3600                           # seconds

[database]
default = "sqlite"
connections = [
  { name = "sqlite", driver = "sqlite", filename = "database.sqlite" },
  { name = "mysql", driver = "mysql", host = "localhost", port = 3306, database = "myapp", user = "root" },
  { name = "pgsql", driver = "pgsql", host = "localhost", port = 5432, database = "myapp", user = "postgres" },
]
```

---

## 15. Performance Targets

| Metric | Target | Notes |
|--------|--------|-------|
| Cold start | < 50ms | V8 snapshot, lazy engine spawn |
| Warm start | < 5ms | Engine pooling |
| `console.log("hello")` | < 2ms | |
| `klyron run hello.ts` | < 100ms | Including TS → JS transpile |
| `klyron dev` (React) | < 2s start | Vite HMR |
| `klyron add react` | < 1s | After cache warm |
| `klyron create-next-app` | < 500ms | File generation only |
| Max output per engine | 1 MB | Truncated with notice |
| Engine spawn limit | 100/min | Prevent thrashing |
| HTTP server throughput | 100k req/s | With V8 runtime |
| WASM plugin execution | < 10μs overhead | |
| Binary size | < 50 MB | Stripped, LTO |
| Memory (idle) | < 30 MB | V8 heap initial |
| Memory (hello world) | < 50 MB | |

---

## 16. Roadmap & Timeline

### Ringkasan Visi

Klyron bertujuan menjadi **universal polyglot runtime** alternatif Bun/Deno yang:
- Menjalankan JS/TS native (V8/Boa/QuickJS) dengan performa setara Bun
- Package management multi-registry (npm, PyPI, RubyGems, crates.io, Packagist, Go proxy)
- Scaffold generator untuk 30+ framework dengan struktur default official (`klyron create next` ≡ `npx create-next-app`)
- Laravel first-class citizen: v9/v10/v11/v12/v13, semua stack (Blade, Inertia, Livewire, API)
- Tooling terintegrasi: dev server, test runner, linter, formatter, type checker, benchmark
- Database toolkit: Prisma, Drizzle, TypeORM, MikroORM, Sequelize, Mongoose, Kysely, Knex
- Workspace monorepo management
- Docker & deployment (Vercel, Cloudflare, Railway, Fly)
- Plugin system with WASM hot-reload
- AI-assisted development

---

### Phase 0 — Foundation (Selesai — Q2 2026)

**Status:** ✅ Completed

#### Core Infrastructure
- ✅ CLI dengan 42+ commands (19 scaffold, 10 engines, 5 project tools, 3 db, 3 package, 6 utility)
- ✅ 9 polyglot engines: C, C++, TS, JS, PHP, Python, Ruby, Go, Rust, Zig
- ✅ JS/Deno runtime core + 13 extensions
- ✅ 30+ scaffold generators dengan file lengkap
- ✅ Project tools auto-detect (dev, build, test, lint, format) untuk JS/TS/PHP/Python/Ruby/Go/Rust
- ✅ Database commands (migrate, seed, rollback) untuk 5 project types
- ✅ Package management (add, install, remove) untuk 6 registries
- ✅ Engine bridges + JSON-line protocol
- ✅ Build zero-warning dengan `cargo build`
- ✅ Engine upgrades: C(5 actions), C++(6), TS(9), PHP(7)

#### Existing Scaffold Templates
| Template | Files | Status |
|----------|-------|--------|
| Next.js | 22 | ✅ |
| React + Vite | 18 | ✅ |
| Vue 3 | 18 | ✅ |
| SvelteKit | 18 | ✅ |
| Nuxt 3 | 18 | ✅ |
| Remix | 17 | ✅ |
| Astro | 16 | ✅ |
| SolidJS | 12 | ✅ |
| Preact | 10 | ✅ |
| Express | 8 | ✅ |
| Fastify | 8 | ✅ |
| AdonisJS | 14 | ✅ |
| Laravel | 40 | ✅ |
| Django | 24 | ✅ |
| Rails | 32 | ✅ |
| Go Gin | 4 | ✅ |
| Go Fiber | 3 | ✅ |
| Actix | 3 | ✅ |
| Axum | 3 | ✅ |
| Rocket | 3 | ✅ |
| Leptos | 4 | ✅ |
| Tauri | 5 | ✅ |
| FastAPI | 10 | ✅ |
| Flask | 10 | ✅ |

#### Working CLI Commands
```bash
# Runtime
klyron run app.js/app.ts     klyron eval "code"
klyron repl                  klyron shell

# Polyglot
klyron cc/ts/py/go/rs/zig   klyron cxx/php/rb/js

# Project Tools
klyron dev                   klyron build
klyron test                  klyron lint
klyron format                klyron check

# Scaffold (19 templates)
klyron create react          klyron create next
klyron create vue            klyron create nuxt
klyron create sveltekit      klyron create astro
klyron create remix          klyron create solid
klyron create laravel        klyron create django
klyron create rails          klyron create express
klyron create fastify        klyron create nest
klyron create adonis         klyron create tauri
klyron create axum           klyron create leptos
klyron create fastapi

# Package
klyron install               klyron add react
klyron remove react          klyron outdated
klyron update

# Database
klyron db migrate            klyron db seed
klyron db rollback

# Laravel
klyron artisan serve         klyron composer require
klyron blade render          klyron tinker
```

---

### Phase 1 — Modularisasi & Core Crate Restructure (Q3 2026)

**Goal:** Split monolithic `src/` into proper crate-based architecture. Every crate independently testable.

#### Crate Implementation

| Crate | Action | Source | Verification |
|-------|--------|--------|-------------|
| `klyron_cli` | Split from `src/cli/src/main.rs` | All CLI command routing + arg parsing | `cargo test -p klyron_cli` |
| `klyron_runtime` | Split from `src/core/` | Event loop, V8 isolate, module system | Run `klyron run hello.ts` |
| `klyron_engine` | Extract from `src/cli/src/engines/` | Engine trait, bridges, JSON protocol | All 10 engines respond to ping |
| `klyron_loader` | New crate | Module resolution, import maps, node_modules | `klyron run` resolves imports correctly |
| `klyron_pm` | New crate | PackageManager trait, registry backends | `klyron add react` installs from npm |
| `klyron_bundler` | New crate | Bundler trait (esbuild, vite, rollup) | `klyron build` outputs dist/ |
| `klyron_transpiler` | New crate | Transpiler trait (swc, esbuild, tsc) | TS → JS transpile test |
| `klyron_watcher` | New crate | File watcher (notify/chokidar) | Watch mode detects file changes |
| `klyron_config` | New crate | klyron.toml + CLI flags merge | `klyron config` reads/writes config |
| `klyron_utils` | New crate | Shared: path, semver, hash, shell, json | Utility function tests |

#### CLI Commands Working After Phase 1
```
Semua command Phase 0 + workspace structure test
```

#### Testable Deliverables
- `cargo build` compiles with zero warnings across all crates
- Each crate has `#[cfg(test)]` module with unit tests
- `cargo test -p klyron_cli` passes all CLI routing tests
- `cargo test -p klyron_engine` passes all engine bridge tests
- All Phase 0 commands still work after refactor

---

### Phase 2 — Web API, HTTP & Extensions (Q4 2026)

**Goal:** Implement Web standard APIs + database bindings. Klyron can run real web apps.

#### Crate Implementation

| Crate | Struct | APIs | Verification |
|-------|--------|------|-------------|
| `klyron_web` | `WebApi` | `fetch()`, `URL`, `URLSearchParams`, `Headers`, `Request`, `Response` | `klyron eval "fetch('...')"` |
| `klyron_http` | `HttpServer` | `serve()`, HTTP/1.1, WebSocket, TLS | `klyron run server.ts` listens on port |
| `klyron_fs` | `FileSystem` | read, write, copy, move, remove, watch, permissions | File operations in eval |
| `klyron_crypto` | `CryptoProvider` | hash (SHA-256/512), encrypt (AES), randomBytes, UUID | `crypto.randomUUID()` |
| `klyron_dns` | `DnsResolver` | resolve A/AAAA/CNAME/MX/TXT, reverse lookup | DNS resolution in runtime |
| `klyron_cache` | `CacheManager` | Memory, SQLite, Redis backend; TTL, tags, patterns | Set/get/delete operations |
| `klyron_sqlite` | `SqliteDb` | open, query, exec, transaction, WAL mode | SQLite CRUD in eval |
| `klyron_postgres` | `PostgresDb` | connect, pool, query, exec, transaction | PostgreSQL CRUD |
| `klyron_mysql` | `MySqlDb` | connect, pool, query, exec, transaction | MySQL CRUD |
| `klyron_process` | `ProcessManager` | spawn, exec, kill, pipe stdin/stdout/stderr | `klyron exec "ls -la"` |
| `klyron_logger` | `Logger` | info/warn/error/debug, structured JSON, file output | Logger output verification |

#### Runtime JS Bootstrap Files

| File | Implements | Compat Target |
|------|-----------|---------------|
| `runtime/js/globals.js` | globalThis, setTimeout, setInterval, queueMicrotask | Web + Node |
| `runtime/js/console.js` | console.log/warn/error/info/table/time/timeEnd | Web + Node |
| `runtime/js/timers.js` | setTimeout, setInterval, setImmediate, clear* | Node |
| `runtime/js/fetch.js` | fetch, Request, Response, Headers | Web |
| `runtime/js/buffer.js` | Buffer, Blob, File | Node |
| `runtime/js/streams.js` | ReadableStream, WritableStream, TransformStream | Web |
| `runtime/js/crypto.js` | crypto.subtle, randomUUID, getRandomValues | Web |
| `runtime/js/encoding.js` | TextEncoder, TextDecoder | Web |
| `runtime/js/url.js` | URL, URLSearchParams | Web |
| `runtime/js/events.js` | EventTarget, EventEmitter, CustomEvent | Web + Node |
| `runtime/js/abort.js` | AbortController, AbortSignal | Web |
| `runtime/js/klyron.js` | `KlyronRuntime` SDK APIs | Klyron-specific |

#### CLI Commands Working After Phase 2
```bash
klyron run server.ts          # Runs HTTP server
klyron eval "fetch('url')"    # Web fetch works
klyron exec "ls -la"          # Process spawn
klyron db init                # SQLite/Postgres/MySQL init
```

#### Testable Deliverables
- `klyron eval "fetch('https://httpbin.org/get').then(r => r.json())"` returns JSON
- `klyron run examples/02-http-server/server.ts` starts server, curl returns 200
- `klyron db init && klyron db migrate` creates SQLite database
- All runtime JS files loaded and functional in V8 isolate
- `cargo test -p klyron_http` passes HTTP server tests

---

### Phase 3 — Node.js Compat & Module System (Q1 2027)

**Goal:** Full Node.js compatibility — run existing Node.js/Bun projects without changes.

#### Crate Implementation

| Crate | Struct | Features | Verification |
|-------|--------|----------|-------------|
| `klyron_node` | `NodeCompat` | require(), module, exports, \_\_dirname, \_\_filename, process, Buffer, global | `klyron run node-app.js` works |
| `klyron_loader` | (extend) | CJS/ESM interop, package.json exports, imports, main | Module resolution tests |
| `klyron_napi` | `NapiLoader` | napi.h binding, Node-API functions | `klyron napi test` passes |

#### Node.js Polyfills (in `compatibility/node/`)

| Module | APIs | Only |
|--------|------|------|
| `fs` | readFile, writeFile, readdir, mkdir, stat, watchFile, exists, promises, createReadStream, createWriteStream | ✅ Core |
| `path` | join, resolve, dirname, basename, extname, relative, normalize, parse | ✅ Core |
| `http` | createServer, request, get, Agent, Server, IncomingMessage, ServerResponse | ✅ Core |
| `https` | createServer, request, get, Agent | ✅ Core |
| `os` | platform, arch, cpus, totalmem, freemem, homedir, hostname, networkInterfaces, tmpdir, EOL | ✅ Core |
| `crypto` | createHash, randomBytes, createCipheriv, createDecipheriv, sign, verify, generateKeyPair | ✅ Core |
| `stream` | Readable, Writable, Transform, Duplex, pipeline, finished, PassThrough | ✅ Core |
| `events` | EventEmitter, once, on, listenerCount, getMaxListeners | ✅ Core |
| `util` | promisify, callbackify, inherits, inspect, types, deprecate | ✅ Core |
| `child_process` | spawn, exec, execFile, fork, ChildProcess | ✅ Core |
| `assert` | strict, deepEqual, doesNotThrow, rejects, throws | ✅ Core |
| `buffer` | Buffer, Blob, SlowBuffer, constants | ✅ Core |
| `url` | URL, URLSearchParams, fileURLToPath, pathToFileURL, format, parse | ✅ Core |
| `timers` | setImmediate, clearImmediate | ✅ Core |
| `process` | env, argv, cwd, chdir, exit, nextTick, stdout, stderr, stdin, on/emit | ✅ Global |
| `console` | Extended: Console class, time/timeLog/timeEnd, count, group, profile | ✅ Global |
| `module` | require.resolve, Module._resolveFilename, Module._extensions | ✅ Core |

#### CJS / ESM Interop
- Auto-detect ESM vs CJS from `package.json` type field
- Auto-detect from file extension (.mjs → ESM, .cjs → CJS, .js → check package.json)
- `require()` ESM modules via dynamic import()
- `import` CJS modules via default interop
- `import()` support for dynamic imports
- Package entry resolution via `exports` map (package.json)
- Conditional exports: `import`, `require`, `node`, `browser`, `default`

#### CLI Commands Working After Phase 3
```bash
klyron run express-app.js     # Runs existing Express.js app
klyron run next-app           # Runs Next.js dev (limited)
klyron run nuxt-app           # Runs Nuxt dev (limited)
klyron eval "require('fs').readFileSync('file.txt')"  # CJS require works
klyron eval "import('lodash').then(_ => _.chunk([1,2,3], 2))"  # Dynamic import works
```

#### Testable Deliverables
- Run existing Node.js Express app: `klyron run node_modules/.bin/express myapp`
- `require('fs')` returns fs module with all core APIs
- `process.env.NODE_ENV` returns value
- `Buffer.from('hello').toString()` returns 'hello'
- CJS/ESM interop: `require('./esm-module.mjs')` works
- `cargo test -p klyron_node` passes all Node compat tests

---

### Phase 4 — Framework Adapters & Scaffold v2 (Q2 2027)

**Goal:** Every `klyron create <framework>` generates the **exact same output** as the official CLI (e.g., `klyron create next` ≡ `npx create-next-app`). Framework version detection + adapter system.

#### Adapter Trait Implementation

```rust
#[async_trait]
pub trait FrameworkAdapter: Send + Sync {
    fn name(&self) -> &'static str;
    fn detect(&self, dir: &Path) -> bool;
    fn supported_versions(&self) -> Vec<&'static str>;
    async fn dev(&self, dir: &Path, port: Option<u16>) -> Result<()>;
    async fn build(&self, dir: &Path, opts: BuildOptions) -> Result<()>;
    async fn test(&self, dir: &Path, filter: Option<&str>) -> Result<()>;
    async fn lint(&self, dir: &Path) -> Result<()>;
    async fn format(&self, dir: &Path) -> Result<()>;
    async fn scaffold(&self, name: &str, dir: &Path, version: &str, options: ScaffoldOptions) -> Result<()>;
}
```

#### Framework Version Matrix

| Framework | Versions | Default | Detection File |
|-----------|----------|---------|---------------|
| React | 18.x, 19.x | 19.x | `vite.config.*` + `package.json` react version |
| Next.js | 13.x, 14.x, 15.x | 15.x | `next.config.*` + package.json next version |
| Vue | 2.x, 3.x | 3.x | `vite.config.*` + vue version in package.json |
| Nuxt | 2.x, 3.x | 3.x | `nuxt.config.*` + nuxt version |
| Astro | 3.x, 4.x, 5.x | 5.x | `astro.config.*` |
| Svelte | 4.x, 5.x | 5.x | `svelte.config.*` or vite with svelte plugin |
| SvelteKit | 1.x, 2.x | 2.x | `svelte.config.js` with kit |
| Solid | 1.x | 1.x | `vite.config.*` with solid plugin |
| Qwik | 1.x | 1.x | `qwik.config.*` or `qwik-city` in deps |
| Angular | 17.x, 18.x, 19.x | 19.x | `angular.json` |
| Remix | 1.x, 2.x | 2.x | `remix.config.*` or `@remix-run` in deps |
| Preact | 10.x | 10.x | `vite.config.*` with preact plugin |
| Lit | 2.x, 3.x | 3.x | `package.json` lit version |
| Express | 4.x | 4.x | `package.json` express version |
| Fastify | 4.x, 5.x | 5.x | `package.json` fastify version |
| NestJS | 9.x, 10.x, 11.x | 11.x | `nest-cli.json` |
| Hono | 3.x, 4.x | 4.x | `package.json` hono version |
| Koa | 2.x | 2.x | `package.json` koa version |
| Hapi | 20.x, 21.x | 21.x | `package.json` @hapi/hapi version |
| AdonisJS | 5.x, 6.x | 6.x | `.adonisrc.*` or `adonisrc.*` |
| Laravel | 9.x, 10.x, 11.x, 12.x, 13.x | 11.x | `artisan` + `composer.json` laravel/framework |
| Django | 4.x, 5.x | 5.x | `manage.py` + `pyproject.toml` |
| Rails | 7.x, 8.x | 8.x | `bin/rails` + `Gemfile` |

#### Scaffold Structure Parity Checklist

For each framework, the generated output must match the official CLI:

| Framework | Official CLI | File Count | Key Files Parity |
|-----------|-------------|------------|-----------------|
| Next.js | `npx create-next-app` | 22 files | app/layout.tsx, page.tsx, globals.css, next.config, tsconfig, package.json |
| React | `npm create vite@latest -- --template react-ts` | 18 files | vite.config, main.tsx, App.tsx, index.html, tsconfig |
| Vue | `npm create vue@latest` | 18 files | vite.config, main.ts, App.vue, router, pinia store |
| Nuxt | `npx nuxi init` | 18 files | nuxt.config, app.vue, pages/, server/ |
| Astro | `npm create astro@latest` | 16 files | astro.config, src/pages/, src/components/ |
| SvelteKit | `npm create svelte@latest` | 18 files | svelte.config, src/routes/, src/lib/ |
| Angular | `ng new` | 20 files | angular.json, main.ts, app.component, standalone |
| Remix | `npx create-remix@latest` | 17 files | remix.config, app/routes/, app/root.tsx |
| Qwik | `npm create qwik@latest` | 14 files | qwik.config, src/routes/, src/components/ |
| Solid | `npm create solid@latest` | 14 files | vite.config, src/App.tsx, src/routes/ |
| Preact | `npm create vite@latest -- --template preact-ts` | 10 files | vite.config, src/main.tsx, src/app.tsx |
| Lit | `npm create vite@latest -- --template lit-ts` | 10 files | vite.config, src/my-element.ts |
| Express | Manual | 8 files | app.js, routes/, middleware/, package.json |
| Fastify | `npm create fastify` | 8 files | app.js, plugins/, routes/, package.json |
| NestJS | `nest new` | 15 files | nest-cli.json, src/main.ts, app.module.ts |
| Hono | `npm create hono@latest` | 8 files | hono.config, src/index.ts, package.json |
| Koa | Manual | 8 files | app.js, routes/, middleware/ |
| Hapi | Manual | 8 files | server.js, plugins/, routes/ |
| AdonisJS | `npm create adonis-js-app` | 14 files | adonisrc, ace, start/routes.ts, app/ |
| Laravel | `composer create-project laravel/laravel` | 40 files | artisan, composer.json, .env, config/, routes/ |
| Django | `django-admin startproject` | 24 files | manage.py, settings.py, urls.py, wsgi.py |
| Rails | `rails new` | 32 files | Gemfile, config/routes.rb, app/controllers/ |

#### Adapter Implementation Order

**Batch 1 (Production-ready — Q2 2027 Week 1-4):**
```
React, Vue, Next.js, Astro → detect(), dev(), build(), scaffold() ✅
Express, Fastify, Hono     → detect(), dev(), build(), scaffold() ✅
Laravel scaffold           → detect(), dev(), build(), test(), scaffold() ✅
```

**Batch 2 (Standard — Q2 2027 Week 5-8):**
```
Nuxt, SvelteKit, Remix, Angular → detect(), dev(), build(), scaffold() ✅
NestJS, AdonisJS, Koa, Hapi     → detect(), dev(), build(), scaffold() ✅
Django, Rails scaffold          → detect(), dev(), build(), scaffold() ✅
```

**Batch 3 (Extended — Q2 2027 Week 9-12):**
```
SolidStart, Qwik, Preact, Lit   → detect(), dev(), build(), scaffold() ✅
Solid, Svelte, tRPC, GraphQL    → detect(), dev(), build(), scaffold() ✅
Go/RS/Polyglot scaffold         → detect(), dev(), build(), scaffold() ✅
```

#### CLI Commands Working After Phase 4
```bash
# Scaffold — output matches official CLI exactly
klyron create next my-app           # = npx create-next-app@latest my-app
klyron create react my-app          # = npm create vite@latest my-app -- --template react-ts
klyron create vue my-app            # = npm create vue@latest my-app
klyron create astro my-app          # = npm create astro@latest my-app
klyron create nuxt my-app           # = npx nuxi init my-app
klyron create sveltekit my-app      # = npm create svelte@latest my-app
klyron create angular my-app        # = ng new my-app
klyron create remix my-app          # = npx create-remix@latest my-app
klyron create qwik my-app           # = npm create qwik@latest my-app
klyron create solid my-app          # = npm create solid@latest my-app

# Backend
klyron create express my-app
klyron create fastify my-app
klyron create nest my-app
klyron create hono my-app
klyron create koa my-app
klyron create hapi my-app
klyron create adonis my-app

# Framework version selection
klyron create next my-app --version 14   # Scaffold Next.js 14
klyron create next my-app --version 15   # Scaffold Next.js 15
```

#### Testable Deliverables
- `klyron create next my-app && ls my-app/` matches `npx create-next-app my-app && ls my-app/`
- `klyron create react my-app && ls my-app/` matches `npm create vite@latest my-app`
- Framework version detection: `klyron info` shows correct framework version
- `klyron dev` inside scaffolded project starts dev server
- `klyron build` inside scaffolded project produces dist/

---

### Phase 5 — Testing, Linting, Formatting, Benchmark, Type Checking (Q3 2027)

**Goal:** Complete tooling suite. Every `klyron test/lint/format/check/bench` works across all languages.

#### Crate Implementation

| Crate | Struct | Features | Backends |
|-------|--------|----------|----------|
| `klyron_test` | `TestRunner` | run, watch, coverage, ui, e2e, unit, integration | vitest, jest, phpunit, pest, pytest, rspec, cargo-test, go-test |
| `klyron_linter` | `Linter` | lint, fix, report, ignore | eslint, biome, pint, ruff, rubocop, clippy, golint |
| `klyron_formatter` | `Formatter` | format, check, write, diff | prettier, biome, pint, black, gofmt, rustfmt, rubocop -A |
| `klyron_bench` | `BenchmarkRunner` | runtime, http, memory, startup | Custom Rust harness + hyperfine |
| `klyron_compat` | `CompatChecker` | check framework compat, API coverage | Node.js compat matrix |

#### Test Runner Commands

| Command | Action | Backend Auto-Detect |
|---------|--------|-------------------|
| `klyron test` | Run all tests | Detects vitest → jest → phpunit → pytest → rspec |
| `klyron test watch` | Watch mode | `vitest --watch`, `jest --watch` |
| `klyron test coverage` | With coverage | `vitest --coverage`, `jest --coverage`, `phpunit --coverage` |
| `klyron test ui` | Vitest UI | `vitest --ui` |
| `klyron test e2e` | E2E tests | Detects playwright → cypress → puppeteer |
| `klyron test unit` | Unit only | Passes `--testPathPattern=unit` or equiv |
| `klyron test integration` | Integration only | Passes `--testPathPattern=integration` or equiv |

#### Linter Commands

| Command | Action | Backend Auto-Detect |
|---------|--------|-------------------|
| `klyron lint` | Lint project | eslint → biome → clippy → ruff → rubocop |
| `klyron lint src/` | Lint specific dir | Passes directory to linter |
| `klyron lint fix` | Auto-fix | `--fix` flag for eslint, `cargo clippy --fix` |

#### Formatter Commands

| Command | Action | Backend Auto-Detect |
|---------|--------|-------------------|
| `klyron format` | Check formatting | prettier → rustfmt → gofmt → black |
| `klyron format src/` | Check specific dir | |
| `klyron format --write` | Format and write | `--write` for prettier, `--fix` for biome |

#### Type Checker Commands

| Command | Action | Backend |
|---------|--------|---------|
| `klyron check` | Run all checks | TypeScript + project validation |
| `klyron check types` | TypeScript only | `tsc --noEmit` |
| `klyron check project` | Full validation | types + lint + format check |

#### Benchmark Commands

| Command | Action | Metrics |
|---------|--------|---------|
| `klyron bench` | All benchmarks | Combined report |
| `klyron bench runtime` | Runtime perf | Execution time, throughput |
| `klyron bench http` | HTTP server | Req/s, latency p50/p95/p99 |
| `klyron bench memory` | Memory usage | RSS, heap used, heap total |
| `klyron bench startup` | Cold/warm start | Time to first response |

#### CLI Commands Working After Phase 5
```bash
klyron test                    # Auto-detect + run
klyron test watch              # Watch mode
klyron test coverage           # Coverage report
klyron test e2e                # E2E tests
klyron lint                    # Auto-detect linter
klyron lint fix                # Auto-fix
klyron format --write          # Format + write
klyron check types             # tsc --noEmit
klyron bench                   # All benchmarks
klyron bench http              # HTTP benchmark
```

#### Testable Deliverables
- `klyron test` in React project runs vitest, shows pass/fail
- `klyron lint` detects ESLint errors in JS project
- `klyron format --write` formats all files in project
- `klyron check types` reports TypeScript errors
- `klyron bench runtime` outputs performance metrics
- Auto-detection works: React → vitest, NestJS → jest, Laravel → phpunit

---

### Phase 6 — Laravel Ecosystem Complete (Q4 2027)

**Goal:** Klyron is the best Laravel development tool outside of `artisan`. Full Laravel v9/v10/v11/v12/v13 support.

#### Laravel Version Support

| Version | PHP Min | Release | EOL | Features |
|---------|---------|---------|-----|----------|
| 9.x | 8.0 - 8.2 | Feb 2022 | Aug 2024 | LTS, Native type declarations, Process layer |
| 10.x | 8.1 - 8.3 | Feb 2023 | Aug 2025 | LTS, No more Http Kernel, Simplified bootstrap |
| 11.x | 8.2 - 8.4 | Mar 2024 | Aug 2026 | SQLite by default, Health routing, Graceful encryption |
| 12.x | 8.3 - 8.4 | Q1 2025 | TBD | Reverb, new Artisan commands, Laravel Pennant GA |
| 13.x | 8.4+ | Q1 2026 | TBD | TBD (latest features) |

#### Laravel Scaffold Stacks

| Stack | Frontend | Inertia | SSR | API | Files |
|-------|----------|---------|-----|-----|-------|
| `blade` | Blade + Tailwind + Alpine.js | ❌ | ❌ | ❌ | 35 |
| `inertia-react` | React + Tailwind + shadcn/ui | ✅ | ✅ | REST | 42 |
| `inertia-vue` | Vue 3 + Tailwind + PrimeVue | ✅ | ✅ | REST | 42 |
| `livewire` | Livewire 3 + Volt + Alpine.js | ❌ | ❌ | ❌ | 38 |
| `react` | React SPA (separate from Laravel) | ❌ | ❌ | Sanctum | 45 |
| `vue` | Vue 3 SPA (separate from Laravel) | ❌ | ❌ | Sanctum | 45 |
| `next` | Next.js BFF + Laravel API backend | ❌ | ❌ | Sanctum + JWT | 50 |
| `astro` | Astro + Laravel API backend | ❌ | ❌ | Sanctum | 48 |
| `api` | No frontend (API-only) | ❌ | ❌ | Sanctum + Swagger | 30 |

#### Artisan Command Wrapper — Complete Set

```bash
# Make Commands (50+)
klyron artisan make:controller PostController --api
klyron artisan make:model Post -m -f -s -c
klyron artisan make:migration create_posts_table
klyron artisan make:seeder PostSeeder
klyron artisan make:factory PostFactory
klyron artisan make:resource PostResource
klyron artisan make:request StorePostRequest
klyron artisan make:mail WelcomeMail
klyron artisan make:notification PostCreated
klyron artisan make:listener SendWelcomeEmail
klyron artisan make:event PostCreated
klyron artisan make:job ProcessPost --sync
klyron artisan make:command ImportPosts
klyron artisan make:rule ValidSlug
klyron artisan make:cast UserIdCast
klyron artisan make:channel PostChannel
klyron artisan make:observer PostObserver
klyron artisan make:policy PostPolicy
klyron artisan make:provider PostServiceProvider
klyron artisan make:action CreatePost        # Laravel 11+
klyron artisan make:enum PostStatus          # Laravel 11+
klyron artisan make:class PostFormatter       # Laravel 11+

# Database
klyron artisan migrate:fresh --seed
klyron artisan migrate:rollback --step=3
klyron artisan db:seed --class=DatabaseSeeder
klyron artisan db:wipe --drop-types

# Cache
klyron artisan cache:clear
klyron artisan config:clear
klyron artisan route:clear
klyron artisan view:clear
klyron artisan optimize:clear
klyron artisan optimize

# Queue (Horizon)
klyron artisan queue:work --queue=high,default
klyron artisan queue:listen
klyron artisan queue:restart
klyron artisan queue:failed
klyron artisan queue:retry all

# Horizon
klyron artisan horizon
klyron artisan horizon:install
klyron artisan horizon:pause
klyron artisan horizon:continue
klyron artisan horizon:terminate
klyron artisan horizon:status
klyron artisan horizon:snapshot

# Telescope
klyron artisan telescope:install
klyron artisan telescope:prune --hours=48
klyron artisan telescope:clear

# Sail
klyron sail up -d
klyron sail down
klyron sail artisan migrate
klyron sail npm run dev

# Schedule
klyron artisan schedule:run
klyron artisan schedule:list
klyron artisan schedule:work

# Route
klyron artisan route:list
klyron artisan route:cache
klyron artisan route:clear

# Storage
klyron artisan storage:link
klyron artisan storage:unlink

# Vendor
klyron artisan vendor:publish --tag=config
```

#### PHP Ecosystem Scaffolds

| PHP Project | Files | Key Features |
|-------------|-------|-------------|
| Laravel (all stacks) | 30-50 | Blade/Inertia/Livewire/API, Sanctum, Breeze, Jetstream |
| Symfony | 28 | Symfony 6/7 skeleton, Maker bundle, Doctrine, Twig |
| CodeIgniter 4 | 18 | CI4 starter, Sparks package manager |
| WordPress | 12 | Plugin scaffold, block theme, wp-env |

#### CLI Commands Working After Phase 6
```bash
# Laravel create — 9 stacks
klyron create laravel-react my-app              # Laravel + React SPA
klyron create laravel-vue my-app                # Laravel + Vue SPA
klyron create laravel-inertia-react my-app       # Inertia + React
klyron create laravel-inertia-vue my-app         # Inertia + Vue
klyron create laravel-livewire my-app           # Livewire + Volt
klyron create laravel-next my-app               # Next.js BFF + Laravel API
klyron create laravel-astro my-app              # Astro + Laravel API
klyron create laravel-api my-app                # API-only + Sanctum
klyron create laravel-blade my-app              # Blade + Tailwind

# With version
klyron create laravel my-app --laravel-version=10
klyron create laravel my-app --laravel-version=11
klyron create laravel my-app --laravel-version=12

# PHP ecosystem
klyron create symfony my-app
klyron create codeigniter my-app
klyron create wordpress my-app

# Artisan full wrapper
klyron artisan make:model Post -a
klyron artisan migrate:fresh --seed
klyron artisan queue:work
klyron artisan horizon

# Sail
klyron sail up -d
klyron sail artisan migrate
```

#### Testable Deliverables
- `klyron create laravel-inertia-react my-app` generates 42 files matching official Laravel + Inertia structure
- `klyron artisan make:model Post -a` creates Post model + migration + factory + seeder + controller
- `klyron artisan migrate:fresh --seed` runs migrations and seeds
- `klyron sail up -d` starts Docker containers
- Laravel v10/v11/v12 detection: scaffolded app has correct version in composer.json
- `klyron artisan route:list` shows all registered routes

---

### Phase 7 — Database, ORM & Prisma/Drizzle Compat (Q1 2028)

**Goal:** `klyron db` commands work with all major ORMs. `klyron prisma` and `klyron drizzle` are drop-in replacements.

#### ORM Support Matrix

| ORM | Language | DB Support | klyron db commands | Files |
|-----|----------|-----------|-------------------|-------|
| Prisma | TS/JS | postgresql, mysql, sqlite, sqlserver, mongodb, cockroachdb | init, generate, migrate, push, pull, seed, studio, reset | `schema.prisma` |
| Drizzle | TS/JS | postgresql, mysql, sqlite, turso, neon, planetscale | init, generate, migrate, push, pull, studio | `drizzle.config.ts` |
| TypeORM | TS/JS | postgresql, mysql, sqlite, sqlserver, mongodb, sql.js | init, migrate, generate, sync | Entity files |
| MikroORM | TS/JS | postgresql, mysql, sqlite, mongodb | init, migrate, generate, seed | `micro-orm.config.ts` |
| Sequelize | JS | postgresql, mysql, sqlite, sqlserver, mariadb | init, migrate, seed | `config/config.json` |
| Mongoose | JS | mongodb | init, generate | `mongoose` schema files |
| Kysely | TS/JS | postgresql, mysql, sqlite | init, generate | TypeScript types |
| Knex | JS | postgresql, mysql, sqlite, sqlserver | init, migrate, seed | `knexfile.ts` |

#### Database Commands per ORM

| klyron command | Prisma | Drizzle | TypeORM | MikroORM | Sequelize | Knex |
|----------------|--------|---------|---------|----------|-----------|------|
| `db init` | `prisma init` | `drizzle-kit init` | `typeorm init` | `mikro-orm init` | `sequelize init` | `knex init` |
| `db generate` | `prisma generate` | `drizzle-kit generate` | `typeorm migration:generate` | `mikro-orm migration:create` | - | `knex migrate:make` |
| `db migrate` | `prisma migrate dev` | `drizzle-kit migrate` | `typeorm migration:run` | `mikro-orm migration:up` | `sequelize db:migrate` | `knex migrate:latest` |
| `db push` | `prisma db push` | `drizzle-kit push` | `typeorm schema:sync` | `mikro-orm schema:update` | `sequelize db:sync` | - |
| `db pull` | `prisma db pull` | `drizzle-kit pull` | `typeorm model:generate` | `mikro-orm schema:dump` | - | - |
| `db seed` | `prisma db seed` | manual | `typeorm migration:run` seed | `mikro-orm seeder:run` | `sequelize db:seed:all` | `knex seed:run` |
| `db reset` | `prisma migrate reset` | manual | `typeorm schema:drop` + migrate | `mikro-orm schema:drop` + migrate | `sequelize db:migrate:undo:all` | `knex migrate:rollback --all` |
| `db studio` | `prisma studio` | `drizzle-kit studio` | - | - | - | - |

#### CLI Commands Working After Phase 7
```bash
# Prisma
klyron prisma generate              # Generate Prisma client
klyron prisma migrate dev           # Dev migration
klyron prisma db push               # Push schema
klyron prisma studio                # Open Prisma Studio

# Drizzle
klyron drizzle generate             # Generate Drizzle schema
klyron drizzle migrate              # Run migrations
klyron drizzle studio               # Open Drizzle Studio

# Generic DB (auto-detect ORM)
klyron db init                      # Init detected ORM
klyron db generate                  # Generate schema
klyron db migrate                   # Run migrations
klyron db push                      # Push schema
klyron db pull                      # Pull schema
klyron db seed                      # Seed data
klyron db reset                     # Full reset
klyron db studio                    # Open studio

# With specific ORM
klyron db --orm=prisma migrate
klyron db --orm=drizzle generate
klyron db --orm=typeorm migration:run
```

#### Testable Deliverables
- `klyron db init` in empty project creates `schema.prisma` (Prisma) or `drizzle.config.ts` (Drizzle)
- `klyron prisma generate` generates Prisma client
- `klyron drizzle migrate` creates SQLite/mysql/postgres tables
- Auto-detection: project with `schema.prisma` → uses Prisma commands
- `klyron db studio` opens Prisma Studio or Drizzle Studio in browser

---

### Phase 8 — Registry, SDK & Package Management Complete (Q2 2028)

**Goal:** Full multi-registry package management + SDKs. Klyron replaces npm/pip/gem/cargo/packagist CLI.

#### Crate Implementation

| Crate | Struct | Features |
|-------|--------|----------|
| `klyron_registry` | `RegistryClient` | search, info, download, publish, unpublish, login, logout, whoami |
| SDK JS | `@klyron/sdk` | npm package: KlyronRuntime API bindings for browser/Node |
| SDK TS | `@klyron/types` | npm package: TypeScript type definitions |
| SDK Rust | `klyron-sdk` | crates.io: Rust SDK for Klyron runtime |
| SDK C++ | `klyron/sdk` | vcpkg: C++ header-only SDK |
| SDK PHP | `klyron/sdk` | Packagist: PHP SDK for Klyron runtime |

#### Registry Backend Details

| Registry | Protocol | Auth | Rate Limit | Cache Strategy |
|----------|----------|------|------------|---------------|
| npm | HTTPS + JSON | Bearer token | 100/min | SQLite metadata cache |
| PyPI | HTTPS + JSON | Token | 60/min | SQLite metadata cache |
| RubyGems | HTTPS + JSON | API key | 60/min | SQLite metadata cache |
| crates.io | HTTPS + JSON + Git index | Token | 60/min | Git sparse index |
| Packagist | HTTPS + JSON | Token | 60/min | SQLite metadata cache |
| Go Proxy | HTTPS | None | 100/min | SQLite metadata cache |

#### Package Management Commands

| Command | Description | Implementation |
|---------|-------------|---------------|
| `klyron install` | Install all deps | Auto-detect package files, install all |
| `klyron add react` | Add single package | Detect registry from package name |
| `klyron add @types/react --dev` | Dev dependency | `--dev` flag |
| `klyron remove react` | Remove package | Remove from all package files |
| `klyron uninstall react` | Alias for remove | Same as remove |
| `klyron update` | Update all deps | Respect semver ranges |
| `klyron outdated` | List outdated | Check latest versions |
| `klyron audit` | Security audit | npm audit, cargo-audit, pip-audit |
| `klyron doctor` | Package health | Check integrity, missing deps |
| `klyron dedupe` | Deduplicate | npm dedupe equivalent |
| `klyron cache clean` | Clear all caches | Package metadata + tarball cache |
| `klyron cache prune` | Remove expired | TTL-based pruning |
| `klyron cache info` | Cache statistics | Size, entry count, hit ratio |

#### Registry Commands

| Command | Description |
|---------|-------------|
| `klyron login npm` | Login to npm registry |
| `klyron login pypi` | Login to PyPI |
| `klyron login` | Login to default registry |
| `klyron logout` | Logout from all registries |
| `klyron whoami` | Show current user |
| `klyron search react` | Search packages across registries |
| `klyron info react` | Show package details |
| `klyron info react --version 18.3.1` | Specific version info |
| `klyron publish` | Publish current package |
| `klyron unpublish @my/pkg` | Unpublish package |

#### CLI Commands Working After Phase 8
```bash
# Package management
klyron install                    # Install all deps
klyron add react                  # npm
klyron add django                 # pip
klyron add rails                  # gem
klyron add serde                  # cargo
klyron add laravel/framework      # composer
klyron add github.com/gin-gonic/gin  # go
klyron update                     # Update all
klyron outdated                   # Check outdated
klyron audit                      # Security audit
klyron dedupe                     # Deduplicate

# Registry
klyron login npm
klyron whoami
klyron search react
klyron info react
klyron publish

# Cache
klyron cache clean
klyron cache info

# SDK usage
# import { KlyronRuntime } from '@klyron/sdk';
# const klyron = new KlyronRuntime();
# await klyron.fs.read('/path');
```

#### Testable Deliverables
- `klyron add react` installs react from npm, creates/updates package.json
- `klyron add django` installs django from PyPI
- `klyron audit` reports vulnerabilities
- `klyron search react` returns search results from npm
- `klyron whoami` shows logged-in user
- SDK packages publishable to their respective registries

---

### Phase 9 — Plugin, Workspace, Docker, Deploy & Node Compat (Q3 2028)

**Goal:** Extensibility + deployment. Klyron manages monorepos, deploys to cloud, runs in Docker.

#### Plugin System (`crates/klyron_plugin`)

| Feature | Implementation |
|---------|---------------|
| Plugin format | WASM modules + JSON manifest |
| Plugin manifest | `klyron-plugin.toml` — name, version, hooks, permissions |
| Hook system | pre/post for every CLI command |
| Lifecycle | install, enable, disable, remove, update |
| Hot-reload | Watch plugin dir, reload on change |
| Sandbox | WASM sandbox with capability-based permissions |
| Registry | Plugin registry at `registry.klyron.dev/plugins` |

#### Workspace / Monorepo (`crates/klyron_workspace`)

| Command | Description |
|---------|-------------|
| `klyron workspace init` | Create workspace root with `klyron.toml` |
| `klyron workspace list` | List all workspace members |
| `klyron workspace add frontend` | Add member from template |
| `klyron workspace remove frontend` | Remove member |
| `klyron workspace run build` | Run build script across all members |
| `klyron workspace run test` | Run test across all members |

Workspace features:
- Shared dependency hoisting (like pnpm workspaces)
- Filtered command execution: `klyron workspace run build --filter=@myapp/frontend`
- Dependency graph: `klyron workspace graph` shows member dependency tree
- Version management: `klyron workspace version patch` bumps all members

#### Docker Integration (`crates/klyron_docker`)

| Command | Description | Generated Files |
|---------|-------------|-----------------|
| `klyron docker init` | Generate Dockerfile + docker-compose.yml + .dockerignore | Dockerfile, docker-compose.yml, .dockerignore |
| `klyron docker build` | Build Docker image | Tags with project name + version |
| `klyron docker run` | Run container | Maps ports from klyron.toml |

Dockerfile templates per framework:
- Node/JS: multi-stage (node:22-alpine → distroless)
- PHP/Laravel: multi-stage (composer → php:8.3-fpm + nginx)
- Python: multi-stage (python:3.12-slim)
- Rust: multi-stage (rust:latest → distroless)
- Go: multi-stage (golang:latest → scratch)

#### Deployment (`crates/klyron_deploy`)

| Command | Platform | Config Generated |
|---------|----------|-----------------|
| `klyron deploy vercel` | Vercel | `vercel.json` + project config |
| `klyron deploy cloudflare` | Cloudflare Workers/Pages | `wrangler.toml` |
| `klyron deploy railway` | Railway | `railway.json` |
| `klyron deploy fly` | Fly.io | `fly.toml` |
| `klyron deploy docker` | Docker/VPS | docker-compose.yml + nginx config |

Deployment features:
- Auto-detect framework → choose deployment strategy
- Generate platform-specific config files
- Inject environment variables from `.env` or `klyron.toml`
- Preview deployments: `klyron deploy vercel --preview`
- One-command deploy: `klyron deploy` (auto-detect platform from git remote)

#### Node Compatibility Checker (`crates/klyron_compat`)

| Command | Description |
|---------|-------------|
| `klyron compat check` | Check if project is fully compatible with Klyron |
| `klyron compat react` | Check React compatibility |
| `klyron compat next` | Check Next.js compatibility |
| `klyron compat astro` | Check Astro compatibility |
| `klyron compat nest` | Check NestJS compatibility |
| `klyron compat prisma` | Check Prisma compatibility |

Compat check report:
- ✅ Fully compatible — runs without changes
- ⚠️ Mostly compatible — minor issues listed
- ❌ Incompatible — blocking issues with workarounds

#### NAPI (`crates/klyron_napi`)

| Command | Description |
|---------|-------------|
| `klyron napi build` | Build N-API native module from C/C++ source |
| `klyron napi generate` | Generate N-API binding boilerplate |
| `klyron napi test` | Test N-API native module |

#### CLI Commands Working After Phase 9
```bash
# Plugin
klyron plugin install my-plugin
klyron plugin list
klyron plugin create my-plugin

# Workspace
klyron workspace init
klyron workspace add frontend
klyron workspace run build

# Docker
klyron docker init
klyron docker build
klyron docker run

# Deploy
klyron deploy vercel
klyron deploy cloudflare
klyron deploy fly

# Compat
klyron compat check
klyron compat react
klyron compat next

# NAPI
klyron napi build ./native/src
klyron napi generate --template=node
klyron napi test
```

#### Testable Deliverables
- `klyron plugin install` loads WASM plugin, hooks fire on command
- `klyron workspace init && workspace add frontend && workspace run build` works
- `klyron docker init` generates valid Dockerfile + docker-compose.yml
- `klyron deploy vercel` generates vercel.json, runs `vc --prod`
- `klyron compat check` reports correct compat status for existing projects
- `klyron napi build` compiles C/C++ to .node binary

---

### Phase 10 — AI & Performance Optimization (Q4 2028)

**Goal:** AI-assisted development + enterprise-grade performance.

#### AI Crate (`crates/klyron_ai`)

| Command | Capability | Backend |
|---------|------------|---------|
| `klyron ai generate` | Generate code from natural language prompt | Local LLM (llama.cpp) + OpenAI/Anthropic API |
| `klyron ai optimize` | Analyze and optimize code performance | Static analysis + LLM suggestions |
| `klyron ai review` | Code review with AI | LLM-based review with diff output |
| `klyron ai docs` | Generate documentation from code | JSDoc/TSDoc + README generation |
| `klyron ai test` | Generate unit tests from implementation | LLM generates test files |
| `klyron ai migrate` | Migrate between frameworks | e.g., React → Vue, Express → Hono, Laravel blade → Inertia |

AI migration paths:
- React → Vue, Vue → React
- Express → Fastify, Express → Hono
- Next.js Pages Router → App Router
- Laravel Blade → Livewire, Laravel Blade → Inertia
- JavaScript → TypeScript
- CJS → ESM

#### Performance Optimization

| Feature | Target | Implementation |
|---------|--------|---------------|
| V8 snapshot caching | < 30ms cold start | Pre-compile runtime bootstrap to snapshot |
| Engine lazy-spawn | < 5ms warm start | Pool V8 isolates, reuse |
| Parallel engine protocol | 2x throughput | Binary framing + tokio channels |
| Module caching | < 1ms resolution | SQLite index of resolved modules |
| Incremental compilation | < 100ms rebuild | Rust sccache + cargo-chef |
| Parallel test execution | 4x faster | Test splitting + worker pool |
| WASM JIT compilation | < 10μs overhead | Pre-compile WASM plugins |

#### Enterprise Features

| Feature | Description |
|---------|-------------|
| SSO/SAML | Enterprise single sign-on for registry |
| Audit logging | All CLI operations logged to audit trail |
| Team workspace | Multi-user workspace management |
| Role-based access | Admin, developer, viewer roles |
| License enforcement | Commercial license validation |
| Telemetry | Usage analytics (`klyron telemetry` on/off) |
| Private registry | Self-hosted package registry |
| Compliance | SOC 2, GDPR, HIPAA audit trails |

#### CLI Commands Working After Phase 10
```bash
# AI
klyron ai generate "Create a React component that fetches users"
klyron ai optimize src/routes.ts
klyron ai review src/components/
klyron ai docs src/utils/
klyron ai test src/services/userService.ts
klyron ai migrate --from=react --to=vue src/components/

# Enterprise
klyron telemetry on
klyron config set private-registry https://registry.mycompany.com

# Performance
klyron info                    # Shows startup time, memory usage
klyron doctor                  # Full health check with perf metrics
```

#### Testable Deliverables
- `klyron ai generate "hello world express server"` creates server code
- `klyron ai migrate --from=react --to=vue` converts React components to Vue
- Cold start < 30ms, warm start < 5ms
- `klyron telemetry` toggles on/off, data collected
- `klyron info` shows version, uptime, memory, cache stats

---

### Visual Timeline

```
Q3 2026    | Q4 2026    | Q1 2027    | Q2 2027    | Q3 2027    | Q4 2027    | Q1 2028    | Q2 2028    | Q3 2028    | Q4 2028
Phase 1    | Phase 2    | Phase 3    | Phase 4    | Phase 5    | Phase 6    | Phase 7    | Phase 8    | Phase 9    | Phase 10
Modular    | Web API    | Node.js    | Framework  | Testing    | Laravel    | Database   | Registry   | Plugin     | AI &
           | HTTP       | Compat     | Adapters   | Linting    | Ecosystem  | ORM        | SDK        | Docker     | Performance
           | Extensions | Module Sys | Scaffold   | Benchmark  |            | Prisma     | Package    | Deploy     | Enterprise
           |            | NAPI       | v2         | TypeCheck  |            | Drizzle    | Mgmt       | NAPI       |
```

### Framework Feature Completeness

| Feature | Phase 0 | P1 | P2 | P3 | P4 | P5 | P6 | P7 | P8 | P9 | P10 |
|---------|:-------:|:--:|:--:|:--:|:--:|:--:|:--:|:--:|:--:|:--:|:---:|
| JS Runtime (V8/Boa/QuickJS) | ⚡ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Polyglot Engines (10 lang) | ⚡ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Web API (fetch, URL, etc) | ❌ | ❌ | ⚡ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| HTTP Server | ❌ | ❌ | ⚡ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| TS/JSX Transpile | ⚡ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Module Resolution (ESM/CJS) | ❌ | ❌ | ❌ | ⚡ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Node.js Compat (require, process, Buffer) | ❌ | ❌ | ❌ | ⚡ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Package Manager (add/install/remove) | ⚡ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Multi-Registry (npm/PyPI/Gems/Cargo/Packagist/Go) | ⚡ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Registry (login/publish/search) | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ⚡ | ✅ | ✅ |
| Framework Scaffold (10 frontend + 9 backend) | ⚡ | ✅ | ✅ | ✅ | ⚡ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Framework Version Detection | ❌ | ❌ | ❌ | ❌ | ⚡ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Laravel (v9/v10/v11/v12/v13, 9 stacks) | ⚡ | ✅ | ✅ | ✅ | ✅ | ✅ | ⚡ | ✅ | ✅ | ✅ | ✅ |
| Laravel Artisan Wrapper (50+ cmds) | ⚡ | ✅ | ✅ | ✅ | ✅ | ✅ | ⚡ | ✅ | ✅ | ✅ | ✅ |
| DB / ORM (Prisma/Drizzle/TypeORM/dll) | ⚡ | ✅ | ✅ | ⚡ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Dev Server (HMR, watch, hot) | ⚡ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Build (minify, sourcemap, target) | ⚡ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Test Runner (vitest/jest/phpunit/pytest) | ❌ | ❌ | ❌ | ❌ | ❌ | ⚡ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Linter / Formatter Auto-Detect | ❌ | ❌ | ❌ | ❌ | ❌ | ⚡ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Benchmark Suite | ❌ | ❌ | ❌ | ❌ | ❌ | ⚡ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Type Checker (tsc) | ❌ | ❌ | ❌ | ❌ | ❌ | ⚡ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Plugin System (WASM) | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ⚡ | ✅ |
| Workspace / Monorepo | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ⚡ | ✅ |
| Docker (init/build/run) | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ⚡ | ✅ |
| Deploy (Vercel/CF/Railway/Fly) | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ⚡ | ✅ |
| Node Compat Checker | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ⚡ | ✅ |
| Multi-Language SDK (JS/TS/Rust/C++/PHP) | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ⚡ | ✅ | ✅ | ✅ |
| AI Features | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ⚡ |
| Enterprise (SSO/audit/private) | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ⚡ |

**Legend:** ❌ Not started | 🔧 In development | ⚡ Active (this phase) | ✅ Complete

---

## 17. Contributing Guide

### 17.1 Prerequisites

- Rust 2024 edition (rust-toolchain.toml)
- Node.js 22+
- PHP 8.1+
- Python 3.10+
- Ruby 3.0+
- Go 1.21+
- Zig 0.11+

### 17.2 Development Setup

```bash
git clone https://github.com/anomalyco/klyronjs
cd klyronjs

# Build
cargo build -p klyron

# Run tests
cargo test

# Run specific engine engine
./target/debug/klyron ts 'console.log("hello")'
./target/debug/klyron php 'echo "hello";'
./target/debug/klyron rs 'fn main() { println!("hello"); }'
```

### 17.3 Adding a New Engine

1. Create `crates/klyron_engine/engines/<lang>/`
2. Implement `Engine` trait
3. Add binary or script
4. Register in `crates/klyron_cli`
5. Add scaffold template if applicable
6. Add tests
7. Update `klyron info` output

### 17.4 Adding a New Scaffold Template

1. Create template folder in `scaffolds/` (e.g., `scaffolds/frontend/`, `scaffolds/backend/`, `scaffolds/laravel/`, or `scaffolds/polyglot/`)
2. Implement `scaffold_<name>()` function
3. Add CLI command variant
4. Register in match arm
5. Add to `klyron info` output
6. Add tests

### 17.5 Code Style

- Rust: `cargo fmt` + `cargo clippy` (no warnings)
- TypeScript: strict mode, no `any`
- PHP: PSR-12, strict types
- Python: PEP 8, type hints
- Ruby: RuboCop defaults
- Go: gofmt, golangci-lint

### 17.6 Commit Convention

```
<type>(<scope>): <description>

type: feat | fix | refactor | perf | test | docs | chore
scope: cli | engine | scaffold | runtime | pm | db | core

Examples:
  feat(engine): add format action to C engine
  fix(cli): handle empty pipe input for cc command
  refactor(scaffold): extract framework adapter trait
  test(engine): add integration tests for PHP engine
  docs: update PLAN.md with Phase 2 details
```
