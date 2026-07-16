# Klyron Total Upgrade Plan — Deep Architecture & Implementation
## From Universal Polyglot Runtime to Next-Gen Developer Platform

---

# 📋 Fase 0: Foundation — Zero Compilation Error

## 0.0 — Pre-existing fixes (done)
- [x] `.cargo/config.toml`: `jobs = 0` → `jobs = 4`
- [x] `crates/klyron_engine/Cargo.toml`: V8 feature → `dep:klyron-core`
- [x] `crates/klyron_engine/src/engine.rs`: V8Adapter → `klyron_core::Runtime`
- [x] `crates/klyron_engine/src/cache.rs`: double borrow fix
- [x] `crates/klyron_engine/src/lib.rs`: borrow after move fix
- [x] `engines/quickjs/Cargo.toml` + `engines/jsc/Cargo.toml`: add `tracing` dep
- [x] `crates/klyron_pm/src/lib.rs`: tutup `mod tests`

## 0.1 — klyron_pm (12 errors)
- [ ] Add missing deps: `ed25519-dalek` (v2.0), `rand` (v0.8), `sha1` (v0.10 via sha1 crate), `base64` (v0.22 via base64 crate) ke Cargo.toml
- [ ] `scripts.rs:306`: fix deref pattern `|(_, &deg)|` → `|&(_, deg)|` (edition 2024 binding mode change)
- [ ] Cleanup 11 warnings (unused imports, variables)
- [ ] `lib.rs:2-10`: hapus unused imports `once_cell::Lazy`, `regex::Regex`, `semver::{Version,VersionReq}`, `HashSet`, `Command`, `SystemTime`

## 0.2 — klyron_plugin (4 errors)
- [ ] `runtime.rs:40`: hapus `?` pada `&mut Config` — config builder pattern, ubah jadi method call biasa
- [ ] `lib.rs:285`: `wasm_dest.clone()` untuk moved value (PathBuf not Copy)
- [ ] `lib.rs:439,446`: hapus `ref` di implicit borrowing patterns (edition 2024)
- [ ] `lib.rs:522,526`: restructure `enable_plugin()` / `disable_plugin()` — clone `info.enabled` before `self.emit()`

## 0.3 — klyron_http (3 errors)
- [ ] Add `parking_lot` (v0.12), `tracing` (v0.1) ke Cargo.toml
- [ ] Fix `reqwest::RequestBuilder::json()` — pake `.body(serde_json::to_string(&val)?)` + `.header("Content-Type", "application/json")`
- [ ] Cek kompatibilitas `reqwest` v0.12 API dengan TLS backend

## 0.4 — klyron_fs (3 errors)
- [ ] Add `tracing` (v0.1) ke Cargo.toml
- [ ] Fix `tokio::sync::Semaphore` → tambah `features = ["sync"]` di tokio dependency

## 0.5 — klyron_cache (5 errors)
- [ ] Add `tracing` (v0.1) ke Cargo.toml
- [ ] `Instant: Serialize/Deserialize` — implement `SerializableInstant(u64)` newtype wrapping `.elapsed().as_nanos()` atau gunakan `chrono::Utc::now()`

## 0.6 — klyron_arena (2 errors)
- [ ] Fix type mismatch — generic lifetime parameter atau associated type binding

## 0.7 — klyron_serde (4 errors)
- [ ] `Deserializer` private → ganti dengan `simd_json::to_value()` atau `simd_json::from_slice()`
- [ ] `String::with_capacity` di const → ganti `const EMPTY: String = String::new()` dengan `lazy_static!` atau `once_cell::sync::Lazy<String>`
- [ ] `Cursor<&[u8]>: Write` → `Cursor<Vec<u8>>` via `Vec::from(bytes)`
- [ ] `simd_json::Deserializer::new()` → cek API simd_json v0.6 terbaru: `simd_json::Serde::from_slice()`

## 0.8 — klyron_bench (12 errors — transitif dari klyron_pm)
- [ ] Akan fix otomatis setelah 0.1 selesai (sama-sama butuh `ed25519-dalek`, `sha1`, `base64`)

## 0.9 — Warning cleanup across workspace
- [ ] Unused imports: `once_cell`, `regex`, `semver`, `HashSet`, `Command`, `SystemTime`
- [ ] Unused variables: `key` di jsc/binding.rs, `scripts_j` di pm/scripts.rs, `plugin` di plugin/lib.rs
- [ ] Unused mut: `with_registry_config` di pm/rate_limit.rs
- [ ] Unused constant: `DEFAULT_MAX_MEMORY_SIZE_MB` di engine/cache.rs

---

# 🔧 Fase 1: Build System & Infrastructure Overhaul

## 1.1 — Architecture Decision: Rust Edition

### Problem
Workspace uses Rust edition 2024. Edition 2024 changes:
- Binding modes: `ref` and `&` in destructuring patterns no longer valid when matching on references
- `unsafe` blocks in `let` and `match` require different syntax
- `impl Trait` capture rules changed
- `extern` block semantics changed

### Analysis
| Factor | Edition 2021 | Edition 2024 |
|---|---|---|
| Stability | Mature, all crates support | Very new, some crates lag |
| Pattern fixes needed | None | ~15 sites across codebase |
| Future-proof | No | Yes, 3+ years |
| CI compatibility | All toolchains | Need nightly or very recent stable |

### Decision
→ **Target: Edition 2024** but keep `edition = "2021"` temporarily for crates with heavy pattern usage until all fixes are complete.

### Implementation
```toml
# Workspace Cargo.toml
[workspace.package]
edition = "2024"  # target
# Individual crates can override:
[package]
edition = "2021"  # temporary for crates with pattern issues
```
Migration path: fix patterns crate-by-crate, switch to 2024 when clean.

## 1.2 — Workspace Dependency Unification

### Current state
~40 crates, each with duplicate dependency declarations. Version drift possible.

### Target
```toml
# workspace Cargo.toml
[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
tracing = "0.1"
tracing-subscriber = "0.3"
anyhow = "1"
thiserror = "2"
serde_json = "1"
reqwest = { version = "0.12", default-features = false, features = ["json"] }
ed25519-dalek = "2"
sha1 = "0.10"
base64 = "0.22"
parking_lot = "0.12"
chrono = { version = "0.4", features = ["serde"] }
```

### Benefit
- Single version truth
- Easier upgrades (change one line)
- Faster `cargo check` (unified dep graph)
- Reduced compile times (shared features)

## 1.3 — CI/CD Pipeline Architecture

### Pipeline stages
```yaml
# .github/workflows/ci.yml
jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - cargo fmt --check
      - cargo clippy --workspace -- -D warnings
      - cargo deny check licenses advisories sources

  test:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, nightly]
    steps:
      - cargo test --workspace --all-features

  build:
    strategy:
      matrix:
        target: [x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu,
                 x86_64-apple-darwin, aarch64-apple-darwin,
                 x86_64-pc-windows-msvc]
    steps:
      - cross build --release --target ${{ matrix.target }}

  release:
    needs: [lint, test, build]
    if: startsWith(github.ref, 'refs/tags/v')
    steps:
      - gh release create
      - Upload binaries for all targets
      - cargo publish (all crates)
```

### Toolchain requirements
```toml
# rust-toolchain.toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
targets = [
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu",
    "x86_64-apple-darwin",
    "aarch64-apple-darwin",
    "x86_64-pc-windows-msvc",
]
```

### Auxiliary checks
- `cargo audit` — vulnerability scanning (via GitHub Actions `audit-check`)
- `cargo deny` — license compliance + duplicate dep detection
- `cargo outdated` — dependency freshness report
- `cargo udeps` — unused dependency detection
- `cargo machete` — unused crate detection (alternative)

## 1.4 — Feature Flag Architecture

### Design
```toml
[features]
default = ["engine-v8"]
full = ["engine-v8", "engine-boa", "engine-quickjs", "engine-jsc",
        "wasm-plugins", "tracing", "simd", "inferno", "ai"]

# Engine backends
engine-v8 = ["dep:klyron-core"]
engine-boa = ["dep:klyron-engine-boa"]
engine-quickjs = ["dep:klyron-engine-quickjs"]
engine-jsc = ["dep:klyron-engine-jsc"]
all-engines = ["engine-v8", "engine-boa", "engine-quickjs", "engine-jsc"]

# Performance
simd = ["dep:simd-json"]
inferno = ["dep:inferno"]  # flamegraph profiling

# Extensions
wasm-plugins = ["dep:wasmtime"]
ai = ["dep:reqwest"]  # LLM integration

# Observability
tracing = ["dep:tracing", "dep:tracing-subscriber"]
```

### Feature resolution strategy
- Default: V8 only (fastest compile, production-ready)
- `--features full` for development machines (all engines + tooling)
- Auto-detection at runtime for CLI features (e.g., `tracing` enabled when `KLYRON_LOG=debug`)

## 1.5 — Binary Distribution & Updater

### Update protocol
```rust
// klyron_updater crate architecture
pub struct UpdateManifest {
    pub version: semver::Version,
    pub platform: String,           // "x86_64-unknown-linux-gnu"
    pub url: String,                // download URL
    pub sha256: String,             // integrity hash
    pub signature: String,          // ed25519 signature
    pub requires: Vec<semver::VersionReq>, // version constraints
    pub changelog: Vec<String>,     // notable changes
}

// Update flow
// 1. GET https://dist.klyron.dev/updates/latest -> UpdateManifest
// 2. Verify signature against known public key (TOFU)
// 3. Compare version with current
// 4. If newer: download binary to ~/.klyron/updates/
// 5. Verify sha256 hash
// 6. Atomic swap: rename current binary -> backup
// 7. Run new binary with --verify flag
// 8. On success: remove backup. On failure: restore.
```

### Installation methods
| Method | Command | Notes |
|---|---|---|
| Shell script | `curl -fsSL https://klyron.dev/install.sh | sh` | Auto-detect platform |
| Homebrew | `brew install klyron` | macOS only |
| Scoop | `scoop bucket add klyron; scoop install klyron` | Windows only |
| npm | `npm install -g klyron` | Requires Node.js |
| Cargo | `cargo install klyron-cli` | Requires Rust toolchain |
| Docker | `docker pull klyron/klyron:latest` | Containerized |
| Binary (manual) | Download from GitHub Releases | tarball/zip |

## 1.6 — Project Structure Standardization

### Target layout
```
klyron/
├── src/cli/                 # Binary entrypoint (thin, ~6 lines)
│   └── src/main.rs          # -> klyron_cli::run_cli()
├── src/core/               # Deno-core runtime (klyron-core crate)
├── src/ext/                 # 13 Deno-core extensions
├── crates/                  # ~40 lib crates
│   ├── klyron_cli/          # CLI implementation
│   ├── klyron_engine/       # Engine abstraction + AutoSwitcher
│   ├── klyron_pm/           # Package manager
│   ├── klyron_plugin/       # WASM plugin host
│   ├── klyron_http/         # HTTP server (axum)
│   ├── klyron_runtime/      # Isolate pool, snapshot cache
│   ├── klyron_loader/       # Module resolution
│   ├── klyron_config/       # Layered config
│   ├── klyron_test/         # Test runner
│   ├── klyron_bundler/      # Bundler
│   ├── klyron_linter/       # Linter
│   ├── klyron_formatter/    # Formatter
│   ├── klyron_cache/        # Caching layer (LFU sharded)
│   └── klyron_common/       # Shared types [NEW]
├── engines/                 # JS engine backends
│   ├── v8/                  # rusty_v8 + deno_core
│   ├── boa/                 # boa_engine
│   ├── quickjs/             # qjs-sys
│   └── jsc/                 # JSC C API
├── php/                     # PHP framework support files
├── adapters/                # Framework adapters (27 total)
├── orm/                     # ORM adapters (8 total)
├── scaffolds/               # Scaffold templates
├── examples/                # Example projects (01-08)
├── tests/                   # Integration tests (~4200 lines)
├── docs/                    # Documentation
└── fuzz/                    # Fuzz targets
```

---

# ⚙️ Fase 2: Engine Architecture — Deep Native Integration

## 2.0 — Architecture Overview

