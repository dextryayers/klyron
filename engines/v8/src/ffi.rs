#![cfg(feature = "native")]

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_double, c_int, c_uint, c_void, c_uchar};

pub use crate::ffi_types::*;

/* Opaque handle types matching C typedefs */
#[repr(C)] pub struct V8IsolateHandle  { _private: [u8; 0] }
#[repr(C)] pub struct V8ContextHandle  { _private: [u8; 0] }
#[repr(C)] pub struct V8ValueHandle    { _private: [u8; 0] }
#[repr(C)] pub struct V8ScriptHandle   { _private: [u8; 0] }
#[repr(C)] pub struct V8ModuleHandle   { _private: [u8; 0] }
#[repr(C)] pub struct V8PromiseHandle  { _private: [u8; 0] }
#[repr(C)] pub struct V8SnapshotHandle { _private: [u8; 0] }
#[repr(C)] pub struct V8Stream         { _private: [u8; 0] }

#[repr(C)]
pub struct V8Url {
    pub href: *mut c_char,
    pub protocol: *mut c_char,
    pub hostname: *mut c_char,
    pub port: *mut c_char,
    pub pathname: *mut c_char,
    pub search: *mut c_char,
    pub hash: *mut c_char,
    pub host: *mut c_char,
    pub origin: *mut c_char,
}

#[repr(C)]
pub struct V8Stat {
    pub dev: u64,
    pub ino: u64,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub size: u64,
    pub blksize: u64,
    pub blocks: u64,
    pub atime: u64,
    pub mtime: u64,
    pub ctime: u64,
    pub file_type: i32,
}

#[repr(C)]
pub struct V8ProcessInfo {
    pub exec_path: *mut c_char,
    pub argv: *mut *mut c_char,
    pub argc: c_int,
    pub cwd: *mut c_char,
    pub platform: *mut c_char,
    pub pid: u64,
    pub title: *mut c_char,
    pub ppid: u64,
}

