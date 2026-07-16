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

pub use runtime::JSCRuntime;
pub use isolate::JSCIsolate;
pub use error::JSCError;
pub use value::JSCValue;
pub use binding::{JSCEngine as JSCBindingEngine, JSCContext};

pub struct JSCEngine {
    pub runtime: JSCRuntime,
    pub binding: JSCBindingEngine,
}

impl JSCEngine {
    pub fn new() -> Result<Self, JSCError> {
        Ok(Self {
            runtime: JSCRuntime::new(),
            binding: JSCBindingEngine::new()?,
        })
    }

    pub fn eval(&self, code: &str) -> Result<String, JSCError> {
        self.runtime.eval(code)
    }

    pub fn execute_script(&self, filename: &str, source: &str) -> Result<String, JSCError> {
        self.runtime.execute_script(filename, source)
    }

    pub fn execute_module(&self, filename: &str, source: &str) -> Result<String, JSCError> {
        self.runtime.execute_module(filename, source)
    }

    pub fn snapshot(&self) -> Result<snapshot::JSCSnapshot, JSCError> {
        snapshot::JSCSnapshot::create(&self.runtime)
    }

    pub fn binding_eval(&self, code: &str) -> Result<String, JSCError> {
        self.binding.eval(code)
    }

    pub fn call_function(&self, name: &str, args: &[&str]) -> Result<String, JSCError> {
        self.binding.call_function(name, args)
    }

    pub fn get_global(&self, key: &str) -> Result<Option<String>, JSCError> {
        self.binding.get_global(key)
    }

    pub fn set_global(&self, key: &str, value: &str) -> Result<(), JSCError> {
        self.binding.set_global(key, value)
    }

    pub fn garbage_collect(&self) {
        self.binding.garbage_collect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jsc_engine_new() {
        match JSCEngine::new() {
            Ok(engine) => {
                let result = engine.eval("1 + 1");
                assert!(result.is_ok() || result.is_err());
            }
            Err(_) => {} // jsc may not be available
        }
    }

    #[test]
    fn test_jsc_eval_string() {
        if let Ok(engine) = JSCEngine::new() {
            let result = engine.eval("\"hello\" + \" world\"");
            match result {
                Ok(val) => assert!(val.contains("hello") || val.contains("\"hello world\"")),
                Err(_) => {}
            }
        }
    }

    #[test]
    fn test_jsc_eval_number() {
        if let Ok(engine) = JSCEngine::new() {
            let result = engine.eval("42");
            assert!(result.is_ok() || result.is_err());
        }
    }

    #[test]
    fn test_jsc_eval_syntax_error() {
        if let Ok(engine) = JSCEngine::new() {
            let _ = engine.eval("\n@");
        }
    }

    #[test]
    fn test_jsc_eval_boolean() {
        if let Ok(engine) = JSCEngine::new() {
            let result = engine.eval("true && false");
            match result {
                Ok(val) => assert!(!val.is_empty()),
                Err(_) => {}
            }
        }
    }

    #[test]
    fn test_jsc_execute_script() {
        if let Ok(engine) = JSCEngine::new() {
            let result = engine.execute_script("test.js", "1 + 2");
            match result {
                Ok(val) => assert!(!val.is_empty()),
                Err(_) => {}
            }
        }
    }

    #[test]
    fn test_jsc_execute_module() {
        if let Ok(engine) = JSCEngine::new() {
            let result = engine.execute_module("test.mjs", "export const x = 1;");
            match result {
                Ok(val) => assert!(!val.is_empty()),
                Err(_) => {}
            }
        }
    }

    #[test]
    fn test_jsc_snapshot() {
        if let Ok(engine) = JSCEngine::new() {
            let result = engine.snapshot();
            match result {
                Ok(_snap) => {}
                Err(_) => {}
            }
        }
    }

    #[test]
    fn test_jsc_array() {
        if let Ok(engine) = JSCEngine::new() {
            let result = engine.eval("[1, 2, 3].length");
            match result {
                Ok(val) => assert!(!val.is_empty()),
                Err(_) => {}
            }
        }
    }

    #[test]
    fn test_jsc_math() {
        if let Ok(engine) = JSCEngine::new() {
            let result = engine.eval("Math.max(1, 2, 3)");
            match result {
                Ok(val) => assert!(!val.is_empty()),
                Err(_) => {}
            }
        }
    }
}
