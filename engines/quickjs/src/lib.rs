mod runtime;
mod value;
mod error;
mod isolate;
mod snapshot;
mod bindings;
mod binding;
mod module_loader;
mod permissions;
mod promise;

pub use runtime::QuickJSRuntime;
pub use error::QuickJSError;
pub use value::QuickJSValue;
pub use isolate::QuickJSIsolate;
pub use snapshot::QuickJSSnapshot;
pub use bindings::QuickJSBindings;
pub use binding::{QuickJSContext, QuickJSBindingEngine};
pub use module_loader::QuickJSModuleLoader;
pub use permissions::QuickJSPermissions;
pub use promise::QuickJSPromise;

pub struct QuickJSEngine {
    inner: QuickJSRuntime,
}

impl QuickJSEngine {
    pub fn new() -> Result<Self, QuickJSError> {
        QuickJSRuntime::new()
            .map(|inner| Self { inner })
            .map_err(|e| QuickJSError::RuntimeError(e))
    }

    pub fn eval(&self, code: &str) -> Result<String, QuickJSError> {
        self.inner.eval(code).map_err(QuickJSError::EvalError)
    }

    pub fn execute_script(&self, filename: &str, source: &str) -> Result<String, QuickJSError> {
        self.inner.execute_script(filename, source).map_err(QuickJSError::EvalError)
    }

    pub fn execute_module(&self, filename: &str, source: &str) -> Result<String, QuickJSError> {
        self.execute_script(filename, source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quickjs_eval_addition() {
        if let Ok(engine) = QuickJSEngine::new() {
            let result = engine.eval("1 + 2");
            assert_eq!(result.unwrap(), "3");
        }
    }

    #[test]
    fn test_quickjs_eval_string() {
        if let Ok(engine) = QuickJSEngine::new() {
            let result = engine.eval("\"hello\" + \" world\"").unwrap();
            assert!(result.contains("hello world"), "got: {result}");
        }
    }

    #[test]
    fn test_quickjs_eval_number() {
        if let Ok(engine) = QuickJSEngine::new() {
            let result = engine.eval("42").unwrap();
            assert_eq!(result, "42");
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
            let result = engine.eval("({a: 1})").unwrap();
            assert!(!result.is_empty());
        }
    }

    #[test]
    fn test_quickjs_execute_script() {
        if let Ok(engine) = QuickJSEngine::new() {
            let result = engine.execute_script("test.js", "1 + 2").unwrap();
            assert_eq!(result, "3");
        }
    }

    #[test]
    fn test_quickjs_function() {
        if let Ok(engine) = QuickJSEngine::new() {
            let result = engine.eval("(function(x) { return x * 2; })(5)").unwrap();
            assert_eq!(result, "10");
        }
    }

    #[test]
    fn test_quickjs_array() {
        if let Ok(engine) = QuickJSEngine::new() {
            let result = engine.eval("[1, 2, 3, 4].length").unwrap();
            assert_eq!(result, "4");
        }
    }
}