### Engine layer diagram
```
┌─────────────────────────────────────────────────────────────────┐
│                        klyron_cli (CLI)                          │
├─────────────────────────────────────────────────────────────────┤
│                         klyron_engine                             │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                    AutoSwitcher v2                        │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐    │   │
│  │  │  V8 via  │ │   Boa    │ │  QuickJS │ │   JSC    │    │   │
│  │  │klyron_core│ │(debug)   │ │(short)   │ │(batch)    │    │   │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘    │   │
│  │  ┌──────────────────────────────────────────────────┐    │   │
│  │  │           Fallback Chain / Hot Path              │    │   │
│  │  └──────────────────────────────────────────────────┘    │   │
│  └──────────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────┤
│                    EngineProcess (polyglot)                       │
│  PHP/Python/Ruby/Go/Zig/Rust/C/C++ via subprocess/WASM          │
├─────────────────────────────────────────────────────────────────┤
│                   klyron_core (deno_core JsRuntime)               │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────────┐   │
│  │  V8 Isolate│ │  Extensions│ │  Modules  │ │  Snapshot cache │   │
│  └──────────┘ └──────────┘ └──────────┘ └──────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

### Engine selection algorithm (AutoSwitcher v2)
```rust
pub struct EngineSelector {
    // Per-engine latency tracking (sliding window)
    v8_latency: ExponentialMovingAverage,
    boa_latency: ExponentialMovingAverage,
    quickjs_latency: ExponentialMovingAverage,
    jsc_latency: ExponentialMovingAverage,

    // Script characteristics for prediction
    script_classifier: ScriptClassifier,

    // Configuration
    min_samples: usize,          // 5 — minimum samples before switching
    switch_threshold: f64,        // 1.5 — 50% faster to switch
    hot_path_threshold: usize,    // 100 — consecutive same-script
}

impl EngineSelector {
    pub fn select(&mut self, script: &str) -> EngineKind {
        // 1. Check if script is on hot path (same script >= 100x)
        if self.is_hot_path(script) {
            return EngineKind::V8;  // Hot path -> V8 (JIT)
        }

        // 2. Classify script characteristics
        let features = self.script_classifier.extract(script);

        // 3. Predict best engine
        //    - Very short (< 10ms expected): QuickJS (fast startup)
        //    - Debug mode: Boa (better stack traces)
        //    - Batch processing: JSC
        //    - Everything else: V8
        self.predict(features)
    }
}
```

## 2.1 — V8 via klyron_core (done ✅)

### What exists
```rust
// src/core/src/runtime.rs
pub struct Runtime {
    inner: JsRuntime,  // deno_core::JsRuntime
}

impl Runtime {
    pub fn eval(&mut self, code: &str) -> Result<String, EvalError> {
        // Uses deno_core's v8::Isolate under the hood
        let result = self.inner.execute_script("<eval>", code)?;
        let scope = &mut self.inner.handle_scope();
        let value = result.open(scope);
        Ok(js_value_to_string(scope, value))
    }
}
```

### Verified: 17/17 test lolos ✅
- `test_eval_number`, `test_eval_string`, `test_eval_boolean`,
- `test_eval_array`, `test_eval_object`, `test_eval_null_undefined`,
- `test_eval_function_call`, `test_eval_template_literal`,
- `test_eval_error`, `test_eval_syntax_error`,
- `test_eval_promise_object`, `test_eval_with_typescript_enum`,
- `test_execute_script`, `test_execute_script_with_name`,
- `test_runtime_builder_default`, `test_runtime_builder_no_typescript`,
- `test_multiple_evals_same_runtime`

### What's needed
- [ ] **Memory limit enforcement**:
  ```rust
  // Rusty_v8 API
  let mut isolate = v8::Isolate::new(params);
  isolate.set_memory_limit(512 * 1024 * 1024); // 512 MB
  isolate.set_sampling_interval(100); // 100ms for profiler
  ```
- [ ] **Snapshot serialization**:
  ```rust
  // Warm startup via snapshot
  let snapshot = runtime.snapshot()?;  // ~10MB binary blob
  std::fs::write("cache/klyron.snap", snapshot)?;
  // Load snapshot: startup from 300ms -> 15ms
  ```
- [ ] **Isolate pooling**:
  ```rust
  pub struct IsolatePool {
      pool: Vec<Runtime>,
      max_size: usize,
      ttl: Duration,
  }
  // Reuse isolates: avoid cold-start per eval
  ```

## 2.2 — Boa Engine (Debug/Dev)

### Implementation plan
```rust
// engines/boa/src/binding.rs
use boa_engine::{Context, Source};

pub struct BoaEngine {
    context: Context,
}

impl BoaEngine {
    pub fn new() -> Self {
        let mut context = Context::default();
        // Add console API
        context.register_global_property("console", console_object, Attribute::all());
        Self { context }
    }

    pub fn eval(&mut self, code: &str) -> Result<String, String> {
        let result = self.context.eval(Source::from_bytes(code))
            .map_err(|e| format!("Boa eval error: {}", e))?;
        Ok(result.to_string(&mut self.context)
            .map_err(|e| format!("Boa display error: {}", e))?)
    }
}
```
- [ ] Compile + link into klyron_engine
- [ ] Feature flag: `engine-boa`
- [ ] Use case: `klyron dev --debug` (better stack traces)

### Performance target
| Metric | V8 | Boa | QuickJS | JSC |
|---|---|---|---|---|
| Cold start | ~50ms | ~5ms | ~2ms | ~3ms |
| eval("1+1") | ~3ms | ~8ms | ~1ms | ~2ms |
| 10K ops loop | 0.5ms | 15ms | 2ms | 4ms |
| Stack trace quality | Good | Excellent | Good | Good |

## 2.3 — QuickJS Engine (Short Scripts)

### Implementation notes
```rust
// engines/quickjs/src/binding.rs
// Uses qjs-sys FFI (C bindings to QuickJS)
pub struct QuickJSContext {
    runtime: *mut sys::JSRuntime,
    context: *mut sys::JSContext,
    memory_pools: Arc<Mutex<Vec<Vec<u8>>>>,
}
// Memory pool: pre-allocate chunks to avoid malloc per eval
// Sandbox: set global object to empty for untrusted scripts
```

- [ ] Compile verification (tracing dep added ✅)
- [ ] Memory pool optimization — reuse allocations
- [ ] Sandbox mode — limited API for untrusted input

## 2.4 — JSC Engine (Batch Processing)

### Implementation notes
```rust
// engines/jsc/src/binding.rs
// Uses JavaScriptCore C API
pub struct JSCObject {
    context: *mut sys::OpaqueJSContext,
    group: *mut sys::OpaqueJSContextGroup,
}
// Batch mode: keep context alive across multiple scripts
// Garbage collection: manual trigger between batches
```

- [ ] Compile verification (tracing dep added ✅)
- [ ] Batch execution pipeline testing
- [ ] macOS-specific optimization (JSC native on macOS)

## 2.5 — AutoSwitcher v2: Predictive Engine Selection

### Algorithm detail
```rust
pub struct ScriptFeatures {
    pub byte_size: usize,
    pub has_loops: bool,
    pub has_promises: bool,
    pub has_async: bool,
    pub import_count: usize,
    pub top_level_await: bool,
    pub expected_runtime_ms: f64,
}

// Decision tree (simplified)
fn predict(features: &ScriptFeatures) -> EngineKind {
    match features {
        // Very short scripts: QuickJS (fastest startup)
        f if f.expected_runtime_ms < 5.0 => EngineKind::QuickJS,
        // Loops + long runtime: V8 (JIT compilation wins)
        f if f.has_loops && f.expected_runtime_ms > 100.0 => EngineKind::V8,
        // Debug mode: Boa (readable stack traces)
        f if f.has_async && f.expected_runtime_ms < 50.0 => EngineKind::Boa,
        // Batch (no interactivity needed): JSC
        f if f.import_count > 50 => EngineKind::JSC,
        // Default: V8
        _ => EngineKind::V8,
    }
}
```

### Fallback chain
```
Attempt: V8 (primary)
  ├─ Success → return
  └─ Failure (OOM/crash) → fallback
     Attempt: QuickJS (lightweight)
       ├─ Success → return
       └─ Failure → fallback
          Attempt: Boa (debug)
            ├─ Success → return
            └─ Failure → return error with all fallback traces
```

## 2.6 — Polyglot Engine Evolution: Subprocess → WASM

### Architecture decision: Subprocess vs WASM

| Factor | Subprocess | WASM |
|---|---|---|
| Startup | ~50-200ms (process spawn) | ~1-5ms |
| Isolation | OS-level (memory safe) | Linear memory (sandboxed) |
| Performance | Native speed | ~60-80% native |
| Lang support | All languages | Limited to WASM targets |
| Memory limit | RLIMIT_AS | Configurable linear memory |
| File system | Full host FS | WASI pre-opens |

### Decision per language

| Language | Strategy | Rationale |
|---|---|---|
| PHP | WASM (php-wasm) + Fallback to subprocess | Eliminate system PHP dep |
| Python | WASM (pyodide) + Fallback | Deterministic, portable |
| Ruby | WASM (ruby-wasm) | Maturing ecosystem |
| Go | Subprocess (go build + exec) | WASM limitations for net/http |
| Rust | Native (compile via rustc, exec) | Klyron written in Rust |
| C/C++ | Subprocess (clang/gcc + exec) | Complex toolchain |
| Zig | Subprocess | Native binary compilation |

### WASM integration pattern
```rust
// Common trait for all engine runtimes
#[async_trait]
pub trait PolyglotEngine: Send + Sync {
    /// Name of the language
    fn language(&self) -> &'static str;

    /// Evaluate a script and return output
    async fn eval(&self, code: &str) -> Result<String, EngineError>;

    /// Check if the engine is available
    async fn health_check(&self) -> Result<(), EngineError>;

    /// Get engine metadata
    fn metadata(&self) -> EngineMetadata;
}

// WASM-based engine
pub struct WasmEngine {
    module: wasmtime::Module,
    store: wasmtime::Store<WasmState>,
    linker: wasmtime::Linker<WasmState>,
}

// Subprocess-based engine
pub struct SubprocessEngine {
    binary_path: PathBuf,
    protocol: JsonProtocol,  // stdin/stdout JSON
}
```

## 2.7 — Polyglot Module Resolution

### Cross-language imports
```javascript
// JavaScript importing PHP
import { renderBlade } from './views/welcome.blade.php';
// Klyron resolves:
// 1. File extension -> .php -> PHP engine
// 2. Load & compile Blade template via PHP engine
// 3. Return result as JS string
```

### Resolution algorithm
```rust
pub struct ModuleResolver {
    extensions: HashMap<String, EngineKind>,
    // .js -> JS, .ts -> JS, .php -> PHP, .py -> Python
    // .rb -> Ruby, .go -> Go, .rs -> Rust, .c -> C, .cpp -> C++
}

impl ModuleResolver {
    pub fn resolve(&self, specifier: &str, referrer: &str) -> Result<ResolvedModule> {
        let path = self.resolve_path(specifier, referrer)?;
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("js");
        let engine = self.extensions.get(ext).copied().unwrap_or(EngineKind::V8);
        Ok(ResolvedModule { path, engine })
    }
}
```

### FFI bridge (cross-language calls)
```rust
// Data translation layer
pub enum FFIValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(Vec<FFIValue>),
    Map(HashMap<String, FFIValue>),
    Buffer(Vec<u8>),
}

// Each engine implements Into<FFIValue> and From<FFIValue>
// for its native types
```

---

# 📦 Fase 3: Package Manager — npm+ Parity

## 3.0 — Architecture Overview

### Dependency resolution pipeline
```
Input: package.json + lockfile (optional)
  │
  ▼
┌─────────────────────────────────────┐
│         Dependency Resolver          │
│  ┌──────────┐  ┌──────────────────┐  │
│  │ Lockfile  │  │  Version Solver  │  │
│  │ Verifier  │  │  (SAT/SMT)       │  │
│  └──────────┘  └──────────────────┘  │
│  ┌──────────────────────────────────┐│
│  │  Tree Builder (hoisting/flat)     ││
│  └──────────────────────────────────┘│
└─────────────────────────────────────┘
  │
  ▼
┌─────────────────────────────────────┐
│         Package Fetcher              │
│  ┌──────────┐  ┌──────────────────┐  │
│  │ Registry  │  │  Cache Manager   │  │
│  │ Router    │  │  (local + remote)│  │
│  └──────────┘  └──────────────────┘  │
│  ┌──────────────────────────────────┐│
│  │  Integrity Verifier (SRI/Hash)    ││
│  └──────────────────────────────────┘│
└─────────────────────────────────────┘
  │
  ▼
┌─────────────────────────────────────┐
│         Package Installer            │
│  ┌──────────┐  ┌──────────────────┐  │
│  │ Extractor │  │  Lifecycle Hooks  │  │
│  │ (tar.gz)  │  │  (pre/postinstall)│  │
│  └──────────┘  └──────────────────┘  │
│  ┌──────────────────────────────────┐│
│  │  Binary Linking (node_modules/.bin)││
│  └──────────────────────────────────┘│
└─────────────────────────────────────┘
  │
  ▼
