#![cfg(feature = "native")]

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_double, c_int, c_uint, c_ulong, c_void, c_uchar};

use crate::ffi_types::*;

/* Opaque handle types matching C typedefs */
#[repr(C)] pub struct V8IsolateHandle  { _private: [u8; 0] }
#[repr(C)] pub struct V8ContextHandle  { _private: [u8; 0] }
#[repr(C)] pub struct V8ValueHandle    { _private: [u8; 0] }
#[repr(C)] pub struct V8ScriptHandle   { _private: [u8; 0] }
#[repr(C)] pub struct V8ModuleHandle   { _private: [u8; 0] }
#[repr(C)] pub struct V8PromiseHandle  { _private: [u8; 0] }
#[repr(C)] pub struct V8SnapshotHandle { _private: [u8; 0] }

unsafe extern "C" {
    /* Platform lifecycle */
    fn klyron_v8_init(config: *const V8Config);
    fn klyron_v8_shutdown();
    fn klyron_v8_is_initialized() -> bool;

    /* Isolate */
    fn klyron_v8_isolate_new() -> *mut V8IsolateHandle;
    fn klyron_v8_isolate_dispose(isolate: *mut V8IsolateHandle);
    fn klyron_v8_isolate_enter(isolate: *mut V8IsolateHandle);
    fn klyron_v8_isolate_exit(isolate: *mut V8IsolateHandle);

    /* Context */
    fn klyron_v8_context_new(isolate: *mut V8IsolateHandle) -> *mut V8ContextHandle;
    fn klyron_v8_context_dispose(context: *mut V8ContextHandle);
    fn klyron_v8_context_enter(context: *mut V8ContextHandle);
    fn klyron_v8_context_exit(context: *mut V8ContextHandle);
    fn klyron_v8_context_new_from_snapshot(
        isolate: *mut V8IsolateHandle,
        snapshot: *mut V8SnapshotHandle,
    ) -> *mut V8ContextHandle;

    /* Script */
    fn klyron_v8_compile(
        context: *mut V8ContextHandle,
        source: *const c_char,
        filename: *const c_char,
    ) -> *mut V8ScriptHandle;
    fn klyron_v8_run(
        context: *mut V8ContextHandle,
        script: *mut V8ScriptHandle,
    ) -> V8StringResult;
    fn klyron_v8_eval(
        context: *mut V8ContextHandle,
        source: *const c_char,
        filename: *const c_char,
    ) -> V8StringResult;
    fn klyron_v8_script_dispose(script: *mut V8ScriptHandle);

    /* JSON */
    fn klyron_v8_json_stringify(
        context: *mut V8ContextHandle,
        value: *mut V8ValueHandle,
    ) -> V8StringResult;
    fn klyron_v8_json_parse(
        context: *mut V8ContextHandle,
        json: *const c_char,
    ) -> *mut V8ValueHandle;

    /* Global */
    fn klyron_v8_set_global(
        context: *mut V8ContextHandle,
        name: *const c_char,
        value: *mut V8ValueHandle,
    ) -> V8Result;
    fn klyron_v8_get_global(
        context: *mut V8ContextHandle,
        name: *const c_char,
    ) -> *mut V8ValueHandle;

    /* Value creation */
    fn klyron_v8_value_new_string(context: *mut V8ContextHandle, str: *const c_char) -> *mut V8ValueHandle;
    fn klyron_v8_value_new_number(context: *mut V8ContextHandle, num: c_double) -> *mut V8ValueHandle;
    fn klyron_v8_value_new_bool(context: *mut V8ContextHandle, val: bool) -> *mut V8ValueHandle;
    fn klyron_v8_value_new_null(context: *mut V8ContextHandle) -> *mut V8ValueHandle;
    fn klyron_v8_value_new_undefined(context: *mut V8ContextHandle) -> *mut V8ValueHandle;
    fn klyron_v8_value_new_object(context: *mut V8ContextHandle) -> *mut V8ValueHandle;
    fn klyron_v8_value_new_array(context: *mut V8ContextHandle) -> *mut V8ValueHandle;

    /* Value inspection */
    fn klyron_v8_value_typeof(context: *mut V8ContextHandle, value: *mut V8ValueHandle) -> V8TypeResult;
    fn klyron_v8_value_to_string(context: *mut V8ContextHandle, value: *mut V8ValueHandle) -> V8StringResult;
    fn klyron_v8_value_to_number(context: *mut V8ContextHandle, value: *mut V8ValueHandle) -> c_double;
    fn klyron_v8_value_to_bool(context: *mut V8ContextHandle, value: *mut V8ValueHandle) -> bool;
    fn klyron_v8_value_is_array(context: *mut V8ContextHandle, value: *mut V8ValueHandle) -> bool;
    fn klyron_v8_value_dispose(value: *mut V8ValueHandle);

    /* Promise */
    fn klyron_v8_promise_new(context: *mut V8ContextHandle) -> *mut V8PromiseHandle;
    fn klyron_v8_promise_resolve(
        context: *mut V8ContextHandle,
        promise: *mut V8PromiseHandle,
        value: *mut V8ValueHandle,
    ) -> V8Result;
    fn klyron_v8_promise_reject(
        context: *mut V8ContextHandle,
        promise: *mut V8PromiseHandle,
        reason: *const c_char,
    ) -> V8Result;
    fn klyron_v8_promise_get_native(promise: *mut V8PromiseHandle) -> *mut V8ValueHandle;
    fn klyron_v8_promise_has_handler(context: *mut V8ContextHandle, promise: *mut V8PromiseHandle) -> bool;
    fn klyron_v8_promise_mark_as_handled(context: *mut V8ContextHandle, promise: *mut V8PromiseHandle) -> V8Result;
    fn klyron_v8_promise_get_state(context: *mut V8ContextHandle, promise: *mut V8PromiseHandle) -> c_uint;

    /* Microtasks */
    fn klyron_v8_microtasks_perform_check(context: *mut V8ContextHandle);

    /* Module */
    fn klyron_v8_module_compile(context: *mut V8ContextHandle, source: *const c_char, origin: *const c_char)
        -> *mut V8ModuleHandle;
    fn klyron_v8_module_instantiate(context: *mut V8ContextHandle, module: *mut V8ModuleHandle) -> V8Result;
    fn klyron_v8_module_evaluate(context: *mut V8ContextHandle, module: *mut V8ModuleHandle) -> V8StringResult;
    fn klyron_v8_module_get_identity(context: *mut V8ContextHandle, module: *mut V8ModuleHandle) -> c_int;
    fn klyron_v8_module_dispose(module: *mut V8ModuleHandle);

    /* Heap & memory */
    fn klyron_v8_get_heap_stats(isolate: *mut V8IsolateHandle, stats: *mut V8HeapStats) -> V8Result;
    fn klyron_v8_low_memory_notification(isolate: *mut V8IsolateHandle);
    fn klyron_v8_idle_notification(isolate: *mut V8IsolateHandle, deadline: c_double);
    fn klyron_v8_set_memory_pressure(isolate: *mut V8IsolateHandle, pressure: c_uint);
    fn klyron_v8_request_gc(isolate: *mut V8IsolateHandle);
    fn klyron_v8_get_malloced_memory(isolate: *mut V8IsolateHandle) -> usize;
    fn klyron_v8_adjust_external_memory(isolate: *mut V8IsolateHandle, change: i64) -> usize;

    /* Snapshots */
    fn klyron_v8_snapshot_create(context: *mut V8ContextHandle) -> *mut V8SnapshotHandle;
    fn klyron_v8_snapshot_load(blob: *const c_char, length: usize) -> *mut V8SnapshotHandle;
    fn klyron_v8_snapshot_dispose(snapshot: *mut V8SnapshotHandle);

    /* Error */
    fn klyron_v8_get_exception(context: *mut V8ContextHandle) -> *mut V8ValueHandle;
    fn klyron_v8_get_exception_message(context: *mut V8ContextHandle) -> *const c_char;
    fn klyron_v8_get_stack_trace(context: *mut V8ContextHandle) -> V8StringResult;

    /* Utility */
    fn klyron_v8_version() -> *const c_char;
    fn klyron_v8_major_version() -> c_int;
    fn klyron_v8_minor_version() -> c_int;
    fn klyron_v8_build_version() -> c_int;
    fn klyron_v8_patch_version() -> c_int;
    fn klyron_v8_free_string(s: *mut c_char);
    fn klyron_v8_free_buffer(buf: *mut c_uchar);
}

