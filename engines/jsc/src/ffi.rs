#![cfg(feature = "native")]

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_double, c_int, c_uint, c_ulong, c_void, c_uchar};

use crate::ffi_types::*;

/* Opaque handle types matching C typedefs */
#[repr(C)] pub struct JSCEngineHandle { _private: [u8; 0] }
#[repr(C)] pub struct JSCValueHandle   { _private: [u8; 0] }
#[repr(C)] pub struct JSCScriptHandle  { _private: [u8; 0] }

unsafe extern "C" {
    /* Engine lifecycle */
    fn klyron_jsc_init() -> *mut JSCEngineHandle;
    fn klyron_jsc_shutdown(engine: *mut JSCEngineHandle);

    /* Isolate (stub in JSC) */
    fn klyron_jsc_isolate_enter(engine: *mut JSCEngineHandle);
    fn klyron_jsc_isolate_exit(engine: *mut JSCEngineHandle);

    /* Context (stub in JSC) */
    fn klyron_jsc_context_enter(engine: *mut JSCEngineHandle);
    fn klyron_jsc_context_exit(engine: *mut JSCEngineHandle);

    /* Script */
    fn klyron_jsc_compile(
        engine: *mut JSCEngineHandle,
        source: *const c_char,
        filename: *const c_char,
    ) -> *mut JSCScriptHandle;
    fn klyron_jsc_run(
        engine: *mut JSCEngineHandle,
        script: *mut JSCScriptHandle,
    ) -> JSCStringResult;
    fn klyron_jsc_eval(
        engine: *mut JSCEngineHandle,
        source: *const c_char,
        filename: *const c_char,
    ) -> JSCStringResult;
    fn klyron_jsc_script_dispose(script: *mut JSCScriptHandle);

    /* Call function */
    fn klyron_jsc_call_function(
        engine: *mut JSCEngineHandle,
        func: *mut JSCValueHandle,
        this_obj: *mut JSCValueHandle,
        argc: c_int,
        argv: *mut *mut JSCValueHandle,
    ) -> *mut JSCValueHandle;

    /* Object property access */
    fn klyron_jsc_object_set_property(
        engine: *mut JSCEngineHandle,
        object: *mut JSCValueHandle,
        name: *const c_char,
        value: *mut JSCValueHandle,
    ) -> JSCResult;
    fn klyron_jsc_object_get_property(
        engine: *mut JSCEngineHandle,
        object: *mut JSCValueHandle,
        name: *const c_char,
    ) -> *mut JSCValueHandle;

    /* JSON */
    fn klyron_jsc_json_stringify(
        engine: *mut JSCEngineHandle,
        value: *mut JSCValueHandle,
    ) -> JSCStringResult;
    fn klyron_jsc_json_parse(
        engine: *mut JSCEngineHandle,
        json: *const c_char,
    ) -> *mut JSCValueHandle;

    /* Global */
    fn klyron_jsc_set_global(
        engine: *mut JSCEngineHandle,
        name: *const c_char,
        value: *mut JSCValueHandle,
    ) -> JSCResult;
    fn klyron_jsc_get_global(
        engine: *mut JSCEngineHandle,
        name: *const c_char,
    ) -> *mut JSCValueHandle;

    /* Value creation */
    fn klyron_jsc_value_new_string(engine: *mut JSCEngineHandle, str: *const c_char) -> *mut JSCValueHandle;
    fn klyron_jsc_value_new_number(engine: *mut JSCEngineHandle, num: c_double) -> *mut JSCValueHandle;
    fn klyron_jsc_value_new_bool(engine: *mut JSCEngineHandle, val: bool) -> *mut JSCValueHandle;
    fn klyron_jsc_value_new_null(engine: *mut JSCEngineHandle) -> *mut JSCValueHandle;
    fn klyron_jsc_value_new_undefined(engine: *mut JSCEngineHandle) -> *mut JSCValueHandle;
    fn klyron_jsc_value_new_object(engine: *mut JSCEngineHandle) -> *mut JSCValueHandle;
    fn klyron_jsc_value_new_array(engine: *mut JSCEngineHandle) -> *mut JSCValueHandle;
    fn klyron_jsc_value_new_symbol(engine: *mut JSCEngineHandle, description: *const c_char) -> *mut JSCValueHandle;
    fn klyron_jsc_value_new_error(engine: *mut JSCEngineHandle, message: *const c_char) -> *mut JSCValueHandle;

    /* Value inspection */
    fn klyron_jsc_value_typeof(engine: *mut JSCEngineHandle, value: *mut JSCValueHandle) -> JSCTypeResult;
    fn klyron_jsc_value_to_string(engine: *mut JSCEngineHandle, value: *mut JSCValueHandle) -> JSCStringResult;
    fn klyron_jsc_value_to_number(engine: *mut JSCEngineHandle, value: *mut JSCValueHandle) -> c_double;
    fn klyron_jsc_value_to_bool(engine: *mut JSCEngineHandle, value: *mut JSCValueHandle) -> bool;
    fn klyron_jsc_value_is_array(engine: *mut JSCEngineHandle, value: *mut JSCValueHandle) -> bool;
    fn klyron_jsc_value_is_function(engine: *mut JSCEngineHandle, value: *mut JSCValueHandle) -> bool;
    fn klyron_jsc_value_is_object(engine: *mut JSCEngineHandle, value: *mut JSCValueHandle) -> bool;
    fn klyron_jsc_value_is_error(engine: *mut JSCEngineHandle, value: *mut JSCValueHandle) -> bool;
    fn klyron_jsc_value_is_symbol(engine: *mut JSCEngineHandle, value: *mut JSCValueHandle) -> bool;
    fn klyron_jsc_value_is_promise(engine: *mut JSCEngineHandle, value: *mut JSCValueHandle) -> bool;
    fn klyron_jsc_value_is_typed_array(engine: *mut JSCEngineHandle, value: *mut JSCValueHandle) -> bool;
    fn klyron_jsc_value_is_null(engine: *mut JSCEngineHandle, value: *mut JSCValueHandle) -> bool;
    fn klyron_jsc_value_is_undefined(engine: *mut JSCEngineHandle, value: *mut JSCValueHandle) -> bool;
    fn klyron_jsc_value_is_boolean(engine: *mut JSCEngineHandle, value: *mut JSCValueHandle) -> bool;
    fn klyron_jsc_value_is_number(engine: *mut JSCEngineHandle, value: *mut JSCValueHandle) -> bool;
    fn klyron_jsc_value_is_string(engine: *mut JSCEngineHandle, value: *mut JSCValueHandle) -> bool;
    fn klyron_jsc_value_dispose(value: *mut JSCValueHandle);

    /* Promise */
    fn klyron_jsc_promise_new(engine: *mut JSCEngineHandle) -> *mut JSCValueHandle;
    fn klyron_jsc_promise_resolve(
        engine: *mut JSCEngineHandle,
        promise: *mut JSCValueHandle,
        value: *mut JSCValueHandle,
    ) -> JSCResult;
    fn klyron_jsc_promise_reject(
        engine: *mut JSCEngineHandle,
        promise: *mut JSCValueHandle,
        reason: *const c_char,
    ) -> JSCResult;

    /* Microtasks */
    fn klyron_jsc_microtasks_perform_check(engine: *mut JSCEngineHandle);

    /* Module */
    fn klyron_jsc_module_compile(
        engine: *mut JSCEngineHandle,
        source: *const c_char,
        origin: *const c_char,
    ) -> *mut JSCValueHandle;
    fn klyron_jsc_module_instantiate(
        engine: *mut JSCEngineHandle,
        module: *mut JSCValueHandle,
    ) -> JSCResult;
    fn klyron_jsc_module_evaluate(
        engine: *mut JSCEngineHandle,
        module: *mut JSCValueHandle,
    ) -> JSCStringResult;
    fn klyron_jsc_module_dispose(module: *mut JSCValueHandle);

    /* Heap & memory */
    fn klyron_jsc_get_heap_stats(engine: *mut JSCEngineHandle, stats: *mut JSCHeapStats) -> JSCResult;
    fn klyron_jsc_request_gc(engine: *mut JSCEngineHandle);
    fn klyron_jsc_low_memory_notification(engine: *mut JSCEngineHandle);

    /* ArrayBuffer / TypedArray */
    fn klyron_jsc_array_buffer_new(
        engine: *mut JSCEngineHandle,
        data: *const c_uchar,
        length: usize,
    ) -> *mut JSCValueHandle;
    fn klyron_jsc_array_buffer_get_data(
        engine: *mut JSCEngineHandle,
        value: *mut JSCValueHandle,
        out_length: *mut usize,
    ) -> *mut c_uchar;
    fn klyron_jsc_typed_array_new(
        engine: *mut JSCEngineHandle,
        type_: *const c_char,
        length: usize,
    ) -> *mut JSCValueHandle;
    fn klyron_jsc_typed_array_get_length(
        engine: *mut JSCEngineHandle,
        value: *mut JSCValueHandle,
    ) -> usize;
    fn klyron_jsc_typed_array_get_buffer(
        engine: *mut JSCEngineHandle,
        value: *mut JSCValueHandle,
    ) -> *mut JSCValueHandle;

    /* Error */
    fn klyron_jsc_get_exception_message(engine: *mut JSCEngineHandle) -> *const c_char;
    fn klyron_jsc_get_stack_trace(engine: *mut JSCEngineHandle) -> JSCStringResult;

    /* Utility */
    fn klyron_jsc_version() -> *const c_char;
    fn klyron_jsc_free_string(s: *mut c_char);
    fn klyron_jsc_free_buffer(buf: *mut c_uchar);
}

