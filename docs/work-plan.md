# Klyron — Work Plan

> Berdasarkan audit `docs/audit.md`
> Prioritas: P0 (blocking) → P1 (user-visible) → P2 (quality) → P3 (polish)

---

## Phase A — Quick Wins (Housekeeping)

1. **Fix duplicate binary**
   - Hapus `[[bin]]` dari `crates/klyron_cli/Cargo.toml`
   - Hapus `"crates/klyron_cli"` dari `default-members` di root `Cargo.toml`
   - `src/cli/` jadi satu-satunya entry point binary `klyron`
   - Verifikasi: `cargo doc --workspace` berhasil tanpa collision

2. **Fix `pack_package()`**
   - Delegasi ke `klyron_pm::pack::pack()` atau append files ke tar builder
   - Verifikasi: test `pack_package` produce non-empty tarball

3. **Remove/relegate dead code**
   - Hapus `doctor.rs` (orphan — tidak di `mod.rs`)
   - Hapus `config.rs` atau merge fungsinya ke `utils.rs`
   - Hapus `completions.rs` (file kosong)

4. **Mark JSC & V8 as unavailable**
   - Di `klyron info`, jangan tampilkan sebagai "available"
   - Atau: nonaktifkan dari `all_engines()` / `default-members`

---

## Phase B — Engine Fix (Core Execution)

5. **Fix JSC Engine** (opsi: real or removed)
   - Opsi 1: Integrasi `javascriptcore-rs` crate → real JSC eval
   - Opsi 2: Hapus engines/jsc/ directory, hapus dari workspace
   - Opsi 3: Mark sebagai "not available" di detect, fallback ke Boa/QuickJS
   - + Fix test (`test_jsc_eval_syntax_error` harus assert `is_err()`)

6. **Fix V8 Engine** (opsi: real or removed)
   - Opsi 1: Integrasi `rusty_v8` crate → real V8 eval
   - Opsi 2: Hapus engines/v8/ directory, hapus dari workspace
   - Opsi 3: Mark sebagai "not available", fallback ke Boa/QuickJS
   - + Fix test (`test_v8_eval_syntax_error` harus assert `is_err()`)

7. **Engine detection integrity**
   - `detect_best_engine()` — skip engine yang no-op/stub
   - `benchmark_all_engines()` — jangan include engine yang no-op
   - `klyron info` — tampilkan status real: "stub" bukan "available"

---

## Phase C — Package Manager Fix (Core Feature)

8. **Connect `klyron install` to real download**
   - `InstallEngine::install()` harus:
     - Fetch package metadata dari npm registry (`GET /{package}`)
     - Resolve dependency tree via pubgrub (sudah ada)
     - Download tarball (`GET /{package}/-/{name}-{version}.tgz`)
     - Extract ke `node_modules/`
     - Generate/update `klyron.lock`
   - Ini perubahan terbesar — melibatkan `klyron_pm::install`, resolver, HTTP client

9. **Fix `klyron_pm` root stub functions**
   - `install_with_lockfile()` — real implementasi
   - `generate_lockfile()` — buat format KlyronLockfile beneran
   - `publish_package()` — delegasi ke `publish::publish()`
   - `add_dist_tag()` / `remove_dist_tag()` / `list_dist_tags()` — delegasi ke npm API
   - `why_package()` — baca lockfile, cari dependency path

---

## Phase D — Test Quality (Reliability)

10. **Fix root `cli_tests.rs`**
    - Ganti semua `match Err(_) => {}` dengan assertion meaningful
    - Test `klyron init` → cek file exist
    - Test `klyron info` → cek output format

11. **Fix engine tests** (Boa, QuickJS, JSC, V8)
    - Ganti `match Ok(_) => ..., Err(_) => {}` dengan assert real
    - Test `eval("1+2")` → `assert_eq!(result, "3")`

12. **Add tests for zero-test crates**
    - `klyron_intern` — intern/unintern string, cache hit/miss
    - `engines/common` — shared utilities

13. **Add integration test**
    - Execute real JS file via klyron runtime
    - Verify output, error handling, module loading

---

## Phase E — Polish & DX

14. **Doc comment sync**
    - Fix semua `rust,no_run` doc test jadi `rust,ignore`
    - Fix wrong `use` paths (contoh: `klyron_pm::install::install` tidak exist)

15. **CLI parameter completeness**
    - `klyron update` — tambah `--force` flag
    - `ConfigArgs` — pass `json` dan `file` ke `run_config()`

16. **Final verification**
    - `cargo check --workspace` — 0 errors
    - `cargo test --workspace` — all pass
    - `cargo doc --workspace` — no collision
    - `./target/debug/klyron --help` — all commands listed

---

## Timeline Estimate

| Phase | Estimated effort | Target |
|-------|-----------------|--------|
| A — Quick Wins | 1-2 jam | Hari 1 |
| B — Engine Fix | 4-8 jam (opsi hapus) / 2-5 hari (opsi integrasi) | Hari 1-3 |
| C — PM Fix | 8-16 jam | Hari 3-5 |
| D — Test Quality | 4-8 jam | Hari 5-7 |
| E — Polish | 2-4 jam | Hari 7-8 |

---

## Recommended Order

Mulai dari Phase A (quick wins — langsung keliatan hasilnya).
Lanjut Phase B (engine — core functionality).
Baru Phase C (PM — biggest impact).
Phase D & E paralel.

Mau mulai dari Phase A?
