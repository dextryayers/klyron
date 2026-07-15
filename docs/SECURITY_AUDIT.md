# Klyron Security Audit: Unsafe Rust Blocks

Date: 2026-07-15  
Auditor: Klyron Security Team  
Scope: All `unsafe { }` blocks across `.rs` source files

## Summary

Total `unsafe` blocks found: 45+ across 35+ files.  
Categories:
- FFI (C interop, libc, dlopen/dlsym): 35 blocks
- Raw pointer dereference: 6 blocks
- Environment variable manipulation: 4 blocks

## Per-Block Audit

### 1. `src/ext/ffi/src/lib.rs:38` — Dynamic FFI call
- **Why unsafe**: Calls `dlopen`/`dlsym`/`transmute` for dynamic library loading
- **Safety invariants**: Library path is validated; function pointer transmute assumes correct signature
- **Can it be replaced?** No — FFI requires unsafe by nature
- **SAFETY**: Library path comes from user-controlled input; caller must validate. Function signature must match.

### 2. `src/ext/ffi/src/lib.rs:67` — dlopen
- **Why unsafe**: FFI call to C library `dlopen`
- **Safety invariants**: Path must be null-terminated valid C string
- **Can it be replaced?** No
- **SAFETY**: `CString::new()` ensures null-termination; path is validated before call

### 3. `src/ext/ffi/src/lib.rs:90` — dlsym
- **Why unsafe**: FFI call to C library `dlsym`
- **Safety invariants**: Handle must be from valid `dlopen`; symbol name valid C string
- **Can it be replaced?** No
- **SAFETY**: Handle was just returned from successful dlopen; name is CString

### 4. `src/ext/ffi/src/lib.rs:96` — transmute
- **Why unsafe**: Transmute `*mut c_void` to function pointer
- **Safety invariants**: Target function must match the `extern "C" fn() -> i64` signature
- **Can it be replaced?** No — transmute is required for dynamic FFI
- **SAFETY**: The function pointer type must match the actual exported symbol's signature

### 5. `src/core/src/sandbox.rs:35` — RLIMIT set
- **Why unsafe**: Calls `libc::setrlimit`
- **Safety invariants**: `rlimit` struct must be properly initialized; all fields set
- **Can it be replaced?** No — requires libc FFI
- **SAFETY**: All rlimit struct fields are initialized before call; OS provides safety

### 6. `src/core/src/sandbox.rs:159` — dlopen for macOS sandbox
- **Why unsafe**: FFI call to `libc::dlopen`
- **Safety invariants**: Path string must be null-terminated
- **Can it be replaced?** No
- **SAFETY**: String literal with explicit `\0` ensures null termination

### 7. `src/core/src/sandbox.rs:171` — dlclose
- **Why unsafe**: FFI call to libc
- **Safety invariants**: Handle must be from dlopen
- **Can it be replaced?** No
- **SAFETY**: Handle is from preceding dlopen call

### 8. `src/core/src/sandbox.rs:270` — chroot
- **Why unsafe**: Calls `libc::chroot` and `libc::chdir`
- **Safety invariants**: Path must be valid C string; process must have CAP_SYS_ADMIN
- **Can it be replaced?** No — OS-level sandboxing requires syscall FFI
- **SAFETY**: Path is from controlled temp dir; CString ensures null termination

### 9. `src/core/src/sandbox.rs:301` — mknod
- **Why unsafe**: Calls `libc::mknod`
- **Safety invariants**: Path valid C string, mode valid, dev_t valid
- **Can it be replaced?** No
- **SAFETY**: All arguments constructed from safe primitives; path is controlled temp dir

### 10. `src/core/src/sandbox.rs:318` — mount (bind mount)
- **Why unsafe**: Calls `libc::mount` twice
- **Safety invariants**: All pointers must be valid C strings or null; flags must be valid
- **Can it be replaced?** No
- **SAFETY**: Sources/dest are CStrings; null pointers for rest; flags are constants

### 11. `src/core/src/sandbox.rs:380` — mount (private)
- **Why unsafe**: Calls `libc::mount`
- **Safety invariants**: Same as #10
- **Can it be replaced?** No
- **SAFETY**: Null destination makes it a remount; flags are constants

### 12. `src/core/src/sandbox.rs:398` — unshare
- **Why unsafe**: Calls `libc::unshare`
- **Safety invariants**: Single integer argument; no memory safety concern
- **Can it be replaced?** No
- **SAFETY**: unshare takes a trivially safe integer flags argument

### 13. `src/core/src/sandbox.rs:505` — landlock ABI probe syscall
- **Why unsafe**: Raw `libc::syscall` with null pointer
- **Safety invariants**: Null pointers are valid for probe; return value check prevents use of invalid FD
- **Can it be replaced?** No
- **SAFETY**: Null pointers are valid for landlock ABI probe; FD is closed immediately

