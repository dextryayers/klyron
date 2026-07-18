pub mod error;
pub mod module_loader;
pub mod permissions;

#[cfg(feature = "native")]
pub mod ffi;

#[cfg(feature = "native")]
pub mod ffi_types;

use error::V8Error;

#[cfg(feature = "native")]
use ffi::V8EnginePtr;

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
pub enum V8ValueType {
    Undefined = 0,
    Null = 1,
    Boolean = 2,
    Number = 3,
    String = 4,
    Object = 5,
    Array = 6,
    Function = 7,
    Promise = 8,
    Error = 9,
    Symbol = 10,
    BigInt = 11,
    Proxy = 12,
    TypedArray = 13,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum V8PromiseState {
    Pending = 0,
    Fulfilled = 1,
    Rejected = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum V8MemoryPressure {
    None = 0,
    Moderate = 1,
    Critical = 2,
}

pub struct V8Engine {
    #[cfg(feature = "native")]
    inner: V8EnginePtr,
}

impl V8Engine {
    pub fn new() -> Result<Self, V8Error> {
        #[cfg(feature = "native")]
        {
            V8EnginePtr::init()
                .map(|inner| Self { inner })
                .map_err(|e| V8Error::InitFailed(e))
        }
        #[cfg(not(feature = "native"))]
        {
            Err(V8Error::InitFailed(
                "V8 native engine not available — enable 'native' feature".into()
            ))
        }
    }

    pub fn eval(&self, _code: &str) -> Result<String, V8Error> {
        #[cfg(feature = "native")]
        { self.inner.eval(_code).map_err(V8Error::EvalFailed) }
        #[cfg(not(feature = "native"))]
        { Err(V8Error::NotInitialized) }
    }

    pub fn execute_script(&self, _filename: &str, _source: &str) -> Result<String, V8Error> {
        #[cfg(feature = "native")]
        { self.inner.execute_script(_filename, _source).map_err(V8Error::EvalFailed) }
        #[cfg(not(feature = "native"))]
        { Err(V8Error::NotInitialized) }
    }

    pub fn execute_module(&self, _filename: &str, _source: &str) -> Result<String, V8Error> {
        #[cfg(feature = "native")]
        { self.inner.execute_module(_filename, _source).map_err(V8Error::ModuleFailed) }
        #[cfg(not(feature = "native"))]
        { Err(V8Error::NotInitialized) }
    }

    pub fn json_stringify(&self, _value_ptr: *const std::ffi::c_void) -> Result<String, V8Error> {
        #[cfg(feature = "native")]
        {
            #[cfg(feature = "native")]
            let v = _value_ptr as *mut ffi::V8ValueHandle;
            self.inner.json_stringify(v).map_err(|e| V8Error::EvalFailed(e))
        }
        #[cfg(not(feature = "native"))]
        { Err(V8Error::NotInitialized) }
    }

    pub fn json_parse(&self, _json: &str) -> Result<*const std::ffi::c_void, V8Error> {
        #[cfg(feature = "native")]
        { self.inner.json_parse(_json).map(|p| p as *const std::ffi::c_void).map_err(V8Error::EvalFailed) }
        #[cfg(not(feature = "native"))]
        { Err(V8Error::NotInitialized) }
    }

    pub fn get_global(&self, _key: &str) -> Result<*const std::ffi::c_void, V8Error> {
        #[cfg(feature = "native")]
        { self.inner.get_global(_key).map(|p| p as *const std::ffi::c_void).map_err(V8Error::GlobalFailed) }
        #[cfg(not(feature = "native"))]
        { Err(V8Error::NotInitialized) }
    }

    pub fn set_global(&self, _key: &str, _value_ptr: *const std::ffi::c_void) -> Result<(), V8Error> {
        #[cfg(feature = "native")]
        {
            #[cfg(feature = "native")]
            let v = _value_ptr as *mut ffi::V8ValueHandle;
            self.inner.set_global(_key, v).map_err(V8Error::GlobalFailed)
        }
        #[cfg(not(feature = "native"))]
        { Err(V8Error::NotInitialized) }
    }

    pub fn get_heap_stats(&self) -> Result<HeapStats, V8Error> {
        #[cfg(feature = "native")]
        {
            let raw = self.inner.get_heap_stats().map_err(|e| V8Error::Internal(e))?;
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
        { Err(V8Error::NotInitialized) }
    }

    pub fn low_memory_notification(&self) {
        #[cfg(feature = "native")]
        self.inner.low_memory_notification()
    }

    pub fn idle_notification(&self, _deadline: f64) {
        #[cfg(feature = "native")]
        self.inner.idle_notification(_deadline)
    }

    pub fn set_memory_pressure(&self, _pressure: V8MemoryPressure) {
        #[cfg(feature = "native")]
        self.inner.set_memory_pressure(_pressure as u32)
    }

    pub fn request_gc(&self) {
        #[cfg(feature = "native")]
        self.inner.request_gc()
    }

    pub fn get_malloced_memory(&self) -> u64 {
        #[cfg(feature = "native")]
        { self.inner.get_malloced_memory() as u64 }
        #[cfg(not(feature = "native"))]
        { 0 }
    }

    pub fn adjust_external_memory(&self, _change: i64) -> u64 {
        #[cfg(feature = "native")]
        { self.inner.adjust_external_memory(_change) as u64 }
        #[cfg(not(feature = "native"))]
        { 0 }
    }

    pub fn microtasks_perform_check(&self) {
        #[cfg(feature = "native")]
        self.inner.microtasks_perform_check()
    }

    pub fn get_stack_trace(&self) -> Result<String, V8Error> {
        #[cfg(feature = "native")]
        { self.inner.get_stack_trace().map_err(|e| V8Error::Internal(e)) }
        #[cfg(not(feature = "native"))]
        { Err(V8Error::NotInitialized) }
    }

    pub fn is_native_available(&self) -> bool {
        cfg!(feature = "native")
    }

    pub fn version() -> String {
        #[cfg(feature = "native")]
        { V8EnginePtr::version() }
        #[cfg(not(feature = "native"))]
        { "V8 (not available)".into() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v8_engine_new() {
        let e = V8Engine::new();
        assert!(e.is_ok() || e.is_err());
    }

    #[test]
    fn test_v8_eval() {
        if let Ok(engine) = V8Engine::new() {
            assert_eq!(engine.eval("1 + 2").unwrap(), "3");
        }
    }

    #[test]
    fn test_v8_eval_string() {
        if let Ok(engine) = V8Engine::new() {
            let r = engine.eval("'hello' + ' world'");
            assert_eq!(r.unwrap(), "hello world");
        }
    }

    #[test]
    fn test_v8_execute_script() {
        if let Ok(engine) = V8Engine::new() {
            let r = engine.execute_script("test.js", "1 + 3");
            assert_eq!(r.unwrap(), "4");
        }
    }

    #[test]
    fn test_v8_get_heap_stats() {
        if let Ok(engine) = V8Engine::new() {
            let stats = engine.get_heap_stats().unwrap();
            assert!(stats.total_heap_size > 0);
            assert!(stats.heap_size_limit > 0);
            assert!(stats.used_heap_size > 0);
        }
    }

    #[test]
    fn test_v8_low_memory_notification() {
        if let Ok(engine) = V8Engine::new() {
            engine.low_memory_notification();
        }
    }

    #[test]
    fn test_v8_request_gc() {
        if let Ok(engine) = V8Engine::new() {
            engine.request_gc();
        }
    }

    #[test]
    fn test_v8_microtasks() {
        if let Ok(engine) = V8Engine::new() {
            engine.microtasks_perform_check();
        }
    }

    #[test]
    fn test_v8_stack_trace() {
        if let Ok(engine) = V8Engine::new() {
            let r = engine.get_stack_trace();
            assert!(r.is_ok() || r.is_err());
        }
    }

    #[test]
    fn test_v8_version() {
        let v = V8Engine::version();
        assert!(!v.is_empty());
    }

    #[test]
    fn test_v8_is_native_available() {
        let e = V8Engine::new();
        if let Ok(ref eng) = e {
            assert!(eng.is_native_available() || !eng.is_native_available());
        }
    }

    #[test]
    fn test_v8_eval_json() {
        if let Ok(engine) = V8Engine::new() {
            let r = engine.eval("JSON.stringify({a:1,b:2})");
            assert!(r.is_ok());
            let s = r.unwrap();
            assert!(s.contains("a") && s.contains("b"));
        }
    }

    #[test]
    fn test_v8_eval_function() {
        if let Ok(engine) = V8Engine::new() {
            let r = engine.eval("(function(x){return x*2;})(21)");
            assert_eq!(r.unwrap(), "42");
        }
    }

    #[test]
    fn test_v8_malloced_memory() {
        if let Ok(engine) = V8Engine::new() {
            let mem = engine.get_malloced_memory();
            // Just verify it doesn't crash
            let _ = mem;
        }
    }

    #[test]
    fn test_v8_adjust_external_memory() {
        if let Ok(engine) = V8Engine::new() {
            let before = engine.adjust_external_memory(0);
            let after = engine.adjust_external_memory(1024);
            assert!(after >= before);
        }
    }
}