Output: node_modules/ + updated lockfile
```

### Key data structures
```rust
pub struct PackageJson {
    pub name: Option<String>,
    pub version: Option<semver::Version>,
    pub dependencies: BTreeMap<String, VersionReq>,
    pub dev_dependencies: BTreeMap<String, VersionReq>,
    pub peer_dependencies: BTreeMap<String, VersionReq>,
    pub optional_dependencies: BTreeMap<String, VersionReq>,
    pub workspaces: Option<Vec<String>>,
    pub scripts: BTreeMap<String, String>,
    pub bin: Option<BTreeMap<String, String>>,
    pub os: Option<Vec<String>>,           // "darwin", "linux", "win32"
    pub cpu: Option<Vec<String>>,           // "x64", "arm64"
    pub exports: Option<serde_json::Value>, // Conditional exports
    pub imports: Option<serde_json::Value>, // Self-referencing imports
}

pub struct LockfileV3 {
    pub name: Option<String>,
    pub lockfile_version: Option<u32>,
    pub packages: BTreeMap<String, LockfilePackage>,
    pub workspaces: Option<BTreeMap<String, String>>,
    pub metadata: Option<BTreeMap<String, String>>,
}

pub struct LockfilePackage {
    pub version: semver::Version,
    pub resolved: Option<String>,  // tarball URL
    pub integrity: Option<String>, // SRI hash
    pub dependencies: Option<BTreeMap<String, String>>,
    pub dev: Option<bool>,
    pub optional: Option<bool>,
    pub engines: Option<BTreeMap<String, String>>,
    pub os: Option<Vec<String>>,
    pub cpu: Option<Vec<String>>,
}
```

## 3.1 — Install Engine

### Algorithm
```rust
pub async fn install(config: InstallConfig) -> Result<InstallReport> {
    // 1. Parse package.json
    let manifest = PackageJson::load(&config.root.join("package.json"))?;

    // 2. Load lockfile if exists
    let existing = LockfileV3::load(&config.root.join("klyron.lock"))?;

    // 3. Resolve dependencies
    let resolution = match config.mode {
        InstallMode::Update => Resolver::solve_full(&manifest).await?,
        InstallMode::Frozen => {
            if existing.is_some() {
                Resolver::verify(&manifest, &existing.unwrap())?;
                existing.unwrap()
            } else {
                bail!("No lockfile found for frozen install")
            }
        }
        InstallMode::Clean => {
            Resolver::solve_full(&manifest).await? // ignore existing lockfile
        }
    };

    // 4. Fetch packages not in cache
    let fetcher = PackageFetcher::new(&config.registry);
    let mut installed = 0;
    for (name, pkg) in &resolution.packages {
        if !cache_has(&config.cache_dir, pkg) {
            fetcher.fetch_and_verify(pkg).await?;
            installed += 1;
        }
    }

    // 5. Extract to node_modules
    let linker = PackageLinker::new(&config.root);
    linker.link_all(&resolution)?;

    // 6. Run lifecycle hooks
    if !config.ignore_scripts {
        for (name, pkg) in &resolution.packages {
            run_lifecycle_hooks(&config.root, name, pkg, "install")?;
        }
    }

    // 7. Write lockfile
    resolution.save(&config.root.join("klyron.lock"))?;

    Ok(InstallReport {
        total: resolution.packages.len(),
        installed,
        from_cache: resolution.packages.len() - installed,
        duration: start.elapsed(),
    })
}
```

### Performance targets
| Operation | Target | npm (v10) | bun (v1.1) |
|---|---|---|---|
| `install` (empty) | < 5ms | ~200ms | ~5ms |
| `install express` (50 deps) | < 300ms | ~2s | ~200ms |
| `install` (1000 deps cached) | < 50ms | ~5s (no cache) | ~50ms |
| Lockfile parse (1000 deps) | < 1ms | ~10ms | ~0.5ms |
| Lockfile write (1000 deps) | < 1ms | ~5ms | ~1ms |

## 3.2 — SAT Solver for Dependency Resolution

### Algorithm choice: pubgrub (used by npm)
```rust
// Use pubgrub crate for dependency resolution
// https://crates.io/crates/pubgrub

use pubgrub::{
    solver::{resolve, Dependencies, OfflineDependencyProvider},
    range::Range,
    report::DefaultStringReporter,
    report::Reporter,
};

pub struct NpmDependencyProvider {
    registry: RegistryClient,
    cache: DependencyCache,
}

impl DependencyProvider<String, semver::Version> for NpmDependencyProvider {
    fn choose_version(&self, package: &String, range: &Range<semver::Version>)
        -> Option<semver::Version> {
        // Try cache first, then registry
        let versions = self.cache.get_versions(package)
            .or_else(|| self.registry.fetch_versions(package));
        versions?.into_iter()
            .filter(|v| range.contains(v))
            .max() // Prefer latest within range
    }

    fn get_dependencies(&self, package: &String, version: &semver::Version)
        -> Result<Dependencies<String, semver::Version>, Box<dyn Error>> {
        let manifest = self.registry.fetch_manifest(package, version)?;
        let deps: Vec<_> = manifest.dependencies.into_iter()
            .map(|(name, req)| (name, Range::from(req)))
            .collect();
        Ok(Dependencies::Available(deps))
    }
}
```

### Solver configuration
| Parameter | Value | Rationale |
|---|---|---|
| Backjumping | Enabled | Skip dead-end branches faster |
| Learning | 50 clauses | Remember conflicts |
| Version ordering | Descending | Prefer newer versions |
| Retry limit | 3 | Network failures |
| Timeout | 30s | Prevent infinite resolution |

## 3.3 — Lockfile Ecosystem

### Lockfile format support matrix
| Format | Read | Write | Convert To |
|---|---|---|---|
| npm lockfile v1 | ✅ | ✅ | Klyron v3 |
| npm lockfile v2 | ✅ | ✅ | Klyron v3 |
| npm lockfile v3 | ✅ | ✅ | (native) |
| yarn.lock v1 | ✅ | ❌ | Klyron v3 |
| yarn.lock v2 | ✅ | ❌ | Klyron v3 |
| pnpm-lock.yaml v5 | ✅ | ❌ | Klyron v3 |
| pnpm-lock.yaml v6 | ✅ | ❌ | Klyron v3 |

### Binary lockfile format (Klyron-native)
```rust
// Binary format for speed (mmap-able)
#[derive(Bincode)]
pub struct BinaryLockfile {
    pub magic: [u8; 8],         // "KLYRONLF"
    pub version: u32,           // 1
    pub created_at: u64,        // unix seconds
    pub content_hash: [u8; 32], // blake3 of package.json
    pub entries: Vec<BinaryEntry>,
    pub checksum: [u8; 32],     // blake3 of everything above
}

// Design goals:
// - Parse < 1ms for 100K entries
// - Memory-map friendly (zero-copy read)
// - Smaller than JSON (2-5x compression)
// - Backward-compatible JSON export: `klyron lockfile export`
```

## 3.4 — Package Publish Workflow

### CLI flow
```
$ klyron publish
  → Run prepublishOnly script (fail if any)
  → Bump version (if --version=patch)
  → Git: tag + commit (if --git-tag)
  → Build: `klyron pack` (creates .tgz)
  → Verify integrity (hash + sign)
  → Upload to registry (retry 3x)
  → Run postpublish script
  → Print URL: https://registry.klyron.dev/packages/@scope/name/v1.2.3
```

### Pack algorithm
```rust
pub fn pack(manifest: &PackageJson, root: &Path) -> Result<Vec<u8>> {
    // 1. Resolve included files (package.json files field + .gitignore)
    let files = resolve_pack_files(manifest, root)?;

    // 2. Create tar.gz stream
    let mut buffer = Vec::new();
    let mut archive = tar::Builder::new(GzEncoder::new(&mut buffer, Compression::best()));

    // 3. Add package.json (normalized)
    archive.append_file("package/package.json", manifest.to_normalized_json())?;

    // 4. Add all included files
    for file in &files {
        let path = root.join(file);
        archive.append_path_with_name(&path, format!("package/{}", file))?;
    }

    // 5. Add signature if signing key present
    if let Some(key) = load_signing_key() {
        let signature = sign_tarball(&buffer, &key)?;
        archive.append_data("package/.klyron_signature", "package/.klyron_signature",
            signature.as_bytes())?;
    }

    archive.finish()?;
    Ok(buffer) // returns .tgz bytes
}
```

## 3.5 — Multi-Registry Support

### Registry configuration
```yaml
# ~/.klyron/config.yaml
registries:
  npmjs:
    url: https://registry.npmjs.org
    type: npm
    auth: token
    token: $NPM_TOKEN  # or encrypted in keychain

  github:
    url: https://npm.pkg.github.com
    type: npm
    scope: "@myorg"
    auth: token
    token: $GITHUB_TOKEN

  verdaccio:
    url: http://localhost:4873
    type: npm
    auth: basic
    username: admin
    password: $VERDACCIO_PASSWORD

  packagist:
    url: https://repo.packagist.org
    type: composer
    # Future: PHP package support via same CLI
```

### Registry routing
```rust
pub fn resolve_registry(package: &str, config: &Config) -> &RegistryConfig {
    // 1. Check scope matching (@scope -> specific registry)
    if let Some(scope) = package.split('/').next() {
        if let Some(reg) = config.registries.iter()
            .find(|r| r.scope.as_deref() == Some(scope)) {
            return reg;
        }
    }

    // 2. Check package-specific override
    if let Some(reg) = config.package_registries.get(package) {
        return reg;
    }

    // 3. Default registry
    &config.default_registry
}
```

## 3.6 — Audit Engine

### Vulnerability database integration
```rust
pub struct AuditEngine {
    // Multiple data sources
    osv_db: OsvDatabase,     // Open Source Vulnerabilities
    gh_advisory: GithubAdvisoryDb, // GitHub Advisory Database
    nvd: NvdFeed,            // NIST NVD

    // Cached for performance
    local_advisories: Vec<Advisory>,
    last_updated: DateTime<Utc>,
}

pub struct Advisory {
    pub id: String,        // "GHSA-xxxx-xxxx-xxxx" or "CVE-2024-xxxxx"
    pub package: String,
    pub severity: Severity, // Critical, High, Medium, Low
    pub versions: Vec<VersionReq>,  // affected versions
    pub patched: Vec<VersionReq>,   // fixed versions
    pub title: String,
    pub description: String,
    pub cvss_score: Option<f32>,
    pub cwe: Option<Vec<String>>,
}

pub fn audit_tree(tree: &DependencyTree, db: &AdvisoryDb) -> AuditReport {
    let mut findings = Vec::new();
    for (package_name, version, path) in tree.walk() {
        if let Some(advisory) = db.query(package_name, version) {
            findings.push(Vulnerability {
                advisory,
                package: package_name.clone(),
                installed: version.clone(),
                path: path.clone(),
                fix_version: find_fix_version(tree, package_name, version),
                fix_type: classify_fix(advisory, tree, package_name),
            });
        }
    }
    AuditReport {
        total: tree.total_count(),
        vulnerable: findings.len(),
        critical: findings.iter().filter(|f| f.advisory.severity == Critical).count(),
        high: findings.iter().filter(|f| f.advisory.severity == High).count(),
        findings,
    }
}
```

### Performance target
| Database | Size | Query time | Update frequency |
|---|---|---|---|
| OSV | ~500MB | < 1ms (indexed) | Daily |
| GitHub Advisory | ~50MB | < 0.5ms | Real-time |
| NVD | ~2GB | < 5ms (indexed) | Daily |

## 3.7 — Lifecycle Hooks Engine

### Script execution model
```rust
pub fn run_lifecycle(pkg_dir: &Path, hook: &str, config: &LifecycleConfig) -> Result<()> {
    // 1. Read package.json scripts
    let pkg = PackageJson::load(pkg_dir.join("package.json"))?;
    let script = match hook {
        "preinstall" => pkg.scripts.get("preinstall"),
        "install" => pkg.scripts.get("install"),
        "postinstall" => pkg.scripts.get("postinstall"),
        _ => None,
    };

    if let Some(script) = script {
        if config.ignore_scripts {
            return Ok(()); // Skip
        }

        // 2. Validate script safety (blocklist check)
        validate_script_safety(script)?;

        // 3. Execute in sandboxed environment
        let output = Command::new("sh")
            .arg("-c")
            .arg(script)
            .env_remove("KLYRON_TOKEN")  // scrub secrets
            .timeout(config.script_timeout) // 5 min default
            .output()?;

        // 4. Check exit code
        if !output.status.success() {
            bail!("Script '{}' failed with exit code {}", hook, output.status);
        }
    }
    Ok(())
}
```

### Script safety validation
```rust
// Blocklist of dangerous patterns
const DANGEROUS_PATTERNS: &[&str] = &[
    "rm -rf /",
    "rm -rf ~",
    "> /dev/sda",
    "format",
    "mkfs",
    "dd if=",
    ":(){ :|:& };:",  // Fork bomb
    "eval $(curl",
    "wget -O - | sh",
    "curl ... | bash",
    "chmod -R 777 /",
];