pub struct JSCEnginePtr {
    ptr: *mut JSCEngineHandle,
}

impl JSCEnginePtr {
    pub fn init() -> Result<Self, String> {
        let ptr = unsafe { klyron_jsc_init() };
        if ptr.is_null() {
            return Err("JSC: engine creation failed".into());
        }
        Ok(Self { ptr })
    }

    fn last_error(&self) -> String {
        unsafe {
            let err = klyron_jsc_get_exception_message(self.ptr);
            if err.is_null() { "Unknown error".into() }
            else { CStr::from_ptr(err).to_string_lossy().into() }
        }
    }

    fn check(result: JSCStringResult) -> Result<String, String> {
        if result.success {
            let s = if result.data.is_null() {
                String::new()
            } else {
                let s = unsafe { CStr::from_ptr(result.data).to_string_lossy().into() };
                unsafe { klyron_jsc_free_string(result.data) };
                s
            };
            Ok(s)
        } else {
            let err = unsafe { CStr::from_ptr(result.error.as_ptr()).to_string_lossy().into() };
            Err(err)
        }
    }

    fn check_void(result: JSCResult) -> Result<(), String> {
        if result.success { Ok(()) }
        else { Err(unsafe { CStr::from_ptr(result.error.as_ptr()).to_string_lossy().into() }) }
    }

