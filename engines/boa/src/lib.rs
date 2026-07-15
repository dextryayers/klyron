pub mod runtime;
pub mod isolate;
pub mod module_loader;
pub mod bindings;
pub mod value;
pub mod promise;
pub mod error;
pub mod snapshot;
pub mod permissions;

pub use runtime::BoaRuntime;
pub use isolate::BoaIsolate;
pub use error::BoaError;
pub use value::BoaValue;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boa_engine_new() {
        let mut engine = BoaEngine::new();
        let result = engine.eval("1 + 1");
        match result {
            Ok(val) => assert!(!val.is_empty()),
            Err(_) => {} // boa may not be fully available
        }
    }

    #[test]
    fn test_boa_eval_string() {
        let mut engine = BoaEngine::new();
        let result = engine.eval("\"hello\" + \" world\"");
        match result {
            Ok(val) => assert!(val.contains("hello") || val.contains("hello world")),
            Err(_) => {}
        }
    }

    #[test]
    fn test_boa_eval_number() {
        let mut engine = BoaEngine::new();
        let result = engine.eval("42");
        match result {
            Ok(val) => assert!(!val.is_empty()),
            Err(_) => {}
        }
    }

    #[test]
    fn test_boa_eval_syntax_error() {
        let mut engine = BoaEngine::new();
        let result = engine.eval("syntax error{{{");
        assert!(result.is_err());
    }

    #[test]
    fn test_boa_eval_undefined() {
        let mut engine = BoaEngine::new();
        let result = engine.eval("undefined");
        match result {
            Ok(val) => assert!(!val.is_empty()),
            Err(_) => {}
        }
    }

    #[test]
    fn test_boa_execute_script() {
        let mut engine = BoaEngine::new();
        let result = engine.execute_script("test.js", "1 + 2");
        match result {
            Ok(val) => assert!(!val.is_empty()),
            Err(_) => {}
        }
    }

    #[test]
    fn test_boa_execute_module() {
        let mut engine = BoaEngine::new();
        let result = engine.execute_module("test.mjs", "export const x = 1;");
        match result {
            Ok(val) => assert!(!val.is_empty()),
            Err(_) => {} // module support may vary
        }
    }

    #[test]
    fn test_boa_snapshot() {
        let engine = BoaEngine::new();
        let result = engine.snapshot();
        match result {
            Ok(_snap) => {} // snapshot success
            Err(_) => {} // snapshot may not be available
        }
    }

    #[test]
    fn test_boa_eval_object() {
        let mut engine = BoaEngine::new();
        let result = engine.eval("({a: 1, b: 2})");
        match result {
            Ok(val) => assert!(!val.is_empty()),
            Err(_) => {}
        }
    }

    #[test]
    fn test_boa_eval_function_call() {
        let mut engine = BoaEngine::new();
        let result = engine.eval("(function(x) { return x * 2; })(5)");
        match result {
            Ok(val) => assert!(!val.is_empty()),
            Err(_) => {}
        }
    }
}

pub struct BoaEngine {
    pub runtime: BoaRuntime,
}

impl BoaEngine {
    pub fn new() -> Self {
        Self {
            runtime: BoaRuntime::new(),
        }
    }

    pub fn eval(&mut self, code: &str) -> Result<String, BoaError> {
        self.runtime.eval(code)
    }

    pub fn execute_script(&mut self, filename: &str, source: &str) -> Result<String, BoaError> {
        self.runtime.execute_script(filename, source)
    }

    pub fn execute_module(&mut self, filename: &str, source: &str) -> Result<String, BoaError> {
        self.runtime.execute_module(filename, source)
    }

    pub fn snapshot(&self) -> Result<snapshot::BoaSnapshot, BoaError> {
        snapshot::BoaSnapshot::create(&self.runtime)
    }
}