fn validate_script_safety(script: &str) -> Result<()> {
    for pattern in DANGEROUS_PATTERNS {
        if script.contains(pattern) {
            bail!("Blocked dangerous pattern: '{}' in install script", pattern);
        }
    }
    Ok(())
}
```

## 3.8 — npx Equivalent: `klyron x`

### Implementation
```rust
pub async fn execute_npx(package: &str, args: &[String]) -> Result<()> {
    // 1. Parse package specifier
    let (name, version) = parse_package_spec(package);

    // 2. Check cache (~/.klyron/_npx/)
    let cache_dir = dirs::home_dir()
        .unwrap()
        .join(".klyron/_npx")
        .join(&name)
        .join(&version.to_string());

    let binary = if cache_dir.join("node_modules/.bin").exists() {
        // Cached: verify TTL (24h default)
        if cache_valid(&cache_dir) {
            find_binary_in_node_modules(&cache_dir, &name)
        } else {
            // Stale: re-fetch
            fetch_and_cache(&name, &version, &cache_dir).await?
        }
    } else {
        // Fetch + install to cache
        fetch_and_cache(&name, &version, &cache_dir).await?
    };

    // 3. Execute binary with args
    let mut cmd = Command::new(binary);
    cmd.args(args);
    let status = cmd.status()?;
    std::process::exit(status.code().unwrap_or(1));
}
```

## 3.9 — Workspaces / Monorepo Support

### Design
```rust
pub struct Workspace {
    pub root: PathBuf,
    pub packages: Vec<WorkspacePackage>,
}

pub struct WorkspacePackage {
    pub name: String,
    pub path: PathBuf,
    pub manifest: PackageJson,
    pub dependencies: BTreeMap<String, String>,
}

// Dependency graph features
impl Workspace {
    /// Build inter-workspace dependency graph
    pub fn dependency_graph(&self) -> petgraph::Graph<String, ()>;

    /// Topological sort for build order
    pub fn build_order(&self) -> Vec<&WorkspacePackage>;

    /// Detect circular dependencies
    pub fn detect_cycles(&self) -> Vec<Vec<String>>;

    /// Run script in all workspaces (parallel)
    pub async fn run_all(&self, script: &str, parallel: bool) -> Vec<RunResult>;
}
```

---

# ⚡ Fase 4: Performance & Runtime — Bun+ Parity

## 4.0 — Performance Architecture

### Runtime comparison targets
| Metric | Klyron Target | Bun (v1.1) | Node (v22) | Notes |
|---|---|---|---|---|
| Cold start (no cache) | < 10ms | ~8ms | ~80ms | Script execution |
| Warm start | < 2ms | ~2ms | ~30ms | With isolate pool |
| `install react` (cached) | < 50ms | ~45ms | ~800ms | 50 deps |
| `build` (simple TS) | < 100ms | ~80ms | ~2s | No minify |
| `test` (1 file) | < 50ms | ~40ms | ~500ms | Single test |
| Binary size (compressed) | < 30MB | ~35MB (Bun) | ~80MB (Node) | |
| Memory per eval | < 5MB | ~6MB | ~15MB | Fresh isolate |
| HTTP requests/sec | 50K+ | 60K | 30K | Simple server |

## 4.1 — Test Runner: `klyron test`

### Architecture
```rust
pub struct TestRunner {
    pub glob: Vec<String>,        // "**/*.test.{js,ts}"
    pub cwd: PathBuf,
    pub timeout: Duration,        // 30s default
    pub parallel: bool,           // default: true
    pub coverage: bool,
    pub watch: bool,
    pub framework: TestFramework, // jest, vitest, mocha, or built-in
}

impl TestRunner {
    pub async fn run(&self) -> TestResults {
        // 1. Discover test files
        let files = glob::glob_matching(&self.glob, &self.cwd);

        // 2. Execute tests (parallel or sequential)
        let mut results = Vec::new();
        let handles: Vec<_> = if self.parallel {
            files.into_iter().map(|f| self.run_test_file(f)).collect()
        } else {
            files.into_iter().map(|f| self.run_test_file(f)).collect()
        };

        // 3. Aggregate results
        TestResults {
            passed: results.iter().filter(|r| r.passed).count(),
            failed: results.iter().filter(|r| !r.passed).count(),
            total: results.len(),
            duration: start.elapsed(),
            results,
        }
    }
}
```

### Built-in assertion library
```javascript
// Globals injected by klyron test runner
// No need to import from 'assert' or 'expect'

describe('Math', () => {
  it('should add correctly', () => {
    expect(1 + 2).toBe(3);
    expect(1 + 2).not.toBe(4);
  });

  it('should handle async', async () => {
    const result = await fetch('https://api.example.com');
    expect(result.status).toBe(200);
  });
});

// TypeScript native
const fn = (a: number, b: number): number => a + b;

// Snapshot testing
it('renders correctly', () => {
  const output = renderComponent();
  expect(output).toMatchSnapshot();
});

// Mocking
const mock = vi.fn(() => 'mocked');
mock('test');
expect(mock).toHaveBeenCalledWith('test');
```

### Coverage integration
```rust
pub struct CoverageEngine {
    pub provider: CoverageProvider, // v8 coverage, istanbul, lcov
    pub threshold: CoverageThreshold,
    pub format: CoverageFormat,     // lcov, html, text, json
}

impl CoverageEngine {
    pub fn collect(&self, isolate: &mut Isolate) -> CoverageReport {
        // V8 built-in coverage: https://v8.dev/blog/javascript-code-coverage
        let coverage = isolate.take_coverage();
        // Convert to standard format
        CoverageReport {
            lines: coverage.line_hits,
            branches: coverage.branch_hits,
            functions: coverage.function_hits,
            total_lines: coverage.total_lines,
            coverage_pct: coverage.hit_count as f64 / coverage.total_count as f64,
        }
    }
}
```

## 4.2 — Bundler: `klyron build`

### Architecture
```rust
pub struct Bundler {
    pub entrypoints: Vec<PathBuf>,
    pub outdir: PathBuf,
    pub format: OutputFormat,      // esm, cjs, iife
    pub target: PlatformTarget,    // browser, node, edge, lambda
    pub minify: bool,
    pub sourcemap: SourcemapMode,  // inline, external, hidden
    pub splitting: bool,           // code splitting
    pub tree_shaking: bool,
    pub external: Vec<String>,     // packages to externalize
}

impl Bundler {
    pub async fn bundle(&self) -> BundlerResult {
        // 1. Parse all modules (parallel)
        let mut graph = ModuleGraph::new();
        for entry in &self.entrypoints {
            graph.parse_entrypoint(entry).await?;
        }

        // 2. Resolve imports
        graph.resolve_all()?;

        // 3. Tree shaking (if enabled)
        if self.tree_shaking {
            graph.eliminate_dead_code()?;
        }

        // 4. Code splitting (if enabled)
        let chunks = if self.splitting {
            graph.split_chunks(ChunkStrategy::default())?
        } else {
            graph.single_chunk()?
        };

        // 5. Generate output
        for (chunk, modules) in chunks {
            let code = self.generate_chunk(chunk, modules)?;

            // 6. Minify (if enabled)
            let code = if self.minify {
                minify_js(&code)?
            } else {
                code
            };

            // 7. Write output
            let out = self.outdir.join(chunk.filename());
            std::fs::write(&out, code)?;

            // 8. Write sourcemap
            if self.sourcemap != SourcemapMode::None {
                write_sourcemap(&out, &chunk, self.sourcemap)?;
            }
        }

        Ok(BundlerResult {
            chunks: chunks.len(),
            total_size: total_bytes,
            duration: start.elapsed(),
        })
    }
}
```

### Minification targets
| Technique | Size reduction | Time |
|---|---|---|
| Whitespace removal | ~20% | < 1ms |
| Identifier shortening | ~15% | < 5ms |
| Dead code elimination | ~10% | < 10ms |
| Property mangling | ~5% | < 5ms |
| **Total** | **~45% avg** | **< 20ms** |

## 4.3 — Dev Server with HMR

### WebSocket protocol
```json
// Client -> Server
{"type": "subscribe", "paths": ["/src/**"]}

// Server -> Client (file change)
{"type": "hmr:update", "module": "/src/App.tsx", "timestamp": 1234567890}

// Server -> Client (full reload)
{"type": "full-reload", "reason": "config changed"}

// Server -> Client (error overlay)
{"type": "error", "message": "SyntaxError: Unexpected token", "stack": "..."}

// Client -> Server (heartbeat)
{"type": "ping"}
```

### HMR update cycle
```rust
pub async fn handle_hmr_update(change: FileChange, state: &mut ServerState) {
    // 1. Invalidate module in cache
    state.module_cache.invalidate(&change.path);

    // 2. Re-compile changed module
    let result = state.bundler.compile_single(&change.path).await;

    match result {
        Ok(compiled) => {
            // 3. Check if HMR is safe (no CSS/global changes)
            if can_hmr(&compiled) {
                // 4. Push update to all connected clients
                state.ws_clients.broadcast(ServerMessage::HmrUpdate {
                    module: change.path.to_str().unwrap(),
                    code: compiled.code,
                    map: compiled.sourcemap,
                });
            } else {
                // 5. Full reload needed
                state.ws_clients.broadcast(ServerMessage::FullReload {
                    reason: "Unsafe HMR: global state change",
                });
            }
        }
        Err(e) => {
            // 6. Push error overlay
            state.ws_clients.broadcast(ServerMessage::CompileError {
                message: e.to_string(),
            });
        }
    }
}
```

### React Fast Refresh integration
```javascript
// Injected by dev server into entrypoint
// Enables React component HMR without losing state

if (import.meta.hot) {
  import.meta.hot.accept('./App', (newModule) => {
    // React Fast Refresh: preserve component state
    // Only re-render changed components
    refreshRuntime.performReactRefresh();
  });
}
```

## 4.4 — Built-in Utilities: Complete API Surface

### Cryptographic utilities
```rust
pub enum HashAlgorithm {
    Md5,
    Sha1,
    Sha256,
    Sha512,
    Blake3,
}

impl HashAlgorithm {
    pub fn hash(&self, data: &[u8]) -> Vec<u8> {
        match self {
            Self::Md5 => md5::compute(data).to_vec(),
            Self::Sha1 => sha1::Sha1::digest(data).to_vec(),
            Self::Sha256 => sha2::Sha256::digest(data).to_vec(),
            Self::Sha512 => sha2::Sha512::digest(data).to_vec(),
            Self::Blake3 => blake3::hash(data).as_bytes().to_vec(),
        }
    }
}
```

### UUID generation
```rust
pub enum UuidVersion {
    V4,  // Random
    V7,  // Time-ordered (MySQL 8.0+ compatible)
}

pub fn generate_uuid(version: UuidVersion) -> String {
    match version {
        UuidVersion::V4 => Uuid::new_v4().to_string(),
        UuidVersion::V7 => Uuid::new_v7().to_string(),
    }
}
```

### Password hashing
```rust
pub enum PasswordAlgorithm {
    Bcrypt { cost: u32 },       // default: 12
    Argon2 {
        m_cost: u64,            // 19456 (19MB)
        t_cost: u32,            // 2
        p_cost: u32,            // 1
    },
}

impl PasswordAlgorithm {
    pub fn hash(&self, password: &str) -> String {
        match self {
            Self::Bcrypt { cost } => {
                bcrypt::hash(password, *cost).unwrap()
            }
            Self::Argon2 { m_cost, t_cost, p_cost } => {
                let config = argon2::Config {
                    mem_cost: *m_cost,
                    time_cost: *t_cost,
                    lanes: *p_cost,
                    ..Default::default()
                };
                argon2::hash_encoded(password.as_bytes(), &salt, &config).unwrap()
            }
        }
    }

    pub fn verify(&self, password: &str, hash: &str) -> bool {
        match self {
            Self::Bcrypt { .. } => bcrypt::verify(password, hash).unwrap_or(false),
            Self::Argon2 { .. } => argon2::verify_encoded(hash, password.as_bytes()).unwrap_or(false),
        }
    }
}
```

### SQLite integration (bun:sqlite equivalent)
```javascript
// bun:sqlite compatible API
import { Database } from 'klyron:sqlite';

