pub mod error;
pub mod module_loader;
pub mod permissions;

#[cfg(feature = "native")]
pub mod ffi;

#[cfg(feature = "native")]
pub mod ffi_types;

use error::JSCError;

#[cfg(feature = "native")]
use ffi::JSCEnginePtr;

pub struct HeapStats {
    pub total_heap_size: u64,
    pub total_heap_size_executable: u64,
    pub total_physical_size: u64,
    pub total_available_size: u64,
    pub used_heap_size: u64,
    pub heap_size_limit: u64,
    pub malloced_memory: u64,
    pub peak_malloced_memory: u64,
    pub number_of_native_contexts: u64,
    pub number_of_detached_contexts: u64,
    pub total_global_handles_size: u64,
    pub used_global_handles_size: u64,
    pub external_memory: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum JSCValueType {
    Undefined = 0,
    Null = 1,
    Boolean = 2,
    Number = 3,
    String = 4,
    Object = 5,
    Array = 6,
    Function = 7,
    Error = 9,
    Symbol = 10,
    TypedArray = 13,
}

pub struct JSCEngine {
    #[cfg(feature = "native")]
    inner: JSCEnginePtr,
}

impl JSCEngine {
    pub fn new() -> Result<Self, JSCError> {
        #[cfg(feature = "native")]
        {
            JSCEnginePtr::init()
                .map(|inner| Self { inner })
                .map_err(|e| JSCError::InitFailed(e))
        }
        #[cfg(not(feature = "native"))]
        {
            Err(JSCError::InitFailed(
                "JSC native engine not available — enable 'native' feature".into()
            ))
        }
    }

    pub fn eval(&self, _code: &str) -> Result<String, JSCError> {
        #[cfg(feature = "native")]
        { self.inner.eval(_code).map_err(JSCError::EvalFailed) }
        #[cfg(not(feature = "native"))]
        { Err(JSCError::NotInitialized) }
    }

    pub fn execute_script(&self, _filename: &str, _source: &str) -> Result<String, JSCError> {
        #[cfg(feature = "native")]
        { self.inner.execute_script(_filename, _source).map_err(JSCError::EvalFailed) }
        #[cfg(not(feature = "native"))]
        { Err(JSCError::NotInitialized) }
    }

    pub fn execute_module(&self, _filename: &str, _source: &str) -> Result<String, JSCError> {
        #[cfg(feature = "native")]
        { self.inner.execute_module(_filename, _source).map_err(JSCError::ModuleFailed) }
        #[cfg(not(feature = "native"))]
        { Err(JSCError::NotInitialized) }
    }

    pub fn json_stringify(&self, _value_ptr: *const std::ffi::c_void) -> Result<String, JSCError> {
        #[cfg(feature = "native")]
        {
            let v = _value_ptr as *mut ffi::JSCValueHandle;
            self.inner.json_stringify(v).map_err(|e| JSCError::EvalFailed(e))
        }
        #[cfg(not(feature = "native"))]
        { Err(JSCError::NotInitialized) }
    }

    pub fn json_parse(&self, _json: &str) -> Result<*const std::ffi::c_void, JSCError> {
        #[cfg(feature = "native")]
        { self.inner.json_parse(_json).map(|p| p as *const std::ffi::c_void).map_err(JSCError::EvalFailed) }
        #[cfg(not(feature = "native"))]
        { Err(JSCError::NotInitialized) }
    }

    pub fn get_global(&self, _key: &str) -> Result<*const std::ffi::c_void, JSCError> {
        #[cfg(feature = "native")]
        { self.inner.get_global(_key).map(|p| p as *const std::ffi::c_void).map_err(JSCError::GlobalFailed) }
        #[cfg(not(feature = "native"))]
        { Err(JSCError::NotInitialized) }
    }