    #[inline]
    fn to_cstring(s: &str) -> Result<CString, String> {
        CString::new(s).map_err(|e| e.to_string())
    }

    pub fn engine_handle(&self) -> *mut JSCEngineHandle { self.ptr }

    /* ─── eval / script ─────────────────────────────────────── */

    pub fn eval(&self, code: &str) -> Result<String, String> {
        let c = Self::to_cstring(code)?;
        let r = unsafe { klyron_jsc_eval(self.ptr, c.as_ptr(), std::ptr::null()) };
        Self::check(r)
    }

    pub fn compile(&self, source: &str, filename: Option<&str>) -> Result<*mut JSCScriptHandle, String> {
        let c_source = Self::to_cstring(source)?;
        let c_file = filename.map(|f| Self::to_cstring(f)).transpose()?;
        let ptr = unsafe {
            klyron_jsc_compile(self.ptr, c_source.as_ptr(), c_file.as_ref().map_or(std::ptr::null(), |f| f.as_ptr()))
        };
        if ptr.is_null() { Err("compile failed".into()) } else { Ok(ptr) }
    }

    pub fn run(&self, script: *mut JSCScriptHandle) -> Result<String, String> {
        let r = unsafe { klyron_jsc_run(self.ptr, script) };
        Self::check(r)
    }