unsafe extern "C" {
    /* Platform lifecycle */
    pub fn klyron_v8_init(config: *const V8Config);
    pub fn klyron_v8_shutdown();
    pub fn klyron_v8_is_initialized() -> bool;

    /* Isolate */
    pub fn klyron_v8_isolate_new() -> *mut V8IsolateHandle;
    pub fn klyron_v8_isolate_dispose(isolate: *mut V8IsolateHandle);
    pub fn klyron_v8_isolate_enter(isolate: *mut V8IsolateHandle);
    pub fn klyron_v8_isolate_exit(isolate: *mut V8IsolateHandle);

    /* Context */
    pub fn klyron_v8_context_new(isolate: *mut V8IsolateHandle) -> *mut V8ContextHandle;
    pub fn klyron_v8_context_dispose(context: *mut V8ContextHandle);
    pub fn klyron_v8_context_enter(context: *mut V8ContextHandle);
    pub fn klyron_v8_context_exit(context: *mut V8ContextHandle);
    pub fn klyron_v8_context_new_from_snapshot(
        isolate: *mut V8IsolateHandle,
        snapshot: *mut V8SnapshotHandle,
    ) -> *mut V8ContextHandle;

    /* Script */
    pub fn klyron_v8_compile(
        context: *mut V8ContextHandle,
        source: *const c_char,
        filename: *const c_char,
    ) -> *mut V8ScriptHandle;
    pub fn klyron_v8_run(
        context: *mut V8ContextHandle,
        script: *mut V8ScriptHandle,
    ) -> V8StringResult;
    pub fn klyron_v8_eval(
        context: *mut V8ContextHandle,
        source: *const c_char,
        filename: *const c_char,
    ) -> V8StringResult;
    pub fn klyron_v8_script_dispose(script: *mut V8ScriptHandle);

    /* JSON */
    pub fn klyron_v8_json_stringify(
        context: *mut V8ContextHandle,
        value: *mut V8ValueHandle,
    ) -> V8StringResult;
    pub fn klyron_v8_json_parse(
        context: *mut V8ContextHandle,
        json: *const c_char,
    ) -> *mut V8ValueHandle;

    /* Global */
    pub fn klyron_v8_set_global(
        context: *mut V8ContextHandle,
        name: *const c_char,
        value: *mut V8ValueHandle,
    ) -> V8Result;
    pub fn klyron_v8_get_global(
        context: *mut V8ContextHandle,
        name: *const c_char,
    ) -> *mut V8ValueHandle;

    /* Value creation */
    pub fn klyron_v8_value_new_string(context: *mut V8ContextHandle, str: *const c_char) -> *mut V8ValueHandle;
    pub fn klyron_v8_value_new_number(context: *mut V8ContextHandle, num: c_double) -> *mut V8ValueHandle;
    pub fn klyron_v8_value_new_bool(context: *mut V8ContextHandle, val: bool) -> *mut V8ValueHandle;
    pub fn klyron_v8_value_new_null(context: *mut V8ContextHandle) -> *mut V8ValueHandle;
    pub fn klyron_v8_value_new_undefined(context: *mut V8ContextHandle) -> *mut V8ValueHandle;
    pub fn klyron_v8_value_new_object(context: *mut V8ContextHandle) -> *mut V8ValueHandle;
    pub fn klyron_v8_value_new_array(context: *mut V8ContextHandle) -> *mut V8ValueHandle;

    /* Value inspection */
    pub fn klyron_v8_value_typeof(context: *mut V8ContextHandle, value: *mut V8ValueHandle) -> V8TypeResult;
    pub fn klyron_v8_value_to_string(context: *mut V8ContextHandle, value: *mut V8ValueHandle) -> V8StringResult;
    pub fn klyron_v8_value_to_number(context: *mut V8ContextHandle, value: *mut V8ValueHandle) -> c_double;
    pub fn klyron_v8_value_to_bool(context: *mut V8ContextHandle, value: *mut V8ValueHandle) -> bool;
    pub fn klyron_v8_value_is_array(context: *mut V8ContextHandle, value: *mut V8ValueHandle) -> bool;
    pub fn klyron_v8_value_dispose(value: *mut V8ValueHandle);

    /* Native function */
    pub fn klyron_v8_function_new(
        context: *mut V8ContextHandle,
        name: *const c_char,
        callback: Option<unsafe extern "C" fn(
            *mut V8ContextHandle, c_int, *mut *mut V8ValueHandle,
            *mut c_void, *mut *mut V8ValueHandle)>,
        user_data: *mut c_void,
    ) -> *mut V8ValueHandle;

    /* Object property access */
    pub fn klyron_v8_object_set_property(
        context: *mut V8ContextHandle,
        object: *mut V8ValueHandle,
        name: *const c_char,
        value: *mut V8ValueHandle,
    ) -> V8Result;
    pub fn klyron_v8_object_get_property(
        context: *mut V8ContextHandle,
        object: *mut V8ValueHandle,
        name: *const c_char,
    ) -> *mut V8ValueHandle;

    /* Typed arrays */
    pub fn klyron_v8_get_typed_array_type(
        context: *mut V8ContextHandle,
        value: *mut V8ValueHandle,
    ) -> u32;
    pub fn klyron_v8_typed_array_new(
        context: *mut V8ContextHandle,
        type_name: *const c_char,
        length: usize,
    ) -> *mut V8ValueHandle;
    pub fn klyron_v8_typed_array_get_length(
        context: *mut V8ContextHandle,
        value: *mut V8ValueHandle,
    ) -> usize;
    pub fn klyron_v8_typed_array_get_buffer(
        context: *mut V8ContextHandle,
        value: *mut V8ValueHandle,
    ) -> *mut V8ValueHandle;
    pub fn klyron_v8_array_buffer_new(
        context: *mut V8ContextHandle,
        data: *const c_uchar,
        length: usize,
    ) -> *mut V8ValueHandle;

    /* Promise */
    pub fn klyron_v8_promise_new(context: *mut V8ContextHandle) -> *mut V8PromiseHandle;
    pub fn klyron_v8_promise_resolve(
        context: *mut V8ContextHandle,
        promise: *mut V8PromiseHandle,
        value: *mut V8ValueHandle,
    ) -> V8Result;
    pub fn klyron_v8_promise_reject(
        context: *mut V8ContextHandle,
        promise: *mut V8PromiseHandle,
        reason: *const c_char,
    ) -> V8Result;
    pub fn klyron_v8_promise_get_native(promise: *mut V8PromiseHandle) -> *mut V8ValueHandle;
    pub fn klyron_v8_promise_has_handler(context: *mut V8ContextHandle, promise: *mut V8PromiseHandle) -> bool;
    pub fn klyron_v8_promise_mark_as_handled(context: *mut V8ContextHandle, promise: *mut V8PromiseHandle) -> V8Result;
    pub fn klyron_v8_promise_get_state(context: *mut V8ContextHandle, promise: *mut V8PromiseHandle) -> c_uint;

    /* Microtasks */
    pub fn klyron_v8_microtasks_perform_check(context: *mut V8ContextHandle);

    /* Module */
    pub fn klyron_v8_module_compile(context: *mut V8ContextHandle, source: *const c_char, origin: *const c_char)
        -> *mut V8ModuleHandle;
    pub fn klyron_v8_module_instantiate(context: *mut V8ContextHandle, module: *mut V8ModuleHandle) -> V8Result;
    pub fn klyron_v8_module_evaluate(context: *mut V8ContextHandle, module: *mut V8ModuleHandle) -> V8StringResult;
    pub fn klyron_v8_module_get_identity(context: *mut V8ContextHandle, module: *mut V8ModuleHandle) -> c_int;
    pub fn klyron_v8_module_dispose(module: *mut V8ModuleHandle);

    /* Heap & memory */
    pub fn klyron_v8_get_heap_stats(isolate: *mut V8IsolateHandle, stats: *mut V8HeapStats) -> V8Result;
    pub fn klyron_v8_low_memory_notification(isolate: *mut V8IsolateHandle);
    pub fn klyron_v8_idle_notification(isolate: *mut V8IsolateHandle, deadline: c_double);
    pub fn klyron_v8_set_memory_pressure(isolate: *mut V8IsolateHandle, pressure: c_uint);
    pub fn klyron_v8_request_gc(isolate: *mut V8IsolateHandle);
    pub fn klyron_v8_get_malloced_memory(isolate: *mut V8IsolateHandle) -> usize;
    pub fn klyron_v8_adjust_external_memory(isolate: *mut V8IsolateHandle, change: i64) -> usize;

    /* Snapshots */
    pub fn klyron_v8_snapshot_create(context: *mut V8ContextHandle) -> *mut V8SnapshotHandle;
    pub fn klyron_v8_snapshot_load(blob: *const c_char, length: usize) -> *mut V8SnapshotHandle;
    pub fn klyron_v8_snapshot_dispose(snapshot: *mut V8SnapshotHandle);

    /* Error */
    pub fn klyron_v8_get_exception(context: *mut V8ContextHandle) -> *mut V8ValueHandle;
    pub fn klyron_v8_get_exception_message(context: *mut V8ContextHandle) -> *const c_char;
    pub fn klyron_v8_get_stack_trace(context: *mut V8ContextHandle) -> V8StringResult;

    /* Utility */
    pub fn klyron_v8_version() -> *const c_char;
    pub fn klyron_v8_major_version() -> c_int;
    pub fn klyron_v8_minor_version() -> c_int;
    pub fn klyron_v8_build_version() -> c_int;
    pub fn klyron_v8_patch_version() -> c_int;
    pub fn klyron_v8_free_string(s: *mut c_char);
    pub fn klyron_v8_free_buffer(buf: *mut c_uchar);

    /* Buffer */
    pub fn klyron_v8_buffer_new(ctx: *mut V8ContextHandle, size: usize) -> *mut V8ValueHandle;
    pub fn klyron_v8_buffer_from_string(ctx: *mut V8ContextHandle, str: *const c_char, encoding: *const c_char) -> *mut V8ValueHandle;
    pub fn klyron_v8_buffer_from_bytes(ctx: *mut V8ContextHandle, data: *const c_uchar, length: usize) -> *mut V8ValueHandle;
    pub fn klyron_v8_buffer_to_string(ctx: *mut V8ContextHandle, buf: *mut V8ValueHandle, encoding: *const c_char, start: usize, end: usize) -> V8StringResult;
    pub fn klyron_v8_buffer_get_data(ctx: *mut V8ContextHandle, buf: *mut V8ValueHandle) -> *mut c_uchar;
    pub fn klyron_v8_buffer_get_length(ctx: *mut V8ContextHandle, buf: *mut V8ValueHandle) -> usize;
    pub fn klyron_v8_buffer_copy(ctx: *mut V8ContextHandle, dst: *mut V8ValueHandle, dst_offset: usize, src: *mut V8ValueHandle, src_offset: usize, count: usize) -> V8Result;
    pub fn klyron_v8_buffer_concat(ctx: *mut V8ContextHandle, bufs: *mut *mut V8ValueHandle, count: usize) -> *mut V8ValueHandle;
    pub fn klyron_v8_buffer_slice(ctx: *mut V8ContextHandle, buf: *mut V8ValueHandle, start: usize, end: usize) -> *mut V8ValueHandle;
    pub fn klyron_v8_buffer_write(ctx: *mut V8ContextHandle, buf: *mut V8ValueHandle, data: *const c_uchar, offset: usize, length: usize) -> V8Result;

    /* Console */
    pub fn klyron_v8_console_new(ctx: *mut V8ContextHandle) -> *mut V8ValueHandle;
    pub fn klyron_v8_console_log(ctx: *mut V8ContextHandle, msg: *const c_char);
    pub fn klyron_v8_console_warn(ctx: *mut V8ContextHandle, msg: *const c_char);
    pub fn klyron_v8_console_error(ctx: *mut V8ContextHandle, msg: *const c_char);
    pub fn klyron_v8_console_info(ctx: *mut V8ContextHandle, msg: *const c_char);
    pub fn klyron_v8_console_debug(ctx: *mut V8ContextHandle, msg: *const c_char);

    /* Crypto */
    pub fn klyron_v8_crypto_random_bytes(ctx: *mut V8ContextHandle, size: usize) -> *mut V8ValueHandle;
    pub fn klyron_v8_crypto_random_uuid(ctx: *mut V8ContextHandle) -> V8StringResult;

    /* Encoding */
    pub fn klyron_v8_encoding_text_encoder_new(ctx: *mut V8ContextHandle) -> *mut V8ValueHandle;
    pub fn klyron_v8_encoding_text_decoder_new(ctx: *mut V8ContextHandle, label: *const c_char) -> *mut V8ValueHandle;
    pub fn klyron_v8_encoding_encode(ctx: *mut V8ContextHandle, input: *const c_char) -> *mut V8ValueHandle;
    pub fn klyron_v8_encoding_decode(ctx: *mut V8ContextHandle, data: *const c_uchar, length: usize, encoding: *const c_char) -> V8StringResult;
    pub fn klyron_v8_encoding_base64_encode(ctx: *mut V8ContextHandle, data: *const c_uchar, length: usize) -> V8StringResult;
    pub fn klyron_v8_encoding_base64_decode(ctx: *mut V8ContextHandle, input: *const c_char) -> *mut V8ValueHandle;
    pub fn klyron_v8_encoding_hex_encode(ctx: *mut V8ContextHandle, data: *const c_uchar, length: usize) -> V8StringResult;
    pub fn klyron_v8_encoding_hex_decode(ctx: *mut V8ContextHandle, input: *const c_char) -> *mut V8ValueHandle;

    /* WebAssembly */
    pub fn klyron_v8_wasm_compile(ctx: *mut V8ContextHandle, wasm_bytes: *const c_uchar, wasm_length: usize) -> *mut V8ValueHandle;
    pub fn klyron_v8_wasm_instantiate(ctx: *mut V8ContextHandle, wasm_bytes: *const c_uchar, wasm_length: usize, imports: *mut V8ValueHandle) -> *mut V8ValueHandle;

    /* Inspector */
    pub fn klyron_v8_inspector_new(isolate: *mut V8IsolateHandle) -> c_int;
    pub fn klyron_v8_inspector_dispose(inspector_id: c_int);
    pub fn klyron_v8_inspector_connect(inspector_id: c_int, url: *const c_char) -> c_int;
    pub fn klyron_v8_inspector_disconnect(session_id: c_int);
    pub fn klyron_v8_inspector_dispatch(session_id: c_int, message: *const c_char, out_response: *mut c_char, out_response_size: usize) -> c_int;
    pub fn klyron_v8_inspector_is_active() -> bool;

    /* Timers */
    pub fn klyron_v8_timer_set_timeout(ctx: *mut V8ContextHandle, cb: Option<unsafe extern "C" fn(*mut c_void)>, data: *mut c_void, ms: u64) -> c_int;
    pub fn klyron_v8_timer_set_interval(ctx: *mut V8ContextHandle, cb: Option<unsafe extern "C" fn(*mut c_void)>, data: *mut c_void, ms: u64) -> c_int;
    pub fn klyron_v8_timer_set_immediate(ctx: *mut V8ContextHandle, cb: Option<unsafe extern "C" fn(*mut c_void)>, data: *mut c_void) -> c_int;
    pub fn klyron_v8_timer_clear(id: c_int);
    pub fn klyron_v8_timer_clear_all();

    /* URL */
    pub fn klyron_v8_url_parse(url: *const c_char, base: *const c_char) -> *mut V8Url;
    pub fn klyron_v8_url_dispose(url: *mut V8Url);

    /* FS */
    pub fn klyron_v8_fs_read_file(ctx: *mut V8ContextHandle, path: *const c_char, result: *mut *mut V8ValueHandle) -> V8Result;
    pub fn klyron_v8_fs_write_file(ctx: *mut V8ContextHandle, path: *const c_char, data: *const c_uchar, length: usize) -> V8Result;
    pub fn klyron_v8_fs_append_file(ctx: *mut V8ContextHandle, path: *const c_char, data: *const c_uchar, length: usize) -> V8Result;
    pub fn klyron_v8_fs_stat(ctx: *mut V8ContextHandle, path: *const c_char, stat: *mut V8Stat) -> V8Result;
    pub fn klyron_v8_fs_mkdir(ctx: *mut V8ContextHandle, path: *const c_char, mode: i32) -> V8Result;
    pub fn klyron_v8_fs_rmdir(ctx: *mut V8ContextHandle, path: *const c_char) -> V8Result;
    pub fn klyron_v8_fs_unlink(ctx: *mut V8ContextHandle, path: *const c_char) -> V8Result;
    pub fn klyron_v8_fs_rename(ctx: *mut V8ContextHandle, old_path: *const c_char, new_path: *const c_char) -> V8Result;
    pub fn klyron_v8_fs_exists(ctx: *mut V8ContextHandle, path: *const c_char, exists: *mut bool) -> V8Result;
    pub fn klyron_v8_fs_read_dir(ctx: *mut V8ContextHandle, path: *const c_char) -> *mut V8ValueHandle;

    /* Process */
    pub fn klyron_v8_process_info(ctx: *mut V8ContextHandle) -> *mut V8ProcessInfo;
    pub fn klyron_v8_process_exit(ctx: *mut V8ContextHandle, code: c_int) -> V8Result;
    pub fn klyron_v8_process_env_get(ctx: *mut V8ContextHandle, name: *const c_char) -> V8StringResult;
    pub fn klyron_v8_process_env_all(ctx: *mut V8ContextHandle) -> *mut V8ValueHandle;
    pub fn klyron_v8_process_info_dispose(info: *mut V8ProcessInfo);

    /* Stream */
    pub fn klyron_v8_stream_new_readable(ctx: *mut V8ContextHandle, read_cb: Option<unsafe extern "C" fn(*mut V8Stream, *mut c_uchar, usize, *mut c_void) -> usize>, user_data: *mut c_void) -> *mut V8Stream;
    pub fn klyron_v8_stream_new_writable(ctx: *mut V8ContextHandle, write_cb: Option<unsafe extern "C" fn(*mut V8Stream, *const c_uchar, usize, *mut c_void) -> usize>, user_data: *mut c_void) -> *mut V8Stream;
    pub fn klyron_v8_stream_new_transform(ctx: *mut V8ContextHandle, read_cb: Option<unsafe extern "C" fn(*mut V8Stream, *mut c_uchar, usize, *mut c_void) -> usize>, write_cb: Option<unsafe extern "C" fn(*mut V8Stream, *const c_uchar, usize, *mut c_void) -> usize>, user_data: *mut c_void) -> *mut V8Stream;
    pub fn klyron_v8_stream_write(ctx: *mut V8ContextHandle, stream: *mut V8Stream, data: *const c_uchar, length: usize) -> V8Result;
    pub fn klyron_v8_stream_end(ctx: *mut V8ContextHandle, stream: *mut V8Stream, data: *const c_uchar, length: usize) -> V8Result;
    pub fn klyron_v8_stream_destroy(ctx: *mut V8ContextHandle, stream: *mut V8Stream) -> V8Result;
    pub fn klyron_v8_stream_set_close_callback(stream: *mut V8Stream, cb: Option<unsafe extern "C" fn(*mut V8Stream, *mut c_void)>, user_data: *mut c_void);
}