const db = new Database(':memory:');
db.run(`CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)`);
db.run(`INSERT INTO users VALUES (?, ?)`, [1, 'Alice']);

const results = db.query(`SELECT * FROM users`).all();
// [{ id: 1, name: 'Alice' }]

const first = db.query(`SELECT * FROM users WHERE id = ?`).get(1);
// { id: 1, name: 'Alice' }

// Type-safe prepared statements
const stmt = db.prepare(`INSERT INTO users VALUES (?, ?)`);
stmt.run(2, 'Bob');

// Backup / SQL dump
db.backup('backup.sqlite');
```

### Cron expression parser
```rust
pub fn describe_cron(expr: &str) -> String {
    // Parse "*/5 * * * *" -> "Every 5 minutes"
    // Parse "0 9 * * 1-5" -> "At 09:00 AM, Monday through Friday"
}

pub fn next_run(expr: &str, from: Option<DateTime<Utc>>) -> Vec<DateTime<Utc>> {
    // Return next N occurrences
}
```

## 4.5 — FFI / Native Addons (N-API)

### Architecture
```rust
pub struct NapiModule {
    pub path: PathBuf,           // path/to/addon.node
    pub symbols: HashMap<String, NapiSymbol>,
}

pub enum NapiSymbol {
    Function {
        args: Vec<NapiType>,
        ret: NapiType,
    },
    Class {
        methods: Vec<(String, Vec<NapiType>, NapiType)>,
        static_methods: Vec<(String, Vec<NapiType>, NapiType)>,
    },
}

pub enum NapiValue {
    Number(f64),
    String(String),
    Boolean(bool),
    Object(HashMap<String, NapiValue>),
    Array(Vec<NapiValue>),
    Buffer(Vec<u8>),
    Null,
    Undefined,
}
```

### WASM plugin system
```rust
// Plugin ABI (WASM component model)
// Interface: plugin.wit
interface plugin {
    /// Initialize plugin with config
    init(config: string) -> result<string, string>;

    /// Hook: pre-build
    pre-build(project: string) -> result<list<hook-action>, string>;

    /// Hook: post-build
    post-build(project: string, result: build-result) -> result<list<hook-action>, string>;