pub struct V8EnginePtr {
    isolate: *mut V8IsolateHandle,
    context: *mut V8ContextHandle,
    init_done: bool,
}

impl V8EnginePtr {
    pub fn init() -> Result<Self, String> {
        let config = V8Config {
            icu_data_path: std::ptr::null(),
            snapshot_blob_path: std::ptr::null(),
            max_heap_size_mb: 0,
            initial_heap_size_mb: 0,
            array_buffer_allocator_pool_size: 0,
            use_shared_memory: false,
            expose_gc: false,
            single_threaded: true,
        };

        unsafe {
            klyron_v8_init(&config);
            if !klyron_v8_is_initialized() {
                return Err("V8 platform init failed".into());
            }
            let isolate = klyron_v8_isolate_new();
            if isolate.is_null() {
                klyron_v8_shutdown();
                return Err("V8 isolate creation failed".into());
            }
            let context = klyron_v8_context_new(isolate);
            if context.is_null() {
                klyron_v8_isolate_dispose(isolate);
                klyron_v8_shutdown();
                return Err("V8 context creation failed".into());
            }
            Ok(Self { isolate, context, init_done: true })
        }
    }

    /* ─── helpers ───────────────────────────────────────────── */

    fn check(result: V8StringResult) -> Result<String, String> {
        if result.success {
            let s = if result.data.is_null() {
                String::new()
            } else {
                let s = unsafe { CStr::from_ptr(result.data).to_string_lossy().into() };
                unsafe { klyron_v8_free_string(result.data) };
                s
            };
            Ok(s)
        } else {
            let err = unsafe { CStr::from_ptr(result.error.as_ptr()).to_string_lossy().into() };
            Err(err)
        }
    }