    pub fn script_dispose(&self, script: *mut JSCScriptHandle) {
        unsafe { klyron_jsc_script_dispose(script) }
    }

    pub fn execute_script(&self, filename: &str, source: &str) -> Result<String, String> {
        let c_file = Self::to_cstring(filename)?;
        let c_src = Self::to_cstring(source)?;
        let r = unsafe { klyron_jsc_eval(self.ptr, c_src.as_ptr(), c_file.as_ptr()) };
        Self::check(r)
    }

    pub fn execute_module(&self, filename: &str, source: &str) -> Result<String, String> {
        let c_file = Self::to_cstring(filename)?;
        let c_src = Self::to_cstring(source)?;
        let mod_ptr = unsafe { klyron_jsc_module_compile(self.ptr, c_src.as_ptr(), c_file.as_ptr()) };
        if mod_ptr.is_null() { return Err("module compile failed".into()) }
        let r = unsafe { klyron_jsc_module_instantiate(self.ptr, mod_ptr) };
        Self::check_void(r)?;
        let r = unsafe { klyron_jsc_module_evaluate(self.ptr, mod_ptr) };
        let result = Self::check(r);
        unsafe { klyron_jsc_module_dispose(mod_ptr) };
        result
    }

    /* ─── call function ─────────────────────────────────────── */

    pub fn call_function(
        &self,
        func: *mut JSCValueHandle,
        this_obj: Option<*mut JSCValueHandle>,
        args: &[*mut JSCValueHandle],
    ) -> Result<*mut JSCValueHandle, String> {
        let mut raw_args: Vec<*mut JSCValueHandle> = args.to_vec();
        let this_ptr = this_obj.unwrap_or(std::ptr::null_mut());
        let ptr = unsafe {
            klyron_jsc_call_function(self.ptr, func, this_ptr, raw_args.len() as c_int, raw_args.as_mut_ptr())
        };
        if ptr.is_null() { Err(self.last_error()) } else { Ok(ptr) }
    }

    /* ─── object property access ────────────────────────────── */

    pub fn object_set_property(
        &self,
        object: *mut JSCValueHandle,
        name: &str,
        value: *mut JSCValueHandle,
    ) -> Result<(), String> {
        let c = Self::to_cstring(name)?;
        Self::check_void(unsafe { klyron_jsc_object_set_property(self.ptr, object, c.as_ptr(), value) })
    }

    pub fn object_get_property(
        &self,
        object: *mut JSCValueHandle,
        name: &str,
    ) -> Result<*mut JSCValueHandle, String> {
        let c = Self::to_cstring(name)?;
        let ptr = unsafe { klyron_jsc_object_get_property(self.ptr, object, c.as_ptr()) };
        if ptr.is_null() { Err("object get property failed".into()) } else { Ok(ptr) }
    }

    /* ─── JSON ──────────────────────────────────────────────── */

    pub fn json_stringify(&self, value: *mut JSCValueHandle) -> Result<String, String> {
        Self::check(unsafe { klyron_jsc_json_stringify(self.ptr, value) })
    }

