# Klyron — Universal Polyglot Runtime

**Version:** 0.1.0
**Edition:** 2024
**License:** MIT
**Architecture:** Rust workspace + Native Language Engines (C, C++, TypeScript, PHP)

---

## 1. Arsitektur Sistem

```
┌─────────────────────────────────────────────────────┐
│                   Klyron CLI                         │
│  src/cli/src/main.rs                                 │
│  ┌──────────────────────────────────────────────┐   │
│  │              Rust Bridge Layer                 │   │
│  │  src/cli/src/engines/                         │   │
│  │  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐        │   │
│  │  │ c.rs │ │cpp.rs│ │ts.rs │ │php.rs│        │   │
│  │  └──┬───┘ └──┬───┘ └──┬───┘ └──┬───┘        │   │
│  └─────┼────────┼────────┼────────┼──────────────┘   │
│        │        │        │        │                   │
│  ┌─────▼────────▼────────▼────────▼──────────────┐   │
│  │           Native Engine Binaries                │   │
│  │  src/cli/engines/                              │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌───┐ │   │
│  │  │engine.c  │ │engine.cpp│ │engine.ts │ │php│ │   │
│  │  │ (C)      │ │ (C++)    │ │ (Node)   │ │   │ │   │
│  │  └──────────┘ └──────────┘ └──────────┘ └───┘ │   │
│  └──────────────────────────────────────────────────┘   │
│                                                         │
│  ┌──────────────────────────────────────────────────┐   │
│  │           Klyron Core (deno_core + V8)            │   │
│  │  src/core/                                        │   │
│  │  Runtime · ModuleLoader · Transpiler · Sandbox    │   │
│  │  Permissions · Audit                             │   │
│  └──────────────────────────────────────────────────┘   │
│                                                         │
│  ┌──────────────────────────────────────────────────┐   │
│  │           Extension Crates (src/ext/)             │   │
│  │  console crypto ffi fs html http klyron net      │   │
│  │  node timers web ws                              │   │
│  └──────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

**Komunikasi Protocol (JSON-Line via stdin/stdout):**
```
Input:  {"action":"exec","code":"...","args":"...","filename":"...","project":"..."}
Output: {"stdout":"...","stderr":"...","exit_code":0,"result":"..."}
Actions: exec, file, eval, compile, ping, check
         artisan, composer, blade, artisan:serve, artisan:make, artisan:migrate
```

---

## 2. Engine Modules

### 2.1 C Engine (`src/cli/engines/c/engine.c`)

**Bahasa:** C23 (GNU C)
**Compiler:** `cc` (gcc/clang)
**File:** 232 lines

| Fitur | Status | Keterangan |
|-------|--------|------------|
| JSON-Line protocol stdin/stdout | ✅ | `json_get_string()` parser, `json_output()` writer |
| Compile + run (`exec`) | ✅ | Temp dir `/tmp/klyron_c_XXXXXX`, `mkdtemp`, cleanup |
| Compile only (`compile`) | ✅ | Skip run step |
| Expression eval (`eval`) | ✅ | Wrapped in `int main(){printf("%d",(expr));}` |
| Ping health check | ✅ | `{"action":"ping"}` → `pong` |
| Signal handling | ✅ | `SIGPIPE` ignored |
| Non-blocking I/O | ✅ | `select()` + `O_NONBLOCK` pada pipe |
| Timeout (30s) | ✅ | `alarm()` + `select()` timeout |
| Compiler flags | ✅ | `-Wall -Wextra -Werror -O2 -lm -pthread` |
| Memory safety | ✅ | `fmemopen` bounded, `free()` cleanup |
| Unicode JSON escape | ✅ | `\uXXXX` untuk < 0x20 |
| FD cleanup | ✅ | `fcntl(_, F_SETFD, FD_CLOEXEC)` on all FDs |

**Arsitektur internal:**
```
main()
 └── loop: fgets(stdin) → json_get_string(action, code, args)
      └── handle_action()
           ├── "exec/run"  → compile_and_run(source, args, 0)
           ├── "compile"   → compile_and_run(source, args, 1)
           ├── "eval"      → wrap + compile_and_run
           ├── "ping"/""   → json_output("pong")
           └── default     → json_output("Unknown action")