### 14. `src/core/src/sandbox.rs:562` — landlock_create_ruleset
- **Why unsafe**: Raw syscall with pointer to struct
- **Safety invariants**: Struct must be properly laid out (repr(C))
- **Can it be replaced?** No
- **SAFETY**: Struct is repr(C) with correct fields; size matches kernel expectation

### 15. `src/core/src/sandbox.rs:596` — open for landlock
- **Why unsafe**: `libc::open` with CString path
- **Safety invariants**: Path must be valid C string; O_CLOEXEC prevents fd leak
- **Can it be replaced?** No
- **SAFETY**: CString provides null termination; open is safe with valid flags

### 16. `src/core/src/sandbox.rs:613` — landlock_add_rule
- **Why unsafe**: Raw syscall
- **Safety invariants**: FD must be valid from create_ruleset; attr struct must be correct
- **Can it be replaced?** No
- **SAFETY**: FD is from successful landlock_create_ruleset; attr is repr(C)

### 17. `src/core/src/sandbox.rs:634` — landlock_restrict_self
- **Why unsafe**: Raw syscall
- **Safety invariants**: FD must be valid; after call, process is restricted
- **Can it be replaced?** No
- **SAFETY**: FD is valid; after call, no further unsafe operations possible

### 18. `src/core/src/sandbox.rs:676` — prctl(PR_SET_NO_NEW_PRIVS)
- **Why unsafe**: libc::prctl FFI
- **Safety invariants**: Integer-only arguments; no memory safety concern
- **Can it be replaced?** No
- **SAFETY**: prctl with integer-only arguments is safe; return value checked

### 19. `src/core/src/sandbox.rs:688` — seccomp syscall
- **Why unsafe**: Raw syscall with pointer to sock_fprog
- **Safety invariants**: BPF program must be valid; pointer must not dangle
- **Can it be replaced?** No
- **SAFETY**: sock_fprog is stack-allocated; BPF program is Vec stored in same scope

### 20. `crates/klyron_napi/src/lib.rs:133` — dlopen for napi
- **Why unsafe**: Loading dynamic library
- **Safety invariants**: Library path from validated source; version checks applied
- **Can it be replaced?** No
- **SAFETY**: Node.js ABI version validated before load; library must be signed by Node

### 21. `crates/klyron_napi/src/lib.rs:203` — dlsym for napi
- **Why unsafe**: Symbol lookup in dynamic library
- **Safety invariants**: Handle valid; C string for name
- **Can it be replaced?** No
- **SAFETY**: Handle from dlopen; CString for name

### 22. `crates/klyron_napi/src/lib.rs:226` — transmute for napi functions
- **Why unsafe**: Transmuting function pointer types
- **Safety invariants**: Function must match N-API calling convention
- **Can it be replaced?** No
- **SAFETY**: Signature matches N-API specification for napi_callback_info

### 23. `crates/klyron_napi/src/lib.rs:232` — calling napi function pointer
- **Why unsafe**: Calling through function pointer
- **Safety invariants**: Env, args must be valid
- **Can it be replaced?** No
- **SAFETY**: Env from V8 isolate; args array from Rust Vec

### 24-45. Bindings FFI files (22 files)
All in `crates/*/bindings/rust/ffi.rs`:
- **Pattern**: `CStr::from_ptr` for input strings; `CString::from_raw` for freeing
- **Why unsafe**: FFI boundary — pointers from C code
- **Safety invariants**: Pointers must be valid, non-null, properly aligned
- **Can it be replaced?** No — FFI requires unsafe
- **SAFETY** (each): Pointer validated for null before use; lifetime bounded by function scope

### 46. `crates/klyron_fs/src/lib.rs:497` — memory map
- **Why unsafe**: `Mmap::map` (from memmap2 crate, inherently unsafe)
- **Safety invariants**: File must not be mutated during mapping
- **Can it be replaced?** No — mmap is inherently unsafe
- **SAFETY**: File is opened read-only and held during map lifetime; no concurrent writes

### 47. `crates/klyron_config/src/lib.rs:429` — set_var
- **Why unsafe**: `std::env::set_var` (unsafe in edition 2024)
- **Safety invariants**: Only used in test code; single-threaded context
- **Can it be replaced?** Yes, by passing config explicitly instead of env var
- **SAFETY**: Used only in test with single thread; no concurrent env access

### 48. `crates/klyron_config/src/lib.rs:434` — remove_var
- **Why unsafe**: `std::env::remove_var`
- **Safety invariants**: Same as #47
- **Can it be replaced?** Yes
- **SAFETY**: Test-only; single-threaded

### 49. `crates/klyron_cli/src/commands/helpers.rs:190` — set_var
- **Why unsafe**: `std::env::set_var`
- **Safety invariants**: Called before any threading starts
- **Can it be replaced?** Yes, but ergonomic tradeoff
- **SAFETY**: Called during CLI initialization before worker threads

## Recommendations

1. Add `// SAFETY:` comment to every unsafe block with invariant explanation
2. Replace env var manipulation with config structs where feasible
3. Add runtime validation for FFI function pointer casts
4. Consider `#[deny(unsafe_code)]` on crates that don't need unsafe