    pub fn json_parse(&self, json: &str) -> Result<*mut JSCValueHandle, String> {
        let c = Self::to_cstring(json)?;
        let ptr = unsafe { klyron_jsc_json_parse(self.ptr, c.as_ptr()) };
        if ptr.is_null() { Err("JSON parse failed".into()) } else { Ok(ptr) }
    }

    /* ─── global ────────────────────────────────────────────── */

    pub fn set_global(&self, name: &str, value: *mut JSCValueHandle) -> Result<(), String> {
        let c = Self::to_cstring(name)?;
        Self::check_void(unsafe { klyron_jsc_set_global(self.ptr, c.as_ptr(), value) })
    }

    pub fn get_global(&self, name: &str) -> Result<*mut JSCValueHandle, String> {
        let c = Self::to_cstring(name)?;
        let ptr = unsafe { klyron_jsc_get_global(self.ptr, c.as_ptr()) };
        if ptr.is_null() { Err("global get failed".into()) } else { Ok(ptr) }
    }

    /* ─── value creation ────────────────────────────────────── */

    pub fn value_new_string(&self, s: &str) -> Result<*mut JSCValueHandle, String> {
        let c = Self::to_cstring(s)?;
        let ptr = unsafe { klyron_jsc_value_new_string(self.ptr, c.as_ptr()) };
        if ptr.is_null() { Err("value_new_string failed".into()) } else { Ok(ptr) }
    }

    pub fn value_new_number(&self, n: f64) -> *mut JSCValueHandle {
        unsafe { klyron_jsc_value_new_number(self.ptr, n) }
    }

    pub fn value_new_bool(&self, v: bool) -> *mut JSCValueHandle {
        unsafe { klyron_jsc_value_new_bool(self.ptr, v) }
    }

    pub fn value_new_null(&self) -> *mut JSCValueHandle {
        unsafe { klyron_jsc_value_new_null(self.ptr) }
    }

    pub fn value_new_undefined(&self) -> *mut JSCValueHandle {
        unsafe { klyron_jsc_value_new_undefined(self.ptr) }
    }

    pub fn value_new_object(&self) -> *mut JSCValueHandle {
        unsafe { klyron_jsc_value_new_object(self.ptr) }
    }

    pub fn value_new_array(&self) -> *mut JSCValueHandle {
        unsafe { klyron_jsc_value_new_array(self.ptr) }
    }

    pub fn value_new_symbol(&self, description: Option<&str>) -> Result<*mut JSCValueHandle, String> {
        let c = description.map(|d| Self::to_cstring(d)).transpose()?;
        let ptr = unsafe {
            klyron_jsc_value_new_symbol(self.ptr, c.as_ref().map_or(std::ptr::null(), |s| s.as_ptr()))
        };
        if ptr.is_null() { Err("value_new_symbol failed".into()) } else { Ok(ptr) }
    }

    pub fn value_new_error(&self, message: &str) -> Result<*mut JSCValueHandle, String> {
        let c = Self::to_cstring(message)?;
        let ptr = unsafe { klyron_jsc_value_new_error(self.ptr, c.as_ptr()) };
        if ptr.is_null() { Err("value_new_error failed".into()) } else { Ok(ptr) }
    }

    pub fn value_typeof(&self, v: *mut JSCValueHandle) -> JSCTypeResult {
        unsafe { klyron_jsc_value_typeof(self.ptr, v) }
    }

    pub fn value_to_string(&self, v: *mut JSCValueHandle) -> Result<String, String> {
        Self::check(unsafe { klyron_jsc_value_to_string(self.ptr, v) })
    }

    pub fn value_to_number(&self, v: *mut JSCValueHandle) -> f64 {
        unsafe { klyron_jsc_value_to_number(self.ptr, v) }
    }

    pub fn value_to_bool(&self, v: *mut JSCValueHandle) -> bool {
        unsafe { klyron_jsc_value_to_bool(self.ptr, v) }
    }

    pub fn value_is_array(&self, v: *mut JSCValueHandle) -> bool {
        unsafe { klyron_jsc_value_is_array(self.ptr, v) }
    }