```

### 2.2 C++ Engine (`src/cli/engines/cpp/engine.cpp`)

**Bahasa:** C++20
**Compiler:** Auto-detect: `g++` → `clang++` → `c++`
**File:** 247 lines

| Fitur | Status | Keterangan |
|-------|--------|------------|
| JSON-Line protocol | ✅ | `std::string` based, modern C++ |
| Compiler auto-detect | ✅ | `detect_compiler()` function |
| Compile + run | ✅ | Temp dir, source write, compile, execute |
| Expression eval | ✅ | `std::cout << (expr)` wrapped |
| STL support | ✅ | `iostream`, `string`, `vector`, `algorithm` |
| Exceptions support | ✅ | Compiler flags include exception handling |
| Templates support | ✅ | C++20 standard |
| Non-blocking I/O | ✅ | `select()` with non-blocking pipes |
| Timeout (compile 120s, run 30s) | ✅ | `alarm()` |
| RAII | ✅ | `std::string` memory management |
| Cleanup | ✅ | `unlink()` + `rmdir()` on temp files |
| `-Werror` strict mode | ✅ | Compilation fails on warnings |

**Compiler detection logic:**
```
detect_compiler()
 → system("g++ --version") → return "g++"
 → system("clang++ --version") → return "clang++"
 → system("c++ --version") → return "c++"
 → default: "g++"
```

### 2.3 TypeScript Engine (`src/cli/engines/ts/engine.ts`)

**Runtime:** Node.js
**File:** 211 lines

| Fitur | Status | Keterangan |
|-------|--------|------------|
| JSON-Line protocol | ✅ | `readLine()` async, `writeOutput()` |
| Transpile + exec | ✅ | `transpileTypeScript()` → `new Function()` |
| File execution | ✅ | `execFile()` via `fs.readFileSync` |
| Expression eval | ✅ | `stripTypeAnnotations` → `eval()` |
| Type check (stub) | ✅ | Reports "requires tsc" |
| Deno fallback | ✅ | `Deno.transpileOnly()` |
| esbuild fallback | ✅ | `require("esbuild").transformSync()` |
| Regex type stripper | ✅ | 16 patterns komprehensif |
| Diagnostics extraction | ✅ | `extractDiagnostics()` file:line:col parser |
| JSX/TSX support | ✅ | esbuild loader auto-detect |
| Uncaught handler | ✅ | `uncaughtException` + `unhandledRejection` |
| Source maps | 🚧 | Planned |

**TypeScript stripping patterns (16 regex):**
```
1. : type annotations        → : number | string | boolean | void | null | undefined | any | never | unknown | bigint | symbol | object
2. : complex types           → : UserType<T> | OtherType
3. as Type casts             → as const | as Type
4. satisfies                 → satisfies Type
5. interface { }             → interface Foo extends Bar { }
6. type alias                → type Foo = ...
7. enum → const object       → enum Foo { A, B = 1 }
8. declare                   → declare const
9. access modifiers          → public | private | protected | readonly | abstract
10. definite assignment      → x!: number
11. optional property        → x?: type
12. decorators               → @decorator
13. import type              → import type { ... }
14. export type              → export type
15. export interface         → export interface
16. as const                 → as const
```

### 2.4 PHP Engine (`src/cli/engines/php/engine.php`)

**Runtime:** PHP 8.x
**File:** 243 lines

| Fitur | Status | Keterangan |
|-------|--------|------------|
| JSON-Line protocol | ✅ | `json_decode` + `json_encode` |
| Code exec (`eval()`) | ✅ | Dengan `ob_start()` capture |
| File execution | ✅ | `php file.php` subprocess |
| Expression eval | ✅ | `eval("return $expr;")` |
| Artisan commands | ✅ | `php artisan <args>` |
| Artisan serve | ✅ | `php artisan serve` |
| Artisan make:* | ✅ | `php artisan make:<type>` |
| Artisan migrate | ✅ | `php artisan migrate --force` |
| Composer commands | ✅ | `composer --working-dir=` |
| Blade rendering | ✅ | Laravel `View::make()` + raw PHP fallback |
| Timeout (30s) | ✅ | `proc_get_status` + `proc_terminate(SIGKILL)` |
| Non-blocking I/O | ✅ | `stream_select()` on pipes |
| Output truncation | ✅ | MAX_OUTPUT = 131072 |
| Error handling | ✅ | `match` expression, `try/catch` |
| `JSON_THROW_ON_ERROR` | ✅ | Strict JSON parsing |

**Laravel integration:**
```
artisan:make    → php artisan make:controller|model|migration|...
artisan:migrate → php artisan migrate --force
artisan:serve   → php artisan serve --host= --port=
composer        → composer install|require|update|...
blade           → View::make('view.name', $data)->render()
```

---

## 3. Rust Bridge Layer (`src/cli/src/engines/`)

### 3.1 Core Protocol (`mod.rs`)

**File:** 100 lines

| Component | Keterangan |
|-----------|------------|
| `EngineInput` | `action`, `code`, `args`, `filename`, `project` — semua `Option` |
| `EngineOutput` | `stdout`, `stderr`, `exit_code`, `result` — `#[serde(default)]` |
| `EngineProcess` | `spawn()` + `communicate()` — stdin/stdout JSON pipe |
| `Drop` impl | `flush()` + `kill()` + `wait()` otomatis |
| `find_engine_path()` | `env!("OUT_DIR")` + binary name |