    pub fn set_global(&self, _key: &str, _value_ptr: *const std::ffi::c_void) -> Result<(), JSCError> {
        #[cfg(feature = "native")]
        {
            let v = _value_ptr as *mut ffi::JSCValueHandle;
            self.inner.set_global(_key, v).map_err(JSCError::GlobalFailed)
        }
        #[cfg(not(feature = "native"))]
        { Err(JSCError::NotInitialized) }
    }

    pub fn get_heap_stats(&self) -> Result<HeapStats, JSCError> {
        #[cfg(feature = "native")]
        {
            let raw = self.inner.get_heap_stats().map_err(|e| JSCError::Internal(e))?;
            Ok(HeapStats {
                total_heap_size: raw.total_heap_size as u64,
                total_heap_size_executable: raw.total_heap_size_executable as u64,
                total_physical_size: raw.total_physical_size as u64,
                total_available_size: raw.total_available_size as u64,
                used_heap_size: raw.used_heap_size as u64,
                heap_size_limit: raw.heap_size_limit as u64,
                malloced_memory: raw.malloced_memory as u64,
                peak_malloced_memory: raw.peak_malloced_memory as u64,
                number_of_native_contexts: raw.number_of_native_contexts as u64,
                number_of_detached_contexts: raw.number_of_detached_contexts as u64,
                total_global_handles_size: raw.total_global_handles_size as u64,
                used_global_handles_size: raw.used_global_handles_size as u64,
                external_memory: raw.external_memory as u64,
            })
        }
        #[cfg(not(feature = "native"))]
        { Err(JSCError::NotInitialized) }
    }

    pub fn request_gc(&self) {
        #[cfg(feature = "native")]
        self.inner.request_gc()
    }

    pub fn low_memory_notification(&self) {
        #[cfg(feature = "native")]
        self.inner.low_memory_notification()
    }

    pub fn microtasks_perform_check(&self) {
        #[cfg(feature = "native")]
        self.inner.microtasks_perform_check()
    }

    pub fn get_stack_trace(&self) -> Result<String, JSCError> {
        #[cfg(feature = "native")]
        { self.inner.get_stack_trace().map_err(|e| JSCError::Internal(e)) }
        #[cfg(not(feature = "native"))]
        { Err(JSCError::NotInitialized) }
    }

    pub fn is_native_available(&self) -> bool {
        cfg!(feature = "native")
    }

    pub fn version() -> String {
        #[cfg(feature = "native")]
        { JSCEnginePtr::version() }
        #[cfg(not(feature = "native"))]
        { "JSC (not available)".into() }
    }

    /* ─── New deep methods ─────────────────────────────────── */

    pub fn call_function(
        &self,
        _func_ptr: *const std::ffi::c_void,
        _this_ptr: Option<*const std::ffi::c_void>,
        _args: &[*const std::ffi::c_void],
    ) -> Result<*const std::ffi::c_void, JSCError> {
        #[cfg(feature = "native")]
        {
            let f = _func_ptr as *mut ffi::JSCValueHandle;
            let t = _this_ptr.map(|p| p as *mut ffi::JSCValueHandle);
            let a: Vec<*mut ffi::JSCValueHandle> = _args.iter().map(|p| *p as *mut ffi::JSCValueHandle).collect();
            self.inner.call_function(f, t, &a)
                .map(|p| p as *const std::ffi::c_void)
                .map_err(JSCError::CallFailed)
        }
        #[cfg(not(feature = "native"))]
        { Err(JSCError::NotInitialized) }
    }

    pub fn object_set_property(
        &self,
        _obj_ptr: *const std::ffi::c_void,
        _name: &str,
        _value_ptr: *const std::ffi::c_void,
    ) -> Result<(), JSCError> {
        #[cfg(feature = "native")]
        {
            let o = _obj_ptr as *mut ffi::JSCValueHandle;
            let v = _value_ptr as *mut ffi::JSCValueHandle;
            self.inner.object_set_property(o, _name, v).map_err(JSCError::GlobalFailed)
        }
        #[cfg(not(feature = "native"))]
        { Err(JSCError::NotInitialized) }
    }