    pub fn value_is_function(&self, v: *mut JSCValueHandle) -> bool {
        unsafe { klyron_jsc_value_is_function(self.ptr, v) }
    }

    pub fn value_is_object(&self, v: *mut JSCValueHandle) -> bool {
        unsafe { klyron_jsc_value_is_object(self.ptr, v) }
    }

    pub fn value_is_error(&self, v: *mut JSCValueHandle) -> bool {
        unsafe { klyron_jsc_value_is_error(self.ptr, v) }
    }

    pub fn value_is_symbol(&self, v: *mut JSCValueHandle) -> bool {
        unsafe { klyron_jsc_value_is_symbol(self.ptr, v) }
    }

    pub fn value_is_promise(&self, v: *mut JSCValueHandle) -> bool {
        unsafe { klyron_jsc_value_is_promise(self.ptr, v) }
    }

    pub fn value_is_typed_array(&self, v: *mut JSCValueHandle) -> bool {
        unsafe { klyron_jsc_value_is_typed_array(self.ptr, v) }
    }

    pub fn value_is_null(&self, v: *mut JSCValueHandle) -> bool {
        unsafe { klyron_jsc_value_is_null(self.ptr, v) }
    }

    pub fn value_is_undefined(&self, v: *mut JSCValueHandle) -> bool {
        unsafe { klyron_jsc_value_is_undefined(self.ptr, v) }
    }

    pub fn value_is_boolean(&self, v: *mut JSCValueHandle) -> bool {
        unsafe { klyron_jsc_value_is_boolean(self.ptr, v) }
    }

    pub fn value_is_number(&self, v: *mut JSCValueHandle) -> bool {
        unsafe { klyron_jsc_value_is_number(self.ptr, v) }
    }

    pub fn value_is_string(&self, v: *mut JSCValueHandle) -> bool {
        unsafe { klyron_jsc_value_is_string(self.ptr, v) }
    }

    pub fn value_dispose(&self, v: *mut JSCValueHandle) {
        unsafe { klyron_jsc_value_dispose(v) }
    }

    /* ─── promise ───────────────────────────────────────────── */

    pub fn promise_new(&self) -> Result<*mut JSCValueHandle, String> {
        let ptr = unsafe { klyron_jsc_promise_new(self.ptr) };
        if ptr.is_null() { Err("promise creation failed".into()) } else { Ok(ptr) }
    }

    pub fn promise_resolve(&self, p: *mut JSCValueHandle, v: *mut JSCValueHandle) -> Result<(), String> {
        Self::check_void(unsafe { klyron_jsc_promise_resolve(self.ptr, p, v) })
    }

    pub fn promise_reject(&self, p: *mut JSCValueHandle, reason: &str) -> Result<(), String> {
        let c = Self::to_cstring(reason)?;
        Self::check_void(unsafe { klyron_jsc_promise_reject(self.ptr, p, c.as_ptr()) })
    }

    /* ─── module ────────────────────────────────────────────── */

    pub fn module_compile(&self, source: &str, origin: Option<&str>) -> Result<*mut JSCValueHandle, String> {
        let c_src = Self::to_cstring(source)?;
        let c_origin = origin.map(|o| Self::to_cstring(o)).transpose()?;
        let ptr = unsafe {
            klyron_jsc_module_compile(self.ptr, c_src.as_ptr(), c_origin.as_ref().map_or(std::ptr::null(), |o| o.as_ptr()))
        };
        if ptr.is_null() { Err("module compile failed".into()) } else { Ok(ptr) }
    }

    pub fn module_instantiate(&self, m: *mut JSCValueHandle) -> Result<(), String> {
        Self::check_void(unsafe { klyron_jsc_module_instantiate(self.ptr, m) })
    }

    pub fn module_evaluate(&self, m: *mut JSCValueHandle) -> Result<String, String> {
        Self::check(unsafe { klyron_jsc_module_evaluate(self.ptr, m) })
    }