pub struct V8EnginePtr {
    isolate: *mut V8IsolateHandle,
    context: *mut V8ContextHandle,
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
            expose_gc: true,
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
            Ok(Self { isolate, context })
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

    /* ─── native function ────────────────────────────────────── */

    pub fn function_new(&self, name: Option<&str>) -> Result<*mut V8ValueHandle, String> {
        let c = name.map(|n| CString::new(n)).transpose().map_err(|e| e.to_string())?;
        let ptr = unsafe {
            klyron_v8_function_new(self.context, c.as_ref().map_or(std::ptr::null(), |s| s.as_ptr()),
                                    None, std::ptr::null_mut())
        };
        if ptr.is_null() { Err("function_new failed".into()) } else { Ok(ptr) }
    }

    /* ─── object property ────────────────────────────────────── */

    pub fn object_set_property(&self, object: *mut V8ValueHandle, name: &str, value: *mut V8ValueHandle) -> Result<(), String> {
        let c = CString::new(name).map_err(|e| e.to_string())?;
        let r = unsafe { klyron_v8_object_set_property(self.context, object, c.as_ptr(), value) };
        V8EnginePtr::check_void(r)
    }

    pub fn object_get_property(&self, object: *mut V8ValueHandle, name: &str) -> Result<*mut V8ValueHandle, String> {
        let c = CString::new(name).map_err(|e| e.to_string())?;
        let ptr = unsafe { klyron_v8_object_get_property(self.context, object, c.as_ptr()) };
        if ptr.is_null() { Err("object_get_property failed".into()) } else { Ok(ptr) }
    }

    /* ─── typed array ────────────────────────────────────────── */

    pub fn typed_array_new(&self, type_name: &str, length: usize) -> Result<*mut V8ValueHandle, String> {
        let c = CString::new(type_name).map_err(|e| e.to_string())?;
        let ptr = unsafe { klyron_v8_typed_array_new(self.context, c.as_ptr(), length) };
        if ptr.is_null() { Err("typed_array_new failed".into()) } else { Ok(ptr) }
    }

    pub fn typed_array_get_length(&self, value: *mut V8ValueHandle) -> usize {
        unsafe { klyron_v8_typed_array_get_length(self.context, value) }
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
        }
    }
}

unsafe impl Send for V8EnginePtr {}
unsafe impl Sync for V8EnginePtr {}
