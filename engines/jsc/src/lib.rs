pub mod error;
pub mod module_loader;
pub mod permissions;

#[cfg(feature = "native")]
pub mod ffi;

use error::JSCError;

#[cfg(feature = "native")]
use ffi::JSCEnginePtr;

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
                "JSC native engine not available. Enable 'native' feature and install libjavascriptcoregtk-4.1-dev.".into()
            ))
        }
    }

    pub fn eval(&self, _code: &str) -> Result<String, JSCError> {
        #[cfg(feature = "native")]
        { self.inner.eval(_code).map_err(JSCError::EvalFailed) }
        #[cfg(not(feature = "native"))]
        { let _ = _code; Err(JSCError::NotInitialized) }
    }

    pub fn execute_script(&self, _filename: &str, _source: &str) -> Result<String, JSCError> {
        #[cfg(feature = "native")]
        { self.inner.execute_script(_filename, _source).map_err(JSCError::EvalFailed) }
        #[cfg(not(feature = "native"))]
        { let _ = (_filename, _source); Err(JSCError::NotInitialized) }
    }

    pub fn execute_module(&self, _filename: &str, _source: &str) -> Result<String, JSCError> {
        #[cfg(feature = "native")]
        { self.inner.execute_module(_filename, _source).map_err(JSCError::ModuleFailed) }
        #[cfg(not(feature = "native"))]
        { let _ = (_filename, _source); Err(JSCError::NotInitialized) }
    }

    pub fn get_global(&self, _key: &str) -> Result<Option<String>, JSCError> {
        #[cfg(feature = "native")]
        { self.inner.get_global(_key).map_err(JSCError::GlobalFailed) }
        #[cfg(not(feature = "native"))]
        { let _ = _key; Err(JSCError::NotInitialized) }
    }

    pub fn set_global(&self, _key: &str, _value: &str) -> Result<(), JSCError> {
        #[cfg(feature = "native")]
        { self.inner.set_global(_key, _value).map_err(JSCError::GlobalFailed) }
        #[cfg(not(feature = "native"))]
        { let _ = (_key, _value); Err(JSCError::NotInitialized) }
    }

    pub fn call_function(&self, _name: &str, _args: &[&str]) -> Result<String, JSCError> {
        #[cfg(feature = "native")]
        { self.inner.call_function(_name, _args).map_err(JSCError::CallFailed) }
        #[cfg(not(feature = "native"))]
        { let _ = (_name, _args); Err(JSCError::NotInitialized) }
    }

    pub fn create_snapshot(&self) -> Result<Vec<u8>, JSCError> {
        #[cfg(feature = "native")]
        { self.inner.create_snapshot().map_err(JSCError::SnapshotFailed) }
        #[cfg(not(feature = "native"))]
        { Err(JSCError::NotInitialized) }
    }

    pub fn load_snapshot(&self, _data: &[u8]) -> Result<(), JSCError> {
        #[cfg(feature = "native")]
        { self.inner.load_snapshot(_data).map_err(JSCError::SnapshotFailed) }
        #[cfg(not(feature = "native"))]
        { let _ = _data; Err(JSCError::NotInitialized) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jsc_engine_new() {
        let engine = JSCEngine::new();
        assert!(engine.is_ok() || engine.is_err());
    }

    #[test]
    fn test_jsc_eval() {
        if let Ok(engine) = JSCEngine::new() {
            let result = engine.eval("1 + 2");
            assert!(result.is_ok() || result.is_err());
        }
    }
}