    pub fn module_dispose(&self, m: *mut JSCValueHandle) {
        unsafe { klyron_jsc_module_dispose(m) }
    }

    /* ─── heap / memory / GC ────────────────────────────────── */

    pub fn get_heap_stats(&self) -> Result<JSCHeapStats, String> {
        let mut stats = JSCHeapStats::zeroed();
        let r = unsafe { klyron_jsc_get_heap_stats(self.ptr, &mut stats) };
        if r.success { Ok(stats) } else { Err("get_heap_stats failed".into()) }
    }

    pub fn request_gc(&self) {
        unsafe { klyron_jsc_request_gc(self.ptr) }
    }

    pub fn low_memory_notification(&self) {
        unsafe { klyron_jsc_low_memory_notification(self.ptr) }
    }

    /* ─── array buffer / typed array ────────────────────────── */

    pub fn array_buffer_new(&self, data: &[u8]) -> Result<*mut JSCValueHandle, String> {
        let ptr = unsafe { klyron_jsc_array_buffer_new(self.ptr, data.as_ptr(), data.len()) };
        if ptr.is_null() { Err("array_buffer_new failed".into()) } else { Ok(ptr) }
    }

    pub fn array_buffer_get_data(&self, v: *mut JSCValueHandle) -> Result<Vec<u8>, String> {
        let mut len: usize = 0;
        let data = unsafe { klyron_jsc_array_buffer_get_data(self.ptr, v, &mut len) };
        if data.is_null() { Err("array_buffer_get_data failed".into()) }
        else {
            let buf = unsafe { std::slice::from_raw_parts(data, len) }.to_vec();
            unsafe { klyron_jsc_free_buffer(data) };
            Ok(buf)
        }
    }

    pub fn typed_array_new(&self, type_name: &str, length: usize) -> Result<*mut JSCValueHandle, String> {
        let c = Self::to_cstring(type_name)?;
        let ptr = unsafe { klyron_jsc_typed_array_new(self.ptr, c.as_ptr(), length) };
        if ptr.is_null() { Err("typed_array_new failed".into()) } else { Ok(ptr) }
    }

    pub fn typed_array_get_length(&self, v: *mut JSCValueHandle) -> usize {
        unsafe { klyron_jsc_typed_array_get_length(self.ptr, v) }
    }

    pub fn typed_array_get_buffer(&self, v: *mut JSCValueHandle) -> Result<*mut JSCValueHandle, String> {
        let ptr = unsafe { klyron_jsc_typed_array_get_buffer(self.ptr, v) };
        if ptr.is_null() { Err("typed_array_get_buffer failed".into()) } else { Ok(ptr) }
    }

    /* ─── microtasks ────────────────────────────────────────── */

    pub fn microtasks_perform_check(&self) {
        unsafe { klyron_jsc_microtasks_perform_check(self.ptr) }
    }

    /* ─── error / stack ─────────────────────────────────────── */

    pub fn get_exception_message(&self) -> String {
        unsafe {
            let p = klyron_jsc_get_exception_message(self.ptr);
            if p.is_null() { String::new() }
            else { CStr::from_ptr(p).to_string_lossy().into() }
        }
    }

    pub fn get_stack_trace(&self) -> Result<String, String> {
        Self::check(unsafe { klyron_jsc_get_stack_trace(self.ptr) })
    }

    /* ─── version ───────────────────────────────────────────── */

    pub fn version() -> String {
        unsafe {
            let p = klyron_jsc_version();
            if p.is_null() { String::new() }
            else { CStr::from_ptr(p).to_string_lossy().into() }
        }
    }
}

impl Drop for JSCEnginePtr {
    fn drop(&mut self) {
        unsafe { klyron_jsc_shutdown(self.ptr) }
    }
}

unsafe impl Send for JSCEnginePtr {}
unsafe impl Sync for JSCEnginePtr {}