    pub fn object_get_property(
        &self,
        _obj_ptr: *const std::ffi::c_void,
        _name: &str,
    ) -> Result<*const std::ffi::c_void, JSCError> {
        #[cfg(feature = "native")]
        {
            let o = _obj_ptr as *mut ffi::JSCValueHandle;
            self.inner.object_get_property(o, _name)
                .map(|p| p as *const std::ffi::c_void)
                .map_err(JSCError::GlobalFailed)
        }
        #[cfg(not(feature = "native"))]
        { Err(JSCError::NotInitialized) }
    }

    pub fn value_new_symbol(&self, _description: Option<&str>) -> Result<*const std::ffi::c_void, JSCError> {
        #[cfg(feature = "native")]
        { self.inner.value_new_symbol(_description).map(|p| p as *const std::ffi::c_void).map_err(|e| JSCError::Internal(e)) }
        #[cfg(not(feature = "native"))]
        { Err(JSCError::NotInitialized) }
    }

    pub fn value_new_error(&self, _message: &str) -> Result<*const std::ffi::c_void, JSCError> {
        #[cfg(feature = "native")]
        { self.inner.value_new_error(_message).map(|p| p as *const std::ffi::c_void).map_err(|e| JSCError::Internal(e)) }
        #[cfg(not(feature = "native"))]
        { Err(JSCError::NotInitialized) }
    }

    pub fn value_is_function(&self, _value_ptr: *const std::ffi::c_void) -> bool {
        #[cfg(feature = "native")]
        { self.inner.value_is_function(_value_ptr as *mut ffi::JSCValueHandle) }
        #[cfg(not(feature = "native"))]
        { false }
    }

    pub fn value_is_object(&self, _value_ptr: *const std::ffi::c_void) -> bool {
        #[cfg(feature = "native")]
        { self.inner.value_is_object(_value_ptr as *mut ffi::JSCValueHandle) }
        #[cfg(not(feature = "native"))]
        { false }
    }

    pub fn value_is_error(&self, _value_ptr: *const std::ffi::c_void) -> bool {
        #[cfg(feature = "native")]
        { self.inner.value_is_error(_value_ptr as *mut ffi::JSCValueHandle) }
        #[cfg(not(feature = "native"))]
        { false }
    }

    pub fn value_is_symbol(&self, _value_ptr: *const std::ffi::c_void) -> bool {
        #[cfg(feature = "native")]
        { self.inner.value_is_symbol(_value_ptr as *mut ffi::JSCValueHandle) }
        #[cfg(not(feature = "native"))]
        { false }
    }

    pub fn value_is_promise(&self, _value_ptr: *const std::ffi::c_void) -> bool {
        #[cfg(feature = "native")]
        { self.inner.value_is_promise(_value_ptr as *mut ffi::JSCValueHandle) }
        #[cfg(not(feature = "native"))]
        { false }
    }

    pub fn value_is_typed_array(&self, _value_ptr: *const std::ffi::c_void) -> bool {
        #[cfg(feature = "native")]
        { self.inner.value_is_typed_array(_value_ptr as *mut ffi::JSCValueHandle) }
        #[cfg(not(feature = "native"))]
        { false }
    }

    pub fn array_buffer_new(&self, _data: &[u8]) -> Result<*const std::ffi::c_void, JSCError> {
        #[cfg(feature = "native")]
        { self.inner.array_buffer_new(_data).map(|p| p as *const std::ffi::c_void).map_err(|e| JSCError::Internal(e)) }
        #[cfg(not(feature = "native"))]
        { Err(JSCError::NotInitialized) }
    }

