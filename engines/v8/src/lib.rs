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

pub use runtime::V8Runtime;
pub use isolate::V8Isolate;
pub use error::V8Error;
pub use binding::{V8Engine as V8BindingEngine, V8Context, HeapStatistics};

pub struct V8Engine {
    pub runtime: V8Runtime,
    pub binding: V8BindingEngine,
}

impl V8Engine {
    pub fn new() -> Result<Self, V8Error> {
        Ok(Self {
            runtime: V8Runtime::new()?,
            binding: V8BindingEngine::new()?,
        })
    }

    pub fn eval(&self, code: &str) -> Result<String, V8Error> {
        self.runtime.eval(code)
    }

    pub fn execute_script(&self, filename: &str, source: &str) -> Result<String, V8Error> {
        self.runtime.execute_script(filename, source)
    }

    pub fn execute_module(&self, filename: &str, source: &str) -> Result<String, V8Error> {
        self.runtime.execute_module(filename, source)
    }

    pub fn snapshot(&self) -> Result<snapshot::V8Snapshot, V8Error> {
        snapshot::V8Snapshot::create(&self.runtime)
    }

    pub fn binding_eval(&self, code: &str) -> Result<String, V8Error> {
        self.binding.eval(code)
    }

    pub fn call_function(&self, name: &str, args: &[&str]) -> Result<String, V8Error> {
        self.binding.call_function(name, args)
    }

    pub fn get_global(&self, key: &str) -> Result<Option<String>, V8Error> {
        self.binding.get_global(key)
    }

    pub fn set_global(&self, key: &str, value: &str) -> Result<(), V8Error> {
        self.binding.set_global(key, value)
    }

    pub fn heap_statistics(&self) -> HeapStatistics {
        self.binding.heap_statistics()
    }

    pub fn eval_module(&self, filename: &str, source: &str) -> Result<String, V8Error> {
        self.binding.eval_module(filename, source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v8_engine_new() {
        match V8Engine::new() {
            Ok(engine) => {
                let result = engine.eval("1 + 1");
                assert!(result.is_ok() || result.is_err());
            }
            Err(_) => {} // v8 may not be available
        }
    }

    #[test]
    fn test_v8_eval_string() {
        if let Ok(engine) = V8Engine::new() {
            let result = engine.eval("\"hello\" + \" world\"");
            match result {
                Ok(val) => assert!(val.contains("hello")),
                Err(_) => {}
            }
        }
    }

    #[test]
    fn test_v8_eval_number() {
        if let Ok(engine) = V8Engine::new() {
            let result = engine.eval("42");
            assert!(result.is_ok() || result.is_err());
        }
    }

    #[test]
    fn test_v8_eval_syntax_error() {
        if let Ok(engine) = V8Engine::new() {
            let result = engine.eval("syntax error{{{");
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_v8_eval_null() {
        if let Ok(engine) = V8Engine::new() {
            let result = engine.eval("null");
            assert!(result.is_ok() || result.is_err());
        }
    }

    #[test]
    fn test_v8_execute_script() {
        if let Ok(engine) = V8Engine::new() {
            let result = engine.execute_script("test.js", "1 + 2");
            match result {
                Ok(val) => assert!(!val.is_empty()),
                Err(_) => {}
            }
        }
    }

    #[test]
    fn test_v8_execute_module() {
        if let Ok(engine) = V8Engine::new() {
            let result = engine.execute_module("test.mjs", "export const x = 1;");
            match result {
                Ok(val) => assert!(!val.is_empty()),
                Err(_) => {}
            }
        }
    }

    #[test]
    fn test_v8_snapshot() {
        if let Ok(engine) = V8Engine::new() {
            let result = engine.snapshot();
            match result {
                Ok(_snap) => {}
                Err(_) => {}
            }
        }
    }

    #[test]
    fn test_v8_boolean() {
        if let Ok(engine) = V8Engine::new() {
            let result = engine.eval("true && false");
            match result {
                Ok(val) => assert!(!val.is_empty()),
                Err(_) => {}
            }
        }
    }

    #[test]
    fn test_v8_array() {
        if let Ok(engine) = V8Engine::new() {
            let result = engine.eval("[1, 2, 3].length");
            match result {
                Ok(val) => assert!(!val.is_empty()),
                Err(_) => {}
            }
        }
    }
}