### 3.2 Module C (`c.rs`)

| Method | Action |
|--------|--------|
| `CEngine::new()` | Spawn compiled `klyron-engine-c` |
| `exec(code, args)` | `"exec"` — compile + run |
| `eval_expr(expr)` | `"eval"` — evaluate expression |
| `compile(code)` | `"compile"` — compile only |
| `ping()` | `"ping"` — health check |

### 3.3 Module C++ (`cpp.rs`)

| Method | Action |
|--------|--------|
| `CppEngine::new()` | Spawn compiled `klyron-engine-cpp` |
| `exec(code, args)` | `"exec"` — compile + run |
| `eval_expr(expr)` | `"eval"` — evaluate expression |
| `compile(code)` | `"compile"` — compile only |
| `ping()` | `"ping"` — health check |

### 3.4 Module TypeScript (`ts.rs`)

| Method | Action |
|--------|--------|
| `TsEngine::new()` | Spawn `node engines/ts/engine.ts` |
| `exec(code)` | `"exec"` — transpile + run |
| `eval_expr(expr)` | `"eval"` — evaluate expression |
| `run_file(path)` | `"file"` — execute file |
| `typecheck(code)` | `"check"` — type check |
| `ping()` | `"ping"` — health check |

### 3.5 Module PHP (`php.rs`)

| Method | Action |
|--------|--------|
| `PhpEngine::new()` | Spawn `php engines/php/engine.php` |
| `exec(code)` | `"exec"` — run PHP code |
| `eval_expr(expr)` | `"eval"` — evaluate expression |
| `run_file(path)` | `"file"` — execute file |
| `artisan(args, project)` | `"artisan"` — Laravel Artisan |
| `composer(args, project)` | `"composer"` — Composer |
| `blade(view, data, project)` | `"blade"` — render Blade |
| `artisan_serve(args, project)` | `"artisan:serve"` |
| `artisan_make(args, project)` | `"artisan:make"` |
| `artisan_migrate(project)` | `"artisan:migrate"` |
| `ping()` | `"ping"` — health check |

---

## 4. CLI Commands (`src/cli/src/main.rs`)

**File:** 443 lines

| Command | Description | Engine |
|---------|-------------|--------|
| `klyron eval <code>` | Evaluate JS/TS code | V8 (core) |
| `klyron run <file>` | Run JS/TS file | V8 (core) |
| `klyron repl` | Interactive REPL | V8 (core) |
| `klyron bundle <entry>` | Bundle dependencies | V8 (core) |
| `klyron serve` | Dev server | V8 (core) |
| `klyron cc <source>` | Compile + run C | C Engine |
| `klyron cxx <source>` | Compile + run C++ | C++ Engine |
| `klyron ts <source>` | Run TypeScript | TS Engine |
| `klyron php <source>` | Run PHP | PHP Engine |
| `klyron artisan <args...>` | Laravel Artisan | PHP Engine |
| `klyron composer <args...>` | Composer | PHP Engine |
| `klyron blade <view>` | Render Blade | PHP Engine |

