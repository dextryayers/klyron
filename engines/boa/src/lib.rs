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

    fn new_engine() -> BoaEngine {
        BoaEngine::new()
    }

    #[test]
    fn test_boa_eval_addition() {
        let mut eng = new_engine();
        let result = eng.eval("1 + 2").unwrap();
        assert_eq!(result, "3");
    }

    #[test]
    fn test_boa_eval_string_concat() {
        let mut eng = new_engine();
        let result = eng.eval("\"hello\" + \" world\"").unwrap();
        assert!(result.contains("hello world"), "got: {result}");
    }

    #[test]
    fn test_boa_eval_syntax_error() {
        let mut eng = new_engine();
        let result = eng.eval("syntax error{{{");
        assert!(result.is_err());
    }

    #[test]
    fn test_boa_execute_script() {
        let mut eng = new_engine();
        let result = eng.execute_script("test.js", "1 + 2").unwrap();
        assert_eq!(result, "3");
    }

    #[test]
    fn test_boa_eval_function_call() {
        let mut eng = new_engine();
        let result = eng.eval("(function(x) { return x * 2; })(5)").unwrap();
        assert_eq!(result, "10");
    }

    #[test]
    fn test_boa_eval_array_length() {
        let mut eng = new_engine();
        let result = eng.eval("[1,2,3,4].length").unwrap();
        assert_eq!(result, "4");
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