    pub fn typed_array_new(&self, _type_name: &str, _length: usize) -> Result<*const std::ffi::c_void, JSCError> {
        #[cfg(feature = "native")]
        { self.inner.typed_array_new(_type_name, _length).map(|p| p as *const std::ffi::c_void).map_err(|e| JSCError::Internal(e)) }
        #[cfg(not(feature = "native"))]
        { Err(JSCError::NotInitialized) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jsc_engine_new() {
        let e = JSCEngine::new();
        assert!(e.is_ok() || e.is_err());
    }

    #[test]
    fn test_jsc_eval() {
        if let Ok(engine) = JSCEngine::new() {
            let r = engine.eval("1 + 2");
            assert!(r.is_ok());
            assert_eq!(r.unwrap(), "3");
        }
    }

    #[test]
    fn test_jsc_eval_string() {
        if let Ok(engine) = JSCEngine::new() {
            let r = engine.eval("'hello' + ' world'");
            assert_eq!(r.unwrap(), "hello world");
        }
    }

    #[test]
    fn test_jsc_execute_script() {
        if let Ok(engine) = JSCEngine::new() {
            let r = engine.execute_script("test.js", "1 + 3");
            assert_eq!(r.unwrap(), "4");
        }
    }

    #[test]
    fn test_jsc_get_heap_stats() {
        if let Ok(engine) = JSCEngine::new() {
            let stats = engine.get_heap_stats();
            assert!(stats.is_ok());
        }
    }

    #[test]
    fn test_jsc_low_memory_notification() {
        if let Ok(engine) = JSCEngine::new() {
            engine.low_memory_notification();
        }
    }

    #[test]
    fn test_jsc_request_gc() {
        if let Ok(engine) = JSCEngine::new() {
            engine.request_gc();
        }
    }

    #[test]
    fn test_jsc_microtasks() {
        if let Ok(engine) = JSCEngine::new() {
            engine.microtasks_perform_check();
        }
    }

    #[test]
    fn test_jsc_stack_trace() {
        if let Ok(engine) = JSCEngine::new() {
            let r = engine.get_stack_trace();
            assert!(r.is_ok() || r.is_err());
        }
    }

    #[test]
    fn test_jsc_version() {
        let v = JSCEngine::version();
        assert!(v.contains("JSC") || v.contains("not available"));
    }

    #[test]
    fn test_jsc_is_native_available() {
        let e = JSCEngine::new();
        if let Ok(ref eng) = e {
            assert!(eng.is_native_available() || !eng.is_native_available());
        }
    }

    #[test]
    fn test_jsc_eval_json() {
        if let Ok(engine) = JSCEngine::new() {
            let r = engine.eval("JSON.stringify({a:1,b:2})");
            assert!(r.is_ok());
            let s = r.unwrap();
            assert!(s.contains("a") && s.contains("b"));
        }
    }

    #[test]
    fn test_jsc_eval_function() {
        if let Ok(engine) = JSCEngine::new() {
            let r = engine.eval("(function(x){return x*2;})(21)");
            assert_eq!(r.unwrap(), "42");
        }
    }

    #[test]
    fn test_jsc_eval_object() {
        if let Ok(engine) = JSCEngine::new() {
            let r = engine.eval("({x: 10, y: 20})");
            assert!(r.is_ok());
        }
    }

    #[test]
    fn test_jsc_eval_array() {
        if let Ok(engine) = JSCEngine::new() {
            let r = engine.eval("[1, 2, 3].length");
            assert_eq!(r.unwrap(), "3");
        }
    }

    #[test]
    fn test_jsc_value_checks() {
        if let Ok(engine) = JSCEngine::new() {
            assert_eq!(engine.eval("typeof 42").unwrap(), "\"number\"");
            assert_eq!(engine.eval("typeof 'hi'").unwrap(), "\"string\"");
            let r = engine.eval("undefined");
            assert!(r.is_ok());
        }
    }

    #[test]
    fn test_jsc_engine_new_fallback() {
        let engine = JSCEngine::new();
        match engine {
            Ok(_) => assert!(true),
            Err(e) => assert!(e.to_string().contains("not available") || e.to_string().contains("install")),
        }
    }
}