---

## 5. Build System

### 5.1 Workspace (`Cargo.toml`)

```
Members: 14 crates
  - src/cli          (klyron)
  - src/core         (klyron-core)
  - src/ext/*        (klyron-ext-*)
```

### 5.2 Build Script (`src/cli/build.rs`)

```
build.rs
 ├── compile engines/c/engine.c
 │   → cc -o $OUT_DIR/klyron-engine-c -Wall -Wextra -O2 -lm
 └── compile engines/cpp/engine.cpp
     → g++/clang++ -std=c++20 -o $OUT_DIR/klyron-engine-cpp -O2 -lm -pthread -Wall
```

**Output binaries (auto-compiled at build time):**
```
$OUT_DIR/
 ├── klyron-engine-c      (C engine — 17KB)
 └── klyron-engine-cpp    (C++ engine — 53KB)
```

### 5.3 Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `deno_core` | 0.403 | V8 JavaScript runtime |
| `clap` | 4 | CLI argument parsing |
| `serde` / `serde_json` | 1 | JSON protocol (engines) |
| `tokio` | 1 | Async runtime |
| `axum` | 0.8 | HTTP server |
| `tower-http` | 0.6 | Static file serving |
| `tracing-subscriber` | 0.3 | Logging |
| `anyhow` | 1 | Error handling |
| `oxc` | 0.139 | TypeScript transpiler |

---

## 6. Core Runtime (`src/core/`)

| Module | File | Lines | Description |
|--------|------|-------|-------------|
| `runtime` | `runtime.rs` | 298 | `JsRuntime` wrapper, eval, execute_script, module loading |
| `permissions` | `permissions.rs` | 749 | PermissionSet, SandboxLevel, PolicyTemplate, audit log |
| `sandbox` | `sandbox.rs` | 853 | Linux sandbox (seccomp, namespaces, resource limits) |
| `module_loader` | `module_loader.rs` | 291 | Module resolution, caching |
| `transpiler` | `transpiler.rs` | 72 | OXC-based TS→JS transpilation with source maps |
| `lib` | `lib.rs` | 8 | Re-exports: Runtime, Permissions, PermissionSet, etc. |

---

## 7. Extension Crates (`src/ext/`)

| Extension | Crate | Lines | Description |
|-----------|-------|-------|-------------|
| console | `klyron-ext-console` | 1.7K | `console.log`, `console.error` ops |
| timers | `klyron-ext-timers` | 1.8K | `setTimeout`, `setInterval` |
| crypto | `klyron-ext-crypto` | 2.4K | Web Crypto API |
| fs | `klyron-ext-fs` | 3.9K | File system operations |
| net | `klyron-ext-net` | 3.3K | TCP/UDP networking |
| http | `klyron-ext-http` | 2.0K | HTTP client/server |
| html | `klyron-ext-html` | 1.4K | HTML rendering |
| node | `klyron-ext-node` | 28K | Node.js compatibility (15 polyfills) |
| klyron | `klyron-ext-klyron` | 860B | Klyron-specific APIs |
| ffi | `klyron-ext-ffi` | 900B | Foreign Function Interface |
| ws | `klyron-ext-ws` | 13K | WebSocket support |
| web | `klyron-ext-web` | 1.9K | Web API (fetch, URL, etc.) |

---

## 8. Roadmap

### ✅ Phase 0 — Struktur & Pembersihan
- [x] Hapus semua crate sampah (18 stub crates, 4 old Rust engine crates, empty dirs)
- [x] Pindahkan engines/ ke dalam src/cli/engines/
- [x] Pindahkan semua crate ke bawah src/
- [x] Update workspace Cargo.toml
- [x] Fix semua path relatif

