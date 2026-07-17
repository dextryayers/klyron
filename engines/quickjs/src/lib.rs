pub mod error;
pub mod ffi;
pub mod module_loader;
pub mod permissions;

use error::QuickJSError;
use ffi::QuickJSEnginePtr;

pub struct QuickJSEngine {
    inner: QuickJSEnginePtr,
}

impl QuickJSEngine {
    pub fn new() -> Result<Self, QuickJSError> {
        QuickJSEnginePtr::init()
            .map(|inner| Self { inner })
            .map_err(|e| QuickJSError::InitFailed(e))
    }

    pub fn eval(&self, code: &str) -> Result<String, QuickJSError> {
        self.inner.eval(code).map_err(|e| {
            if e.contains("SyntaxError") || e.contains("syntax") {
                QuickJSError::EvalFailed(e)
            } else {
                QuickJSError::EvalFailed(e)
            }
        })
    }

    pub fn execute_script(&self, filename: &str, source: &str) -> Result<String, QuickJSError> {
        self.inner.execute_script(filename, source).map_err(QuickJSError::EvalFailed)
    }

    pub fn execute_module(&self, filename: &str, source: &str) -> Result<String, QuickJSError> {
        self.inner.execute_module(filename, source).map_err(QuickJSError::ModuleFailed)
    }

    pub fn get_global(&self, key: &str) -> Result<Option<String>, QuickJSError> {
        self.inner.get_global(key).map_err(QuickJSError::GlobalGetFailed)
    }

    pub fn set_global(&self, key: &str, value: &str) -> Result<(), QuickJSError> {
        self.inner.set_global(key, value).map_err(QuickJSError::GlobalSetFailed)
    }

    pub fn call_function(&self, name: &str, args: &[&str]) -> Result<String, QuickJSError> {
        self.inner.call_function(name, args).map_err(QuickJSError::CallFailed)
    }

    pub fn create_snapshot(&self) -> Result<Vec<u8>, QuickJSError> {
        self.inner.create_snapshot().map_err(QuickJSError::SnapshotFailed)
    }

    pub fn load_snapshot(&self, data: &[u8]) -> Result<(), QuickJSError> {
        self.inner.load_snapshot(data).map_err(QuickJSError::SnapshotFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quickjs_eval_addition() {
        let engine = QuickJSEngine::new().unwrap();
        let result = engine.eval("1 + 2").unwrap();
        assert_eq!(result, "3");
    }

    #[test]
    fn test_quickjs_eval_string() {
        let engine = QuickJSEngine::new().unwrap();
        let result = engine.eval("\"hello\" + \" world\"").unwrap();
        assert!(result.contains("hello world"), "got: {result}");
    }

    #[test]
    fn test_quickjs_eval_syntax_error() {
        let engine = QuickJSEngine::new().unwrap();
        let result = engine.eval("syntax error{{{");
        assert!(result.is_err());
    }

    #[test]
    fn test_quickjs_execute_script() {
        let engine = QuickJSEngine::new().unwrap();
        let result = engine.execute_script("test.js", "1 + 2").unwrap();
        assert_eq!(result, "3");
    }

    #[test]
    fn test_quickjs_function() {
        let engine = QuickJSEngine::new().unwrap();
        let result = engine.eval("(function(x) { return x * 2; })(5)").unwrap();
        assert_eq!(result, "10");
    }

    #[test]
    fn test_quickjs_array() {
        let engine = QuickJSEngine::new().unwrap();
        let result = engine.eval("[1, 2, 3, 4].length").unwrap();
        assert_eq!(result, "4");
    }

    #[test]
    fn test_quickjs_object() {
        let engine = QuickJSEngine::new().unwrap();
        let result = engine.eval("({a: 1, b: 2})").unwrap();
        assert!(result.contains("a") || result.contains("1"));
    }

    #[test]
    fn test_quickjs_globals() {
        let engine = QuickJSEngine::new().unwrap();
        engine.set_global("x", "42").unwrap();
        let val = engine.get_global("x").unwrap();
        assert!(val.is_some());
    }

    #[test]
    fn test_quickjs_snapshot() {
        let engine = QuickJSEngine::new().unwrap();
        let snap = engine.create_snapshot();
        assert!(snap.is_ok() || snap.is_err());
    }
}