    fn check_void(result: V8Result) -> Result<(), String> {
        if result.success { Ok(()) }
        else { Err(unsafe { CStr::from_ptr(result.error.as_ptr()).to_string_lossy().into() }) }
    }

    #[inline]
    fn to_cstring(s: &str) -> Result<CString, String> {
        CString::new(s).map_err(|e| e.to_string())
    }

    pub fn isolate_handle(&self) -> *mut V8IsolateHandle { self.isolate }
    pub fn context_handle(&self) -> *mut V8ContextHandle { self.context }

    /* ─── eval / script ─────────────────────────────────────── */

    pub fn eval(&self, code: &str) -> Result<String, String> {
        let c = Self::to_cstring(code)?;
        let r = unsafe { klyron_v8_eval(self.context, c.as_ptr(), std::ptr::null()) };
        Self::check(r)
    }

    pub fn compile(&self, source: &str, filename: Option<&str>) -> Result<*mut V8ScriptHandle, String> {
        let c_source = Self::to_cstring(source)?;
        let c_file = filename.map(|f| Self::to_cstring(f)).transpose()?;
        let ptr = unsafe {
            klyron_v8_compile(self.context, c_source.as_ptr(), c_file.as_ref().map_or(std::ptr::null(), |f| f.as_ptr()))
        };
        if ptr.is_null() { Err("compile failed".into()) } else { Ok(ptr) }
    }