### ✅ Phase 1 — C Engine
- [x] Native C engine (`src/cli/engines/c/engine.c`)
- [x] JSON-Line protocol stdin/stdout
- [x] Compile + run via fork/exec
- [x] Signal handling, timeout, non-blocking I/O
- [x] Rust bridge (`src/cli/src/engines/c.rs`)

### ✅ Phase 2 — C++ Engine
- [x] Native C++ engine (`src/cli/engines/cpp/engine.cpp`)
- [x] Auto compiler detection (g++ → clang++ → c++)
- [x] C++20, STL, templates, exceptions
- [x] Non-blocking I/O, timeout (compile 120s, run 30s)
- [x] Rust bridge (`src/cli/src/engines/cpp.rs`)

### ✅ Phase 3 — TypeScript Engine
- [x] Native TS engine (`src/cli/engines/ts/engine.ts`)
- [x] 16 regex pattern type stripper
- [x] Deno + esbuild fallback
- [x] Diagnostics extraction (file:line:col)
- [x] Error boundary (uncaughtException, unhandledRejection)
- [x] Rust bridge (`src/cli/src/engines/ts.rs`)

### ✅ Phase 4 — PHP Engine
- [x] Native PHP engine (`src/cli/engines/php/engine.php`)
- [x] Laravel integration: artisan, composer, blade
- [x] Non-blocking I/O with `stream_select()`
- [x] Timeout with `proc_terminate(SIGKILL)`
- [x] Output truncation
- [x] Rust bridge (`src/cli/src/engines/php.rs`)

### ✅ Phase 5 — Integrasi CLI
- [x] CLI subcommands: cc, cxx, ts, php, artisan, composer, blade
- [x] build.rs compile C/C++ engines
- [x] Bridge: EngineProcess, EngineInput, EngineOutput
- [x] 0 warnings, 34/34 tests passed

### 🔄 Phase 6 — Advanced Features (Planned)
- [ ] **Source maps** untuk TS engine (inline base64)
- [ ] **Full type checking** TS via tsc subprocess
- [ ] **Multi-file compilation** untuk C/C++ engines
- [ ] **Module caching** untuk PHP engine (Blade cache)
- [ ] **Watch mode** untuk cc/cxx/ts/php commands
- [ ] **Artisan REPL** mode untuk debugging Laravel
- [ ] **Memory limit** engine subprocess via cgroups
- [ ] **Cross-compilation** support untuk C/C++ engines
- [ ] **gccgo/rustc** sebagai backend alternatif C engine

### 🔮 Phase 7 — Language Expansion (Planned)
- [ ] **Python engine** (`src/cli/engines/py/engine.py`)
- [ ] **Ruby engine** (`src/cli/engines/rb/engine.rb`)
- [ ] **Go engine** (`src/cli/engines/go/engine.go`)
- [ ] **Rust engine** (`src/cli/engines/rs/engine.rs`)
- [ ] **Zig engine** (`src/cli/engines/zig/engine.zig`)

---

## 9. Technical Details

### JSON Protocol Specification
```
Input:  {"action":"<string>","code":"<string>","args":"<string>","filename":"<string>","project":"<string>"}
Output: {"stdout":"<string>","stderr":"<string>","exit_code":<int>,"result":"<string>"}

All fields optional except "action".
Engine must respond with exactly one JSON line per input line.
Engine must set exit_code = 0 on success, non-zero on failure.
Engine must flush stdout after each response.
Engine must ignore SIGPIPE.
```

### Engine Discovery
```
C Engine:    $OUT_DIR/klyron-engine-c          (compiled by build.rs)
C++ Engine:  $OUT_DIR/klyron-engine-cpp        (compiled by build.rs)
TS Engine:   node <CARGO_MANIFEST_DIR>/engines/ts/engine.ts
PHP Engine:  php <CARGO_MANIFEST_DIR>/engines/php/engine.php
```

### Testing
```bash
cargo test                    # 34 tests (runtime + permissions + console + timers)
cargo check                   # 0 warnings
echo '{"action":"ping"}' | ./target/debug/klyron-engine-c   # pong
```

---

## 10. Project Structure