    /// Cleanup
    cleanup() -> result;
}
```

---

# 🐘 Fase 5: Composer / Laravel — Deep Native Integration

## 5.0 — Architecture

### Composer compatibility layer
```
┌─────────────────────────────────────────────────┐
│                 klyron composer                   │
│  Transparent pass-through for existing projects  │
│  `klyron composer require laravel/laravel`       │
├─────────────────────────────────────────────────┤
│                                                      
▼
┌─────────────────────────────────────────────────┐
│             Native Rust Implementation            │
├─────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────────────────┐  │
│  │  composer.json│  │   Dependency Resolver     │  │
│  │  Parser       │  │   (SAT/SMT solver,        │  │
│  │               │  │    Packagist integration)  │  │
│  └──────────────┘  └──────────────────────────┘  │
│  ┌──────────────┐  ┌──────────────────────────┐  │
│  │  composer.lock│  │   Autoloader Generator   │  │
│  │  Parser/Writer│  │   (PSR-4, Classmap)      │  │
│  └──────────────┘  └──────────────────────────┘  │
│  ┌──────────────────────────────────────────────┐│
│  │         PHP Engine (WASM + Subprocess)        ││
│  │         Blade Compiler, Artisan Runner        ││
│  └──────────────────────────────────────────────┘│
└─────────────────────────────────────────────────┘
```

### Decision: Native vs Pass-through
| Feature | Pass-through (current) | Native (target) |
|---|---|---|
| composer.lock parse | Calls `composer` CLI | Rust struct |
| Dependency resolution | Calls `composer update` | SAT solver in Rust |
| Autoloader generation | Via `composer dump-autoload` | Rust generator |
| PHP execution | System PHP subprocess | WASM PHP |
| Artisan commands | Subprocess | Managed subprocess |
| Performance | Slow (subprocess overhead) | Fast (Rust native) |
| Reliability | Depends on system PHP | Deterministic |

## 5.1 — composer.lock Native Parser (Rust)

### Data structures
```rust
#[derive(Serialize, Deserialize, Debug)]
pub struct ComposerLock {
    #[serde(rename = "_readme")]
    pub readme: Option<Vec<String>>,
    #[serde(rename = "content-hash")]
    pub content_hash: String,
    pub packages: Vec<ComposerPackage>,
    #[serde(rename = "packages-dev")]
    pub packages_dev: Vec<ComposerPackage>,
    #[serde(rename = "aliases")]
    pub aliases: Vec<ComposerAlias>,
    #[serde(rename = "minimum-stability")]
    pub minimum_stability: Option<String>,
    #[serde(rename = "stability-flags")]
    pub stability_flags: Option<BTreeMap<String, String>>,
    #[serde(rename = "prefer-stable")]
    pub prefer_stable: Option<bool>,
    #[serde(rename = "prefer-lowest")]
    pub prefer_lowest: Option<bool>,
    pub platform: Option<BTreeMap<String, String>>,
    pub platform_dev: Option<BTreeMap<String, String>>,
    #[serde(rename = "plugin-api-version")]
    pub plugin_api_version: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ComposerPackage {
    pub name: String,
    pub version: String,
    #[serde(rename = "version_normalized")]
    pub version_normalized: String,
    pub source: Option<ComposerSource>,
    pub dist: Option<ComposerDist>,
    pub require: Option<BTreeMap<String, String>>,
    #[serde(rename = "require-dev")]
    pub require_dev: Option<BTreeMap<String, String>>,
    pub conflict: Option<BTreeMap<String, String>>,
    pub replace: Option<BTreeMap<String, String>>,
    pub provide: Option<BTreeMap<String, String>>,
    pub suggest: Option<BTreeMap<String, String>>,
    #[serde(rename = "autoload")]
    pub autoload: Option<ComposerAutoload>,
    pub scripts: Option<BTreeMap<String, Vec<String>>>,
    pub license: Option<Vec<String>>,
    pub authors: Option<Vec<ComposerAuthor>>,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub time: Option<String>,
    #[serde(rename = "type")]
    pub pkg_type: Option<String>,
    pub notification_url: Option<String>,
    pub extra: Option<BTreeMap<String, serde_json::Value>>,
    pub bin: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ComposerAutoload {
    #[serde(rename = "psr-0")]
    pub psr0: Option<BTreeMap<String, String>>,
    #[serde(rename = "psr-4")]
    pub psr4: Option<BTreeMap<String, String>>,
    pub classmap: Option<Vec<String>>,
    pub files: Option<Vec<String>>,
    #[serde(rename = "exclude-from-classmap")]
    pub exclude_from_classmap: Option<Vec<String>>,
}
```

### Content-hash algorithm
```rust
pub fn compute_content_hash(manifest: &ComposerJson) -> String {
    // Mirrors Composer's hash algorithm
    // Sorts keys alphabetically, encodes as JSON without whitespace
    // Hashes with MD5 (for compatibility)
    let normalized = serde_json::to_string(&normalize_manifest(manifest)).unwrap();
    format!("{:x}", md5::compute(normalized.as_bytes()))
}
```

## 5.2 — composer.json Schema Validation

### Version constraint parser
```rust
pub enum ComposerConstraint {
    Exact(semver::Version),
    Range { min: semver::Version, max: semver::Version },
    Caret(semver::Version),       // ^1.2.3 -> >=1.2.3 <2.0.0
    Tilde(semver::Version),       // ~1.2.3 -> >=1.2.3 <1.3.0
    Or(Vec<ComposerConstraint>),
    And(Vec<ComposerConstraint>),
    Not(Box<ComposerConstraint>),
    Any,
    None,
}

impl ComposerConstraint {
    /// Parse composer version string
    /// ">=1.0 <2.0" -> Range
    /// "^1.2" -> Caret
    /// "~1.2.3" -> Tilde
    /// "1.2.*" -> Range(1.2.0, 1.3.0)
    /// ">=2.0 || >=1.0 <1.5" -> Or
    pub fn parse(input: &str) -> Result<Self>;

    /// Check if version satisfies constraint
    pub fn matches(&self, version: &semver::Version) -> bool;
}
```

### Platform packages
```rust
pub struct PlatformPackages {
    // PHP version
    pub php: Option<semver::Version>,
    // Extensions (ext-json, ext-pdo, ext-mbstring, etc.)
    pub extensions: HashMap<String, Option<semver::Version>>,
    // Libraries (lib-curl, lib-xml, etc.)
    pub libraries: HashMap<String, Option<semver::Version>>,
}

impl PlatformPackages {
    /// Detect installed PHP extensions by scanning php -m
    pub fn detect_system() -> Self;

    /// Check if platform requirements are satisfied
    pub fn satisfies(&self, require: &BTreeMap<String, String>) -> Vec<String> {
        let mut missing = Vec::new();
        for (pkg, constraint) in require {
            if pkg.starts_with("php") || pkg.starts_with("ext-") || pkg.starts_with("lib-") {
                let installed = self.get(pkg);
                let req = ComposerConstraint::parse(constraint).unwrap();
                if !installed.map_or(false, |v| req.matches(v)) {
                    missing.push(format!("{}: need {} but {} installed",
                        pkg, constraint, installed.map_or("missing".into(), |v| v.to_string())));
                }
            }
        }
        missing
    }
}
```

## 5.3 — PHP Dependency Resolver (SAT Solver)

### Algorithm
```rust
pub struct ComposerResolver {
    pub pool: PackagePool,          // All available versions
    pub requested: BTreeMap<String, String>,  // From composer.json require
    pub stability: Stability,       // Minimum stability
    pub prefer_stable: bool,
}

impl ComposerResolver {
    pub fn solve(&self) -> Result<Resolution> {
        // Use the same pubgrub algorithm as npm resolver
        // but with Composer-specific constraint handling:
        // 1. Platform packages (php, ext-*) are pre-solved
        // 2. Replace/provide/conflict relationships
        // 3. Stability flags per package
        // 4. Repository priorities (packagist > vcs > path)
        self.solve_with_pubgrub()
    }
}
```

### Repository integration
```rust
#[async_trait]
pub trait ComposerRepository {
    async fn search(&self, query: &str) -> Result<Vec<PackageInfo>>;
    async fn fetch_package(&self, name: &str) -> Result<PackageVersions>;
    async fn fetch_manifest(&self, name: &str, version: &semver::Version) -> Result<ComposerJson>;
}

pub struct PackagistRepo { /* https://repo.packagist.org/p2/*.json */ }
pub struct VcsRepo { /* Git, GitHub, GitLab */ }
pub struct PathRepo { /* Local directory */ }
pub struct ArtifactRepo { /* zip/tar files */ }
```

## 5.4 — Laravel Optimization Commands

### Full optimization pipeline
```rust
pub async fn optimize_laravel(project: &Path) -> Result<OptimizationReport> {
    let mut results = Vec::new();

    // 1. Config cache
    results.push(run_artisan(project, &["config:cache"]).await?);
    // Merges all config files into a single cached file (bootstrap/cache/config.php)
    // Reduces config loading from ~50 file reads to 1

    // 2. Route cache
    results.push(run_artisan(project, &["route:cache"]).await?);
    // Serializes all routes into a single file (bootstrap/cache/routes-v7.php)

    // 3. Event cache
    results.push(run_artisan(project, &["event:cache"]).await?);
    // Caches event/listener mappings

    // 4. View cache
    results.push(run_artisan(project, &["view:cache"]).await?);
    // Pre-compiles all Blade templates

    // 5. Icons cache (Laravel 11+)
    if version >= 11 {
        results.push(run_artisan(project, &["icons:cache"]).await?);
    }

    Ok(OptimizationReport {
        cached: results.iter().filter(|r| r.success).count(),
        failed: results.iter().filter(|r| !r.success).count(),
        results,
    })
}
```

### Performance benchmarks (Laravel)
| Operation | Before (no cache) | After (full cache) | Improvement |
|---|---|---|---|
| Route registration | ~500ms | ~5ms | 100x |
| Config loading | ~200ms | ~2ms | 100x |
| Blade render | ~50ms | ~5ms | 10x |
| Event dispatch | ~10ms | ~1ms | 10x |
| **Full page TTFB** | **~800ms** | **~50ms** | **16x** |

## 5.5 — Laravel Dev Tools Integration

### Sail commands
```rust
pub enum SailCommand {
    Up { services: Vec<String> },     // mysql, pgsql, redis, meilisearch, mailpit
    Down,
    Build,
    Artisan { args: Vec<String> },
    Composer { args: Vec<String> },
    Node { args: Vec<String> },
    Shell,
}

impl SailCommand {
    pub fn run(&self, project: &Path) -> Result<()> {
        let sail = project.join("vendor/bin/sail");
        if !sail.exists() {
            bail!("Laravel Sail not installed. Run: `composer require laravel/sail --dev`");
        }

        match self {
            Self::Up { services } => {
                let mut cmd = Command::new(&sail);
                cmd.arg("up").arg("-d");
                for svc in services { cmd.arg(svc); }
                cmd.status()?;
            }
            Self::Artisan { args } => {
                let status = Command::new(&sail)
                    .arg("artisan")
                    .args(args)
                    .status()?;
                if !status.success() {
                    bail!("Artisan command failed");
                }
            }
            // ...
        }
        Ok(())
    }
}
```

## 5.6 — PHP Autoloader Generation

### Output files
```
vendor/
├── autoload.php                # Entry point
└── composer/
    ├── autoload_real.php       # Autoloader class
    ├── autoload_namespaces.php # PSR-0 mappings
    ├── autoload_psr4.php       # PSR-4 mappings
    ├── autoload_classmap.php   # Class → file mapping
    ├── autoload_static.php     # Optimized static autoloader
    └── autoload_files.php      # Function includes
```

### Generator algorithm
```rust
pub struct AutoloadGenerator {
    pub psr4: BTreeMap<String, String>,
    pub psr0: BTreeMap<String, String>,
    pub classmap: Vec<PathBuf>,
    pub files: Vec<PathBuf>,
    pub authoritative: bool,
}

impl AutoloadGenerator {
    pub fn generate(&self, vendor_dir: &Path) -> Result<()> {
        // 1. Generate classmap (scan all PHP files for class declarations)
        let classmap = if self.authoritative || self.classmap.is_empty() {
            self.scan_classmap(vendor_dir)?
        } else {
            self.classmap.clone()
        };

        // 2. Write PSR-4 mapping
        self.write_file(vendor_dir.join("composer/autoload_psr4.php"),
            generate_psr4_php(&self.psr4))?;

        // 3. Write classmap
        self.write_file(vendor_dir.join("composer/autoload_classmap.php"),
            generate_classmap_php(&classmap))?;

        // 4. Write static autoloader (optimized)
        self.write_file(vendor_dir.join("composer/autoload_static.php"),
            generate_static_php(&self.psr4, &classmap))?;

        // 5. Write entry point
        self.write_file(vendor_dir.join("autoload.php"),
            generate_entry_php())?;

        Ok(())
    }
}
```

## 5.7 — PHP Version Management

### Commands
```
klyron php install 8.3    # Download PHP 8.3 to ~/.klyron/php/8.3
klyron php install 8.2    # Download PHP 8.2
klyron php use 8.3        # Set active PHP version
klyron php list           # List installed versions
klyron php default 8.3    # Set default version
klyron php ext list       # List installed extensions
klyron php ext install pdo_mysql  # Install extension
klyron php which          # Show path to active PHP binary
```

### Pre-built PHP binaries
- Static binaries (no system deps needed)
- Versions: 8.1, 8.2, 8.3, 8.4
- Platforms: linux-x64, linux-arm64, macos-x64, macos-arm64
- Common extensions pre-compiled: PDO, mbstring, curl, gd, xml, json, openssl, sodium, pcntl, posix, redis, imagick, swoole (Laravel Octane)

---

# 🔗 Fase 6: Adapters & Plugins — Universal Framework Integration

## 6.0 — Adapter Architecture

### Common adapter interface
```rust
#[async_trait]
pub trait FrameworkAdapter: Send + Sync {
    /// Framework name
    fn name(&self) -> &'static str;

    /// Detect if project uses this framework
    async fn detect(&self, project: &Path) -> bool;

    /// Start dev server
    async fn dev(&self, project: &Path, options: DevOptions) -> Result<()>;

    /// Build for production
    async fn build(&self, project: &Path, options: BuildOptions) -> Result<BuildResult>;

    /// Run tests
    async fn test(&self, project: &Path, options: TestOptions) -> Result<TestResult>;

    /// Lint code
    async fn lint(&self, project: &Path) -> Result<LintResult>;

    /// Format code
    async fn format(&self, project: &Path, write: bool) -> Result<FormatResult>;

    /// Generate scaffold
    async fn scaffold(&self, dest: &Path, vars: HashMap<String, String>) -> Result<()>;
}
```

### Implementation pattern for JS frameworks
```javascript
// adapters/next/src/index.js (existing pattern)
module.exports = {
  detect: (project) => {
    return fs.existsSync(path.join(project, 'next.config.js')) ||
           fs.existsSync(path.join(project, 'next.config.mjs')) ||
           fs.existsSync(path.join(project, 'next.config.ts'));
  },
  dev: (project) => {
    execSync('npx next dev', { cwd: project, stdio: 'inherit' });
  },
  build: (project) => {
    execSync('npx next build', { cwd: project, stdio: 'inherit' });
  },
  // ...
};
```

## 6.1 — JS/TS Framework Adapters (13 stubs → production)

### Prioritized implementation order
| Priority | Adapter | Reason |
|---|---|---|
| P0 | Remix | Growing ecosystem, Vite-based, similar to Next.js |
| P0 | SvelteKit | Popular, Vite-based |
| P0 | Angular | Enterprise demand (despite decline) |
| P1 | SolidStart | Rising star, Qwik competitor |
| P1 | Preact | Lightweight React alternative |
| P1 | Lit | Google's web component standard |
| P2 | Qwik | Resumability pioneer, growing |
| P2 | Solid.js | Standalone version (not Start) |
| P3 | Rails | Ruby on Rails (polyglot value) |
| P3 | tRPC | API layer, not full framework |
| P3 | Hapi/Koa | Node.js server frameworks |
| P4 | AdonisJS | Node.js Laravel-like framework |

### Implementation pattern for each
```javascript
// Example: Remix adapter
module.exports = {
  name: 'remix',
  detect: (project) => {
    return fs.existsSync(path.join(project, 'remix.config.js')) ||
           hasDep(project, '@remix-run/react');
  },
  dev: (project) => {
    // Remix v2+ uses Vite
    execSync('npx remix dev', { cwd: project, stdio: 'inherit' });
  },
  build: (project) => {
    execSync('npx remix build', { cwd: project, stdio: 'inherit' });
  },
  test: (project) => {
    // Prefers vitest
    execSync('npx vitest run', { cwd: project, stdio: 'inherit' });
  },
  scaffold: (project, vars) => {
    // Generate full Remix app with routing
    const template = path.join(__dirname, 'scaffold');
    copyTemplate(template, project, vars);
  }
};
```

## 6.2 — ORM Adapters (all done ✅)

All 8 ORM adapters are implemented with full scaffold code. No changes needed.

## 6.3-6.6 — Polyglot Framework Adapters

### PHP adapters
```rust
pub struct LaravelAdapter;   // exists, enhance
pub struct SymfonyAdapter;   // new
pub struct WordPressAdapter; // new
pub struct CodeIgniterAdapter; // new
pub struct CakePHPAdapter;   // new
pub struct Yii2Adapter;      // new

impl FrameworkAdapter for SymfonyAdapter {
    fn name(&self) -> &'static str { "symfony" }

    async fn detect(&self, project: &Path) -> bool {
        project.join("symfony.lock").exists()
            || project.join("config/services.yaml").exists()
            || has_composer_dep(project, "symfony/framework-bundle")
    }

    async fn dev(&self, project: &Path, _opts: DevOptions) -> Result<()> {
        // Symfony CLI or symfony server:start
        if which("symfony").is_ok() {
            run_cmd("symfony server:start", project)?;
        } else {
            run_cmd("php bin/console server:start", project)?;
        }
        Ok(())
    }
}
```

### Python adapters
```rust
pub struct DjangoAdapter;   // exists, enhance
pub struct FastApiAdapter;  // new
pub struct FlaskAdapter;    // new

impl FrameworkAdapter for FastApiAdapter {
    async fn detect(&self, project: &Path) -> bool {
        has_pip_dep(project, "fastapi")
    }

    async fn dev(&self, project: &Path, opts: DevOptions) -> Result<()> {
        let port = opts.port.unwrap_or(8000);
        run_cmd(&format!("uvicorn main:app --reload --port {}", port), project)?;
        Ok(())
    }
}
```

## 6.7 — Plugin System (WASM Component Model)

### Plugin lifecycle
```
Init ──> Ready
  │         │
  │    ┌────┴────┐
  │    │         │
  ▼    ▼         ▼
Pre-build   Pre-dev   Pre-deploy
  │          │          │
  ▼          ▼          ▼
Build       Dev        Deploy
  │          │          │
  ▼          ▼          ▼
Post-build  Post-dev  Post-deploy
```

### WASM plugin ABI
```wit
// klyron-plugin.wit
package klyron:plugin;

interface types {
    enum severity { info, warn, error }
    
    record diagnostic {
        severity: severity,
        message: string,
        file: option<string>,
        line: option<u32>,
        column: option<u32>,
    }

    record build-options {
        entrypoints: list<string>,
        outdir: string,
        minify: bool,
        sourcemap: bool,
        target: string,
    }

    record build-result {
        success: bool,
        duration-ms: u64,
        output-size: u64,
        diagnostics: list<diagnostic>,
    }
}

interface plugin {
    use types;

    /// Initialize plugin with JSON config
    init(config: string) -> result<_, string>;

    /// Hook called before build starts
    pre-build(options: build-options) -> result<list<diagnostic>, string>;

    /// Hook called after build completes
    post-build(result: build-result) -> result<list<diagnostic>, string>;

    /// Cleanup resources
    cleanup() -> result;
}
```

---

# 🏗️ Fase 7: Developer Experience — Production-Grade DX

## 7.0 — DX Architecture

### Key design principles
1. **Zero config**: `klyron dev` should just work
2. **Frictionless errors**: Every error has a suggested fix
3. **Progressive disclosure**: Simple for beginners, powerful for experts
4. **Consistent patterns`: `klyron <verb> <target>` everywhere
5. **Fast feedback loop**: Changes visible in <100ms

## 7.1 — Console / CLI Experience

### Error message design system
```
<-- Klyron Error ------------------------------------------->
│                                                           │
│  ✘ error[PM001]: Package not found                        │
│                                                           │
│  Could not resolve "nonexistent-package@^999.0.0"         │
│  from /home/user/project/package.json                     │
│                                                           │
│  ── Did you mean? ──                                      │
│     • "nonexistent-package" → "nonexistent-package"        │
│       (note: version ^999.0.0 doesn't exist)              │
│     • "existent-package" at ^1.0.0?                       │
│                                                           │
│  ── Troubleshooting ──                                     │
│     1. Check spelling: "nonexistent-package"              │
│     2. Search registry: `klyron search nonexistent-pkg`   │
│     3. Check for typos in package.json                    │
│     4. Ensure registry is accessible: npmjs.org           │
│                                                           │
│  Location: package.json:3:5                               │
│  See: https://klyron.dev/errors/PM001                     │
│                                                           │
<----------------------------------------------------------->
```

### Progress bars
```
klyron install react

  Downloading packages ━━━━━━━━━━━━━━━━━━━━━━ 100%  ████████████████████████
  Extracting          ━━━━━━━━━━━━━━━━━━━━━━  87%  ██████████████████░░░░░░
  Linking             ━━━━━━━━━━━━━━━━━━━━━━  45%  █████████░░░░░░░░░░░░░░░

  ✓ 47 packages installed in 1.2s (42 from cache)
```

### Auto-completion
```shell
# Shell completions for bash/zsh/fish/powershell
$ klyron <TAB>
create    — Bootstrap a new project
dev       — Start dev server
build     — Build for production
test      — Run tests
install   — Install packages
add       — Add a package
run       — Run a script
deploy    — Deploy to cloud
db        — Database commands
...

$ klyron create <TAB>
react     — React + Vite
next      — Next.js
astro     — Astro
vue       — Vue + Vite
nuxt      — Nuxt
sveltekit  — SvelteKit
laravel-react  — Laravel + React (Inertia)
laravel-vue    — Laravel + Vue (Inertia)
...
```

## 7.2 — Debugger (Chrome DevTools Protocol)

### Protocol support
```rust
pub struct Debugger {
    pub inspector: V8Inspector,        // V8 inspector protocol
    pub clients: Vec<DebugClient>,      // Connected DevTools
    pub breakpoints: Vec<Breakpoint>,
    pub watch_expressions: Vec<String>,
}

pub struct Breakpoint {
    pub id: u32,
    pub url: String,
    pub line: u32,
    pub column: Option<u32>,
    pub condition: Option<String>,      // Conditional breakpoint
    pub hit_count: u64,
    pub enabled: bool,
}

// Chrome DevTools Protocol methods
impl Debugger {
    // Enable debugging
    pub async fn enable(&mut self) -> Result<()>;

    // Set breakpoint
    pub fn set_breakpoint(&mut self, url: &str, line: u32) -> BreakpointId;

    // Step commands
    pub fn step_over(&mut self);
    pub fn step_into(&mut self);
    pub fn step_out(&mut self);
    pub fn continue_(&mut self);

    // Evaluate in paused context
    pub async fn evaluate(&self, expr: &str, frame_id: &str) -> Result<serde_json::Value>;
}
```

## 7.3 — Language Server Protocol (LSP)

### Capabilities
```rust
pub struct KlyronLsp {
    pub documents: HashMap<Url, Document>,
    pub diagnostics: HashMap<Url, Vec<Diagnostic>>,
    pub completions: CompletionEngine,
    pub symbols: SymbolIndex,
}

impl KlyronLsp {
    // Initialize: send capabilities
    fn initialize(&self) -> InitializeResult {
        InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::Incremental,
                )),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![".".into(), "/".into()]),
                    ..Default::default()
                }),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                workspace_symbol_provider: Some(OneOf::Left(true)),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                rename_provider: Some(OneOf::Left(true)),
                signature_help_provider: Some(SignatureHelpOptions {
                    trigger_characters: Some(vec!["(".into(), ",".into()]),
                    ..Default::default()
                }),
                ..Default::default()
            },
        }
    }
}
```

## 7.4 — IDE Extensions

### VS Code extension features
```json
// klyron-vscode/package.json
{
  "contributes": {
    "commands": [
      { "command": "klyron.createProject", "title": "Klyron: Create Project" },
      { "command": "klyron.installDeps", "title": "Klyron: Install Dependencies" },
      { "command": "klyron.devServer", "title": "Klyron: Start Dev Server" },
      { "command": "klyron.runTests", "title": "Klyron: Run Tests" },
      { "command": "klyron.openDbStudio", "title": "Klyron: Open Database Studio" }
    ],
    "keybindings": [
      { "key": "ctrl+shift+k", "command": "klyron.devServer" },
      { "key": "ctrl+shift+t", "command": "klyron.runTests" }
    ]
  }
}
```

## 7.5 — Cloud Deployment

### Deployment strategy
```rust
pub struct Deployment {
    pub provider: DeployProvider, // Vercel, Cloudflare, Railway, Fly, Docker
    pub build: BuildResult,
    pub config: DeployConfig,
}

pub enum DeployProvider {
    Vercel {
        team: Option<String>,
        project: String,
        production: bool,
    },
    Cloudflare {
        account: String,
        project: String,
        routes: Vec<String>,
    },
    Railway {
        environment: String,
    },
    Fly {
        app: String,
        region: String,
    },
    Docker {
        registry: String,
        tag: String,
        dockerfile: PathBuf,
    },
}
```

## 7.6 — AI / LLM Integration

### Architecture
```rust
pub struct AiEngine {
    pub provider: LlmProvider,
    pub model: String,
    pub system_prompt: String,
}

pub enum LlmProvider {
    OpenAI { api_key: String, model: String },
    Anthropic { api_key: String, model: String },
    Google { api_key: String, model: String },
    Ollama { endpoint: String, model: String },
    Custom { endpoint: String, api_key: Option<String> },
}

impl AiEngine {
    /// Generate project from description
    pub async fn generate_project(&self, description: &str) -> Result<ScaffoldResult>;

    /// Review code changes
    pub async fn review_code(&self, diff: &str) -> Result<Vec<ReviewComment>>;

    /// Generate commit message
    pub async fn commit_message(&self, diff: &str) -> Result<String>;

    /// Generate documentation
    pub async fn generate_docs(&self, source: &str, language: &str) -> Result<String>;

    /// Suggest fix for compile error
    pub async fn suggest_fix(&self, error: &str, code: &str) -> Result<String>;
}
```

---

# 🛡️ Fase 8: Security Hardening

## 8.0 — Threat Model

### Trust boundaries
```
┌─────────────────────┐     ┌──────────────────────┐
│   User's Terminal    │     │   npm Registry        │
│   (untrusted input)  │────▶│   (semi-trusted)      │
└─────────────────────┘     └──────────────────────┘
         │                           │
         │                           │
         ▼                           ▼
┌─────────────────────────────────────────────────────┐
│                 Klyron CLI                            │
│  ┌──────────────────────────────────────────────┐   │
│  │        Sandbox Boundary                       │   │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐   │   │
│  │  │ JS Engine│  │PHP Engine│  │ WASM     │   │   │
│  │  │ (V8 iso) │  │(process) │  │ Plugins  │   │   │
│  │  │ limited  │  │ seccomp  │  │ linear   │   │   │
│  │  │ memory   │  │ RLIMIT   │  │ memory   │   │   │
│  │  └──────────┘  └──────────┘  └──────────┘   │   │
│  └──────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────┘
```

### Risk register
| Threat | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Malicious npm package (postinstall) | Medium | Critical | Script validation, sandbox, blocklist |
| Registry supply chain attack | Low | Critical | Signature verification, TUF, SRI |
| Engine sandbox escape | Low | Critical | seccomp, landlock, process isolation |
| Dependency confusion | Medium | High | Scope-based registry resolution |
| Path traversal in pack/unpack | Low | High | Canonicalize all paths |
| Credential leak | Medium | High | Encrypted storage, env scrub |
| DoS via resource exhaustion | Medium | Medium | Memory limits, CPU limits, timeouts |

---

# 📊 Fase 9: Quality Assurance

## 9.0 — Testing Pyramid

```
         ╱╲
        ╱  ╲          E2E Tests (10)
       ╱    ╲         Full CLI scenarios
      ╱──────╲
     ╱        ╲       Integration Tests (100)
    ╱          ╲      Multi-component, network isolated
   ╱────────────╲
  ╱              ╲    Unit Tests (1000+)
 ╱                ╲   Per-crate, fast, deterministic
╱──────────────────╲
╱   Property/Fuzz   ╲  Fuzz Tests (∞)
╱────────────────────╲ Random inputs, invariant checking
```

## 9.1 — Unit Test Targets

| Crate | Current | Target | Priority |
|---|---|---|---|
| klyron_core | 17 | 50 | High |
| klyron_engine | 40 | 100 | High |
| klyron_pm | 40 | 200 | High |
| klyron_plugin | 5 | 50 | Medium |
| klyron_http | 0 | 50 | Medium |
| klyron_loader | 10 | 50 | Medium |
| klyron_config | 5 | 30 | Low |
| klyron_cache | 5 | 30 | Low |
| Extensions (13) | ~75 | 200 | Medium |
| **Total** | **~170+** | **~1000+** | |

## 9.2 — Cross-Platform Test Matrix

| Platform | Rust target | Notes |
|---|---|---|
| Ubuntu 22.04 x64 | x86_64-unknown-linux-gnu | Primary |
| Ubuntu 24.04 x64 | x86_64-unknown-linux-gnu | Latest |
| Alpine 3.20 x64 | x86_64-unknown-linux-musl | musl compatibility |
| macOS 14 ARM | aarch64-apple-darwin | Apple Silicon |
| macOS 13 x64 | x86_64-apple-darwin | Intel Mac |
| Windows 11 x64 | x86_64-pc-windows-msvc | Primary Windows |
| Windows 11 ARM | aarch64-pc-windows-msvc | Surface/ARM |
| Docker multi-arch | Both | Final verification |

---

# 🌍 Fase 10: Ecosystem & Community

## 10.0 — Documentation Architecture

### Documentation tree
```
docs/
├── index.md                    # Welcome + quick start
├── getting-started.md          # Installation + first project
├── commands/                   # Per-command reference
│   ├── run.md
│   ├── dev.md
│   ├── build.md
│   ├── install.md
│   ├── test.md
│   └── ...
├── guides/                     # Task-oriented guides
│   ├── laravel-setup.md
│   ├── monorepo.md
│   ├── publishing-packages.md
│   ├── migration-from-npm.md
│   └── creating-plugins.md
├── api/                        # API reference
│   ├── rust/                   # Rustdoc
│   └── javascript/             # JS API docs
├── architecture.md             # System architecture
├── contributing.md             # Contributor guide
├── security.md                 # Security policy
└── changelog/                  # Release notes
    ├── v0.1.0.md
    └── ...
```

---

# 🎯 CLI Command Reference — Complete Surface

## Runtime

| Command | Status | Fase | Implementation |
|---|---|---|---|
| `klyron run app.js` | ⚠️ Partial | 2 | `EngineRuntime::run_file()` |
| `klyron run app.ts` | ⚠️ Partial | 2 | TypeScript transpile → run |
| `klyron run app.tsx` | ❌ | 2 | TSX transpile needed |
| `klyron run app.jsx` | ❌ | 2 | JSX transpile needed |
| `klyron repl` | ⚠️ Partial | 7 | Interactive REPL with history |
| `klyron eval "console.log('hello')"` | ⚠️ Partial | 2 | One-shot eval via V8 |
| `klyron shell` | ⚠️ Partial | 7 | System shell with Klyron env |

## Development

| Command | Status | Fase | Implementation |
|---|---|---|---|
| `klyron dev` | ⚠️ Partial | 4 | Auto-detect entrypoint + Vite-like server |
| `klyron dev src/index.ts` | ❌ | 4 | Custom entrypoint |
| `klyron dev --watch` | ❌ | 4 | File watcher + auto-restart |
| `klyron dev --hot` | ❌ | 4 | HMR via WebSocket |
| `klyron dev --host` | ❌ | 4 | Bind to 0.0.0.0 |
| `klyron dev --port 3000` | ❌ | 4 | Custom port |

## Build

| Command | Status | Fase | Implementation |
|---|---|---|---|
| `klyron build` | ⚠️ Partial | 4 | Auto-detect + bundle |
| `klyron build src/index.ts` | ❌ | 4 | Custom entrypoint |
| `klyron build --minify` | ❌ | 4 | Esbuild/speedy minify |
| `klyron build --sourcemap` | ❌ | 4 | Inline/external sourcemap |
| `klyron build --target browser` | ❌ | 4 | Browser-compatible output |
| `klyron build --target node` | ❌ | 4 | Node.js CJS/ESM output |
| `klyron build --target edge` | ❌ | 4 | Cloudflare Workers format |
| `klyron build --target lambda` | ❌ | 4 | AWS Lambda zip |

## Package Manager

| Command | Status | Fase | Implementation |
|---|---|---|---|
| `klyron install` | ⚠️ Partial | 3 | `install()` with lockfile |
| `klyron install react` | ⚠️ Partial | 3 | Single package install |
| `klyron install react react-dom` | ❌ | 3 | Multiple packages |
| `klyron add react` | ⚠️ Partial | 3 | Add + save to package.json |
| `klyron add vue` | ⚠️ Partial | 3 | Same as above |
| `klyron remove react` | ⚠️ Partial | 3 | Remove + save |
| `klyron uninstall react` | ❌ | 3 | Alias for remove |
| `klyron update` | ❌ | 3 | Update all deps within semver |
| `klyron upgrade` | ⚠️ Partial | 3 | Upgrade to latest |
| `klyron outdated` | ❌ | 3 | Show outdated packages |
| `klyron audit` | ⚠️ Partial | 3 | Vulnerability scan |
| `klyron doctor` | ⚠️ Partial | 1 | System health check |
| `klyron dedupe` | ❌ | 3 | Deduplicate deps |

## Package.json Scripts

| Command | Status | Fase | Implementation |
|---|---|---|---|
| `klyron start` | ❌ | 3 | Runs `npm start` script |
| `klyron test` | ⚠️ Partial | 4 | Runs `npm test` or built-in |
| `klyron lint` | ⚠️ Partial | 7 | Runs linter |
| `klyron format` | ⚠️ Partial | 7 | Runs formatter |
| `klyron run dev` | ⚠️ Partial | 3 | Runs `dev` script |
| `klyron run build` | ⚠️ Partial | 3 | Runs `build` script |
| `klyron run start` | ⚠️ Partial | 3 | Runs `start` script |

## Workspace / Monorepo

| Command | Status | Fase | Implementation |
|---|---|---|---|
| `klyron workspace init` | ❌ | 3 | Initialize workspace |
| `klyron workspace list` | ❌ | 3 | List workspace packages |
| `klyron workspace add frontend` | ❌ | 3 | Add workspace package |
| `klyron workspace add backend` | ❌ | 3 | Add workspace package |
| `klyron workspace run build` | ❌ | 3 | Run script in all packages |
| `klyron workspace run test` | ❌ | 3 | Run test in all packages |

## Framework Generator — Frontend

| Command | Status | Fase | Implementation |
|---|---|---|---|
| `klyron create react` | ✅ Ada | 6 | React + Vite scaffold |
| `klyron create vue` | ✅ Ada | 6 | Vue + Vite scaffold |
| `klyron create astro` | ✅ Ada | 6 | Astro scaffold |
| `klyron create next` | ✅ Ada | 6 | Next.js scaffold |
| `klyron create nuxt` | ✅ Ada | 6 | Nuxt scaffold |
| `klyron create sveltekit` | ❌ Stub | 6 | SvelteKit scaffold |
| `klyron create solid` | ❌ Stub | 6 | Solid scaffold |
| `klyron create qwik` | ❌ Stub | 6 | Qwik scaffold |
| `klyron create angular` | ❌ Stub | 6 | Angular scaffold |
| `klyron create remix` | ❌ Stub | 6 | Remix scaffold |

## Framework Generator — Backend

| Command | Status | Fase | Implementation |
|---|---|---|---|
| `klyron create express` | ✅ Ada | 6 | Express scaffold |
| `klyron create fastify` | ✅ Ada | 6 | Fastify scaffold |
| `klyron create nest` | ✅ Ada | 6 | NestJS scaffold |
| `klyron create hono` | ✅ Ada | 6 | Hono scaffold |
| `klyron create koa` | ❌ Stub | 6 | Koa scaffold |
| `klyron create hapi` | ❌ Stub | 6 | Hapi scaffold |
| `klyron create adonis` | ❌ Stub | 6 | AdonisJS scaffold |

## Laravel Integration

| Command | Status | Fase | Implementation |
|---|---|---|---|
| `klyron create laravel-react` | ✅ Ada | 5 | Laravel + React (Inertia) |
| `klyron create laravel-vue` | ✅ Ada | 5 | Laravel + Vue (Inertia) |
| `klyron create laravel-inertia-react` | ✅ Ada | 5 | Explicit Inertia React |
| `klyron create laravel-inertia-vue` | ✅ Ada | 5 | Explicit Inertia Vue |
| `klyron create laravel-livewire` | ✅ Ada | 5 | Laravel + Livewire |
| `klyron create laravel-next` | ✅ Ada | 5 | Laravel + Next.js BFF |
| `klyron create laravel-astro` | ✅ Ada | 5 | Laravel + Astro |
| `klyron create laravel-api` | ✅ Ada | 5 | Laravel API only |

## Database

| Command | Status | Fase | Implementation |
|---|---|---|---|
| `klyron db init` | ❌ | 7 | Initialize database |
| `klyron db generate` | ❌ | 7 | Generate schema/types |
| `klyron db migrate` | ❌ | 7 | Run migrations |
| `klyron db push` | ❌ | 7 | Push schema to DB |
| `klyron db pull` | ❌ | 7 | Pull schema from DB |
| `klyron db seed` | ❌ | 7 | Seed database |
| `klyron db reset` | ❌ | 7 | Reset database |
| `klyron db studio` | ❌ | 7 | Web-based DB browser |

## Prisma & Drizzle

| Command | Status | Fase | Implementation |
|---|---|---|---|
| `klyron prisma generate` | ⚠️ Partial | 6 | Prisma client generation |
| `klyron prisma migrate` | ⚠️ Partial | 6 | Prisma migration |
| `klyron prisma studio` | ⚠️ Partial | 6 | Prisma Studio |
| `klyron prisma db push` | ⚠️ Partial | 6 | Push Prisma schema |
| `klyron drizzle generate` | ⚠️ Partial | 6 | Drizzle schema generation |
| `klyron drizzle migrate` | ⚠️ Partial | 6 | Drizzle migration |
| `klyron drizzle studio` | ⚠️ Partial | 6 | Drizzle Studio |

## Testing

| Command | Status | Fase | Implementation |
|---|---|---|---|
| `klyron test` | ⚠️ Partial | 4 | Run all tests |
| `klyron test watch` | ❌ | 4 | Watch mode |
| `klyron test coverage` | ❌ | 4 | Coverage report |
| `klyron test ui` | ❌ | 4 | Interactive UI |
| `klyron test e2e` | ❌ | 4 | End-to-end tests |
| `klyron test unit` | ❌ | 4 | Unit tests only |
| `klyron test integration` | ❌ | 4 | Integration tests only |

## Linter & Formatter

| Command | Status | Fase | Implementation |
|---|---|---|---|
| `klyron lint` | ✅ Ada | 7 | Lint all |
| `klyron lint src` | ✅ Ada | 7 | Lint specific dir |
| `klyron lint fix` | ❌ | 7 | Auto-fix lints |
| `klyron format` | ✅ Ada | 7 | Check formatting |
| `klyron format src` | ✅ Ada | 7 | Check specific dir |
| `klyron format --write` | ❌ | 7 | Format + write |

## Type Checking

| Command | Status | Fase | Implementation |
|---|---|---|---|
| `klyron check` | ⚠️ Partial | 7 | Type check project |
| `klyron check types` | ❌ | 7 | Check types only |
| `klyron check project` | ❌ | 7 | Full project check |

## Plugin System

| Command | Status | Fase | Implementation |
|---|---|---|---|
| `klyron plugin install` | ❌ | 6 | Install plugin |
| `klyron plugin remove` | ❌ | 6 | Remove plugin |
| `klyron plugin list` | ❌ | 6 | List plugins |
| `klyron plugin update` | ❌ | 6 | Update plugins |
| `klyron plugin create` | ❌ | 6 | Create new plugin |

## Registry

| Command | Status | Fase | Implementation |
|---|---|---|---|
| `klyron publish` | ⚠️ Partial | 3 | Publish package |
| `klyron unpublish` | ❌ | 3 | Unpublish package |
| `klyron login` | ❌ | 3 | Registry login |
| `klyron logout` | ❌ | 3 | Registry logout |
| `klyron whoami` | ❌ | 3 | Current user |
| `klyron search react` | ❌ | 3 | Search packages |
| `klyron info react` | ❌ | 3 | Package info |

## Cache

| Command | Status | Fase | Implementation |
|---|---|---|---|
| `klyron cache clean` | ⚠️ Partial | 1 | Clear cache |
| `klyron cache prune` | ❌ | 1 | Remove stale cache |
| `klyron cache info` | ❌ | 1 | Cache statistics |

## Compatibility

| Command | Status | Fase | Implementation |
|---|---|---|---|
| `klyron compat check` | ❌ | 7 | Check compatibility |
| `klyron compat react` | ❌ | 7 | React compatibility |
| `klyron compat next` | ❌ | 7 | Next.js compatibility |
| `klyron compat astro` | ❌ | 7 | Astro compatibility |
| `klyron compat nest` | ❌ | 7 | NestJS compatibility |
| `klyron compat prisma` | ❌ | 7 | Prisma compatibility |

## Native Modules

| Command | Status | Fase | Implementation |
|---|---|---|---|
| `klyron napi build` | ❌ | 4 | Build N-API addon |
| `klyron napi generate` | ❌ | 4 | Generate N-API bindings |
| `klyron napi test` | ❌ | 4 | Test N-API addon |

## Docker

| Command | Status | Fase | Implementation |
|---|---|---|---|
| `klyron docker init` | ⚠️ Partial | 7 | Generate Dockerfile |
| `klyron docker build` | ⚠️ Partial | 7 | Build Docker image |
| `klyron docker run` | ⚠️ Partial | 7 | Run Docker container |

## Deployment

| Command | Status | Fase | Implementation |
|---|---|---|---|
| `klyron deploy vercel` | ❌ | 7 | Deploy to Vercel |
| `klyron deploy cloudflare` | ❌ | 7 | Deploy to Cloudflare |
| `klyron deploy railway` | ❌ | 7 | Deploy to Railway |
| `klyron deploy fly` | ❌ | 7 | Deploy to Fly.io |
| `klyron deploy docker` | ❌ | 7 | Deploy via Docker |

## Project Utilities

| Command | Status | Fase | Implementation |
|---|---|---|---|
| `klyron init` | ✅ Ada | 1 | Initialize project |
| `klyron upgrade` | ❌ | 1 | Self-upgrade |
| `klyron doctor` | ⚠️ Partial | 1 | System diagnostics |
| `klyron info` | ❌ | 1 | Project info |
| `klyron version` | ✅ Ada | 1 | Show version |
| `klyron telemetry` | ❌ | 7 | Telemetry config |
| `klyron config` | ❌ | 3 | Configuration |
| `klyron clean` | ❌ | 1 | Clean artifacts |

## AI / Enterprise

| Command | Status | Fase | Implementation |
|---|---|---|---|
| `klyron ai generate` | ❌ | 7 | AI code generation |
| `klyron ai optimize` | ❌ | 7 | AI optimization |
| `klyron ai review` | ❌ | 7 | AI code review |
| `klyron ai docs` | ❌ | 7 | AI documentation |
| `klyron ai test` | ❌ | 7 | AI test generation |
| `klyron ai migrate` | ❌ | 7 | AI migration assist |

---

# 📊 Implementation Summary

## Dependency Graph (Phase Order)
```
Fase 0 (Fix compile) ───┐
                         ▼
Fase 1 (Build system) ───▶───┐
                              ▼
              ┌───────────────┼───────────────┐
              ▼               ▼               ▼
        Fase 2 (Engine)  Fase 3 (npm)   Fase 5 (Composer)
              │               │               │
              ▼               ▼               ▼
              └───────┬───────┘               │
                      ▼                       │
                Fase 4 (Bun+) ◄──────────────┘
                      │
                      ▼
                Fase 6 (Adapters)
                      │
                      ▼
                Fase 7 (DX)
                      │
                      ▼
                Fase 8 (Security)
                      │
                      ▼
                Fase 9 (Quality)
                      │
                      ▼
                Fase 10 (Ecosystem)
```

## Resource Estimation
| Role | Count | Focus |
|---|---|---|
| Rust engineers | 2-3 | Core engine, CLI, package manager |
| Frontend engineers | 1-2 | Adapters, scaffolds, IDE extension |
| PHP/Laravel engineers | 1-2 | Composer, Laravel integration |
| Security engineer | 1 | Security audit, sandbox |
| QA/DevOps | 1 | CI/CD, testing, docs |
| **Total** | **6-9** | Full team |

### Timeline (parallel tracks)
```
Track 1: Core (Fase 0→1→2→4)
Track 2: PM (Fase 3→5)
Track 3: Adapters (Fase 6→7)
Track 4: Quality (Fase 8→9→10)

W1:  ████████░░░░░░░░░░░░  Track 1: Fix compile
     ████████░░░░░░░░░░░░  Track 2: Start Fase 3
W2:  ██████████████░░░░░░  Track 1: Engine native
     ██████████████░░░░░░  Track 2: Package manager
W3:  ████████████████████  Track 1: Bun parity
     ████████████████████  Track 2: Composer native
W4:  ████████████████████  Track 1: Test runner + bundler
     ████████████████████  Track 3: Adapters (P0-P1)
W5:  ████████████████████  Track 3: Adapters (P2-P4)
     ████████████████████  Track 4: Security audit
W6:  ████████████████████  Track 3: IDE extensions, LSP
     ████████████████████  Track 4: Documentation
W7:  ████████████████████  Final testing, release prep
```

## Milestones
**M1** (Week 1): `cargo check --workspace` ✅, CI green
**M2** (Week 2): `klyron install react` works, all 4 engines pass tests
**M3** (Week 3): `klyron build`, `klyron test`, `klyron publish` work
**M4** (Week 4): `klyron create laravel-react` generates working app, native composer.lock parse
**M5** (Week 5): 1000+ tests, security audit complete, all 27 adapters real
**M6** (Week 7): v1.0 release candidate — all 190+ commands functional

## Klyron DNA: The 7-in-1 Fusion

| Tool | What Klyron Takes | Commands |
|---|---|---|
| **Bun** | Runtime, speed, TS native | `klyron run`, `klyron test`, `klyron build` |
| **Deno** | Integrated tooling, security | `klyron lint`, `klyron format`, `klyron check` |
| **pnpm/npm** | Package mgmt, registry | `klyron install`, `klyron add`, `klyron publish` |
| **Vite** | Dev server, HMR, plugins | `klyron dev`, `klyron dev --hot` |
| **Prisma CLI** | Database workflow | `klyron db`, `klyron prisma` |
| **Laravel** | PHP framework deep integration | `klyron create laravel-*`, `klyron artisan` |
| **Cargo** | Workspace, monorepo, deps | `klyron workspace`, dependency graph |