    pub fn run(&self, script: *mut V8ScriptHandle) -> Result<String, String> {
        let r = unsafe { klyron_v8_run(self.context, script) };
        Self::check(r)
    }

    pub fn script_dispose(&self, script: *mut V8ScriptHandle) {
        unsafe { klyron_v8_script_dispose(script) }
    }

    pub fn execute_script(&self, filename: &str, source: &str) -> Result<String, String> {
        let c_file = Self::to_cstring(filename)?;
        let c_src = Self::to_cstring(source)?;
        let r = unsafe { klyron_v8_eval(self.context, c_src.as_ptr(), c_file.as_ptr()) };
        Self::check(r)
    }

    pub fn execute_module(&self, filename: &str, source: &str) -> Result<String, String> {
        let c_file = Self::to_cstring(filename)?;
        let c_src = Self::to_cstring(source)?;
        let mod_ptr = unsafe { klyron_v8_module_compile(self.context, c_src.as_ptr(), c_file.as_ptr()) };
        if mod_ptr.is_null() { return Err("module compile failed".into()) }
        let r = unsafe { klyron_v8_module_instantiate(self.context, mod_ptr) };
        Self::check_void(r)?;
        let r = unsafe { klyron_v8_module_evaluate(self.context, mod_ptr) };
        let result = Self::check(r);
        unsafe { klyron_v8_module_dispose(mod_ptr) };
        result
    }

    /* ─── JSON ──────────────────────────────────────────────── */

    pub fn json_stringify(&self, value: *mut V8ValueHandle) -> Result<String, String> {
        Self::check(unsafe { klyron_v8_json_stringify(self.context, value) })
    }

    pub fn json_parse(&self, json: &str) -> Result<*mut V8ValueHandle, String> {
        let c = Self::to_cstring(json)?;
        let ptr = unsafe { klyron_v8_json_parse(self.context, c.as_ptr()) };
        if ptr.is_null() { Err("JSON parse failed".into()) } else { Ok(ptr) }
    }

    /* ─── global ────────────────────────────────────────────── */

    pub fn set_global(&self, name: &str, value: *mut V8ValueHandle) -> Result<(), String> {
        let c = Self::to_cstring(name)?;
        Self::check_void(unsafe { klyron_v8_set_global(self.context, c.as_ptr(), value) })
    }

    pub fn get_global(&self, name: &str) -> Result<*mut V8ValueHandle, String> {
        let c = Self::to_cstring(name)?;
        let ptr = unsafe { klyron_v8_get_global(self.context, c.as_ptr()) };
        if ptr.is_null() { Err("global get failed".into()) } else { Ok(ptr) }
    }

    /* ─── value creation ────────────────────────────────────── */

    pub fn value_new_string(&self, s: &str) -> Result<*mut V8ValueHandle, String> {
        let c = Self::to_cstring(s)?;
        let ptr = unsafe { klyron_v8_value_new_string(self.context, c.as_ptr()) };
        if ptr.is_null() { Err("value_new_string failed".into()) } else { Ok(ptr) }
    }

    pub fn value_new_number(&self, n: f64) -> *mut V8ValueHandle {
        unsafe { klyron_v8_value_new_number(self.context, n) }
    }

    pub fn value_new_bool(&self, v: bool) -> *mut V8ValueHandle {
        unsafe { klyron_v8_value_new_bool(self.context, v) }
    }

    pub fn value_new_null(&self) -> *mut V8ValueHandle {
        unsafe { klyron_v8_value_new_null(self.context) }
    }

    pub fn value_new_undefined(&self) -> *mut V8ValueHandle {
        unsafe { klyron_v8_value_new_undefined(self.context) }
    }

    pub fn value_new_object(&self) -> *mut V8ValueHandle {
        unsafe { klyron_v8_value_new_object(self.context) }
    }