```
klyronjs/
├── Cargo.toml                        # Workspace — 14 crate members
├── Cargo.lock                        # Dependency lock
├── rust-toolchain.toml               # Rust toolchain config
├── rustfmt.toml                      # Rust formatter
├── install.sh                        # Install script
├── .gitignore                        # Git ignore rules
├── .github/workflows/ci.yml          # CI pipeline
│
├── src/
│   ├── cli/                          # ── CLI & Engine Modules ──
│   │   ├── Cargo.toml                #    CLI dependencies
│   │   ├── build.rs                  #    Compile C/C++ engine binaries
│   │   ├── engines/                  #    Native engine source files
│   │   │   ├── c/engine.c            #      C engine (232 lines)
│   │   │   ├── cpp/engine.cpp        #      C++ engine (247 lines)
│   │   │   ├── ts/engine.ts          #      TypeScript engine (211 lines)
│   │   │   └── php/engine.php        #      PHP engine (243 lines)
│   │   └── src/
│   │       ├── main.rs               #    CLI entry point (443 lines)
│   │       └── engines/              #    Rust bridge modules
│   │           ├── mod.rs            #      EngineProcess, types (100 lines)
│   │           ├── c.rs              #      CEngine (42 lines)
│   │           ├── cpp.rs            #      CppEngine (42 lines)
│   │           ├── ts.rs             #      TsEngine (55 lines)
│   │           └── php.rs            #      PhpEngine (90 lines)
│   │
│   ├── core/                         # ── Klyron Core Runtime ──
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                #    Re-exports
│   │       ├── runtime.rs            #    JsRuntime wrapper (298 lines)
│   │       ├── permissions.rs        #    Permission system (749 lines)
│   │       ├── sandbox.rs            #    Linux sandbox (853 lines)
│   │       ├── module_loader.rs      #    Module loader (291 lines)
│   │       └── transpiler.rs         #    OXC transpiler (72 lines)
│   │
│   └── ext/                          # ── Extension Crates ──
│       ├── console/  crypto/  ffi/   #    Console, Crypto, FFI
│       ├── fs/       html/    http/  #    Filesystem, HTML, HTTP
│       ├── klyron/   net/     node/  #    Klyron API, Networking, Node.js compat
│       ├── timers/   web/     ws/    #    Timers, Web API, WebSocket
│       └── ...
│
└── target/                           # Build artifacts (gitignored)
    └── debug/build/klyron-*/out/
        ├── klyron-engine-c           #    Compiled C engine binary
        └── klyron-engine-cpp         #    Compiled C++ engine binary
```

---

## 11. Metrics

| Metric | Value |
|--------|-------|
| Total Rust crates | 14 |
| Total source files | 73 |
| Total Rust lines | 3,976 |
| C engine (C code) | 232 lines |
| C++ engine (C++ code) | 247 lines |
| TS engine (TypeScript) | 211 lines |
| PHP engine (PHP) | 243 lines |
| Core runtime | ~2,271 lines |
| Extension crates | ~56,000+ lines (incl. deps) |
| Tests | 37 (34 core + 2 console + 1 timers) |
| Warnings | 0 |
| Top-level folders | 2 (src/, .git/, .github/) |

---

## 12. Design Principles

1. **Native is king** — Setiap engine ditulis dalam bahasa aslinya (.c, .cpp, .ts, .php), bukan Rust. Rust hanya sebagai bridge/subprocess launcher.

2. **JSON-Line protocol** — Komunikasi universal via stdin/stdout dengan format JSON satu baris per request/response. Sederhana, debuggable, language-agnostic.

3. **Sub-process isolation** — Setiap engine berjalan di proses terpisah. Crash engine tidak mempengaruhi runtime utama. Resource limits via OS.

4. **Zero-copy where possible** — Engine binaries dikompilasi sekali di build time, reuse untuk semua eksekusi.

5. **Fail fast** — Validasi input di setiap layer. Error messages informatif dengan file:line:col.

6. **Defense in depth** — Timeout di semua tingkatan (alarm, select, Rust-side). Non-blocking I/O. Signal handling.

7. **Progressive enhancement** — TypeScript: Deno → esbuild → regex stripper. C++: g++ → clang++ → c++. PHP: Laravel → raw PHP.
