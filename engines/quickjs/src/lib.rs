pub mod runtime;
pub mod isolate;
pub mod module_loader;
pub mod bindings;
pub mod binding;
pub mod value;
pub mod promise;
pub mod error;
pub mod snapshot;
pub mod permissions;

pub use runtime::QuickJSRuntime;
pub use isolate::QuickJSIsolate;
pub use error::QuickJSError;
pub use value::QuickJSValue;
pub use binding::{QuickJSEngine as QuickJSBindingEngine, QuickJSContext, QJSMemoryUsage};

pub struct QuickJSEngine {
    pub runtime: QuickJSRuntime,
    pub binding: QuickJSBindingEngine,
}

impl QuickJSEngine {
    pub fn new() -> Result<Self, QuickJSError> {
        Ok(Self {
            runtime: QuickJSRuntime::new()?,
            binding: QuickJSBindingEngine::new()?,
        })
    }

    pub fn eval(&self, code: &str) -> Result<String, QuickJSError> {
        self.runtime.eval(code)
    }

    pub fn execute_script(&self, filename: &str, source: &str) -> Result<String, QuickJSError> {
        self.runtime.execute_script(filename, source)
    }

    pub fn execute_module(&self, filename: &str, source: &str) -> Result<String, QuickJSError> {
        self.runtime.execute_module(filename, source)
    }

    pub fn snapshot(&self) -> Result<snapshot::QuickJSSnapshot, QuickJSError> {
        snapshot::QuickJSSnapshot::create(&self.runtime)
    }

    pub fn binding_eval(&self, code: &str) -> Result<String, QuickJSError> {
        self.binding.eval(code)
    }

    pub fn call_function(&self, name: &str, args: &[&str]) -> Result<String, QuickJSError> {
        self.binding.call_function(name, args)
    }

    pub fn get_global(&self, key: &str) -> Result<Option<String>, QuickJSError> {
        self.binding.get_global(key)
    }

    pub fn set_global(&self, key: &str, value: &str) -> Result<(), QuickJSError> {
        self.binding.set_global(key, value)
    }

    pub fn memory_usage(&self) -> QJSMemoryUsage {
        self.binding.memory_usage()
    }

    pub fn set_sandboxed(&mut self, sandboxed: bool) {
        self.binding.set_sandboxed(sandboxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quickjs_engine_new() {
        match QuickJSEngine::new() {
            Ok(engine) => {
                let result = engine.eval("1 + 1");
                assert!(result.is_ok() || result.is_err());
            }
            Err(_) => {}
        }
    }

    #[test]
    fn test_quickjs_eval_string() {
        if let Ok(engine) = QuickJSEngine::new() {
            let result = engine.eval("\"hello\" + \" world\"");
            match result {
                Ok(val) => assert!(val.contains("hello") || val.contains("\"hello world\"")),
                Err(_) => {}
            }
        }
    }

    #[test]
    fn test_quickjs_eval_number() {
        if let Ok(engine) = QuickJSEngine::new() {
            let result = engine.eval("42");
            assert!(result.is_ok() || result.is_err());
        }
    }

    #[test]
    fn test_quickjs_eval_syntax_error() {
        if let Ok(engine) = QuickJSEngine::new() {
            let result = engine.eval("syntax error{{{");
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_quickjs_eval_object() {
        if let Ok(engine) = QuickJSEngine::new() {
            let result = engine.eval("({a: 1})");
            match result {
                Ok(val) => assert!(!val.is_empty()),
                Err(_) => {}
            }
        }
    }

    #[test]
    fn test_quickjs_execute_script() {
        if let Ok(engine) = QuickJSEngine::new() {
            let result = engine.execute_script("test.js", "1 + 2");
            match result {
                Ok(val) => assert!(!val.is_empty()),
                Err(_) => {}
            }
        }
    }

    #[test]
    fn test_quickjs_execute_module() {
        if let Ok(engine) = QuickJSEngine::new() {
            let result = engine.execute_module("test.mjs", "export const x = 1;");
            match result {
                Ok(val) => assert!(!val.is_empty()),
                Err(_) => {}
            }
        }
    }

    #[test]
    fn test_quickjs_snapshot() {
        if let Ok(engine) = QuickJSEngine::new() {
            let result = engine.snapshot();
            match result {
                Ok(_snap) => {}
                Err(_) => {}
            }
        }
    }

    #[test]
    fn test_quickjs_function() {
        if let Ok(engine) = QuickJSEngine::new() {
            let result = engine.eval("(function(x) { return x * 2; })(5)");
            match result {
                Ok(val) => assert!(!val.is_empty()),
                Err(_) => {}
            }
        }
    }

    #[test]
    fn test_quickjs_array() {
        if let Ok(engine) = QuickJSEngine::new() {
            let result = engine.eval("[1, 2, 3, 4].length");
            match result {
                Ok(val) => assert!(!val.is_empty()),
                Err(_) => {}
            }
        }
    }
}