    pub fn value_new_array(&self) -> *mut V8ValueHandle {
        unsafe { klyron_v8_value_new_array(self.context) }
    }

    pub fn value_typeof(&self, v: *mut V8ValueHandle) -> V8TypeResult {
        unsafe { klyron_v8_value_typeof(self.context, v) }
    }

    pub fn value_to_string(&self, v: *mut V8ValueHandle) -> Result<String, String> {
        Self::check(unsafe { klyron_v8_value_to_string(self.context, v) })
    }

    pub fn value_to_number(&self, v: *mut V8ValueHandle) -> f64 {
        unsafe { klyron_v8_value_to_number(self.context, v) }
    }

    pub fn value_to_bool(&self, v: *mut V8ValueHandle) -> bool {
        unsafe { klyron_v8_value_to_bool(self.context, v) }
    }

    pub fn value_is_array(&self, v: *mut V8ValueHandle) -> bool {
        unsafe { klyron_v8_value_is_array(self.context, v) }
    }

    pub fn value_dispose(&self, v: *mut V8ValueHandle) {
        unsafe { klyron_v8_value_dispose(v) }
    }

    /* ─── snaphots ──────────────────────────────────────────── */

    pub fn create_snapshot(&self) -> Result<*mut V8SnapshotHandle, String> {
        let ptr = unsafe { klyron_v8_snapshot_create(self.context) };
        if ptr.is_null() { Err("snapshot create failed (runtime snapshots not supported)".into()) }
        else { Ok(ptr) }
    }

    pub fn load_snapshot(&self, data: &[u8]) -> Result<*mut V8SnapshotHandle, String> {
        let ptr = unsafe { klyron_v8_snapshot_load(data.as_ptr() as *const c_char, data.len()) };
        if ptr.is_null() { Err("snapshot load failed".into()) } else { Ok(ptr) }
    }

    pub fn snapshot_dispose(&self, s: *mut V8SnapshotHandle) {
        unsafe { klyron_v8_snapshot_dispose(s) }
    }

    /* ─── promise ───────────────────────────────────────────── */

    pub fn promise_new(&self) -> Result<*mut V8PromiseHandle, String> {
        let ptr = unsafe { klyron_v8_promise_new(self.context) };
        if ptr.is_null() { Err("promise creation failed".into()) } else { Ok(ptr) }
    }

    pub fn promise_resolve(&self, p: *mut V8PromiseHandle, v: *mut V8ValueHandle) -> Result<(), String> {
        Self::check_void(unsafe { klyron_v8_promise_resolve(self.context, p, v) })
    }

    pub fn promise_reject(&self, p: *mut V8PromiseHandle, reason: &str) -> Result<(), String> {
        let c = Self::to_cstring(reason)?;
        Self::check_void(unsafe { klyron_v8_promise_reject(self.context, p, c.as_ptr()) })
    }

    pub fn promise_state(&self, p: *mut V8PromiseHandle) -> u32 {
        unsafe { klyron_v8_promise_get_state(self.context, p) }
    }

    pub fn promise_has_handler(&self, p: *mut V8PromiseHandle) -> bool {
        unsafe { klyron_v8_promise_has_handler(self.context, p) }
    }

    pub fn promise_mark_as_handled(&self, p: *mut V8PromiseHandle) -> Result<(), String> {
        Self::check_void(unsafe { klyron_v8_promise_mark_as_handled(self.context, p) })
    }

    /* ─── module ────────────────────────────────────────────── */

    pub fn module_compile(&self, source: &str, origin: Option<&str>) -> Result<*mut V8ModuleHandle, String> {
        let c_src = Self::to_cstring(source)?;
        let c_origin = origin.map(|o| Self::to_cstring(o)).transpose()?;
        let ptr = unsafe {
            klyron_v8_module_compile(self.context, c_src.as_ptr(), c_origin.as_ref().map_or(std::ptr::null(), |o| o.as_ptr()))
        };
        if ptr.is_null() { Err("module compile failed".into()) } else { Ok(ptr) }
    }

    pub fn module_instantiate(&self, m: *mut V8ModuleHandle) -> Result<(), String> {
        Self::check_void(unsafe { klyron_v8_module_instantiate(self.context, m) })
    }

    pub fn module_evaluate(&self, m: *mut V8ModuleHandle) -> Result<String, String> {
        Self::check(unsafe { klyron_v8_module_evaluate(self.context, m) })
    }

    pub fn module_identity(&self, m: *mut V8ModuleHandle) -> i32 {
        unsafe { klyron_v8_module_get_identity(self.context, m) }
    }

    pub fn module_dispose(&self, m: *mut V8ModuleHandle) {
        unsafe { klyron_v8_module_dispose(m) }
    }

    /* ─── heap / memory / GC ────────────────────────────────── */

    pub fn get_heap_stats(&self) -> Result<V8HeapStats, String> {
        let mut stats = V8HeapStats::zeroed();
        let r = unsafe { klyron_v8_get_heap_stats(self.isolate, &mut stats) };
        if r.success { Ok(stats) } else { Err("get_heap_stats failed".into()) }
    }

    pub fn low_memory_notification(&self) {
        unsafe { klyron_v8_low_memory_notification(self.isolate) }
    }

    pub fn idle_notification(&self, deadline: f64) {
        unsafe { klyron_v8_idle_notification(self.isolate, deadline) }
    }

    pub fn set_memory_pressure(&self, pressure: u32) {
        unsafe { klyron_v8_set_memory_pressure(self.isolate, pressure) }
    }

    pub fn request_gc(&self) {
        unsafe { klyron_v8_request_gc(self.isolate) }
    }

    pub fn get_malloced_memory(&self) -> usize {
        unsafe { klyron_v8_get_malloced_memory(self.isolate) }
    }

    pub fn adjust_external_memory(&self, change: i64) -> usize {
        unsafe { klyron_v8_adjust_external_memory(self.isolate, change) }
    }

    /* ─── microtasks ────────────────────────────────────────── */

    pub fn microtasks_perform_check(&self) {
        unsafe { klyron_v8_microtasks_perform_check(self.context) }
    }

    /* ─── error / stack ─────────────────────────────────────── */

    pub fn get_exception_message(&self) -> String {
        unsafe {
            let p = klyron_v8_get_exception_message(self.context);
            if p.is_null() { String::new() }
            else { CStr::from_ptr(p).to_string_lossy().into() }
        }
    }

    pub fn get_stack_trace(&self) -> Result<String, String> {
        Self::check(unsafe { klyron_v8_get_stack_trace(self.context) })
    }

    /* ─── version ───────────────────────────────────────────── */

    pub fn version() -> String {
        unsafe {
            let p = klyron_v8_version();
            if p.is_null() { String::new() }
            else { CStr::from_ptr(p).to_string_lossy().into() }
        }
    }

    pub fn major_version() -> i32 { unsafe { klyron_v8_major_version() } }
    pub fn minor_version() -> i32 { unsafe { klyron_v8_minor_version() } }
    pub fn build_version() -> i32 { unsafe { klyron_v8_build_version() } }
    pub fn patch_version() -> i32 { unsafe { klyron_v8_patch_version() } }
}

impl Drop for V8EnginePtr {
    fn drop(&mut self) {
        unsafe {
            if !self.context.is_null() {
                klyron_v8_context_dispose(self.context);
                self.context = std::ptr::null_mut();
            }
            if !self.isolate.is_null() {
                klyron_v8_isolate_dispose(self.isolate);
                self.isolate = std::ptr::null_mut();
            }
            if self.init_done {
                klyron_v8_shutdown();
            }
        }
    }
}

unsafe impl Send for V8EnginePtr {}
unsafe impl Sync for V8EnginePtr {}
