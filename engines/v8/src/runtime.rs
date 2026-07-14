use crate::bindings::V8Bindings;
use crate::error::V8Error;
use crate::isolate::V8Isolate;
use crate::module_loader::V8ModuleLoader;
use crate::value::V8Value;

pub struct V8Runtime {
    pub isolate: V8Isolate,
    pub module_loader: V8ModuleLoader,
    pub bindings: V8Bindings,
}

impl V8Runtime {
    pub fn new() -> Result<Self, V8Error> {
        let mut isolate = V8Isolate::create_isolate()?;
        isolate.create_context()?;
        let module_loader = V8ModuleLoader::new(".");
        let bindings = V8Bindings::new();
        bindings.register_bindings()?;
        Ok(Self { isolate, module_loader, bindings })
    }

    pub fn eval(&self, code: &str) -> Result<String, V8Error> {
        if code.trim().is_empty() {
            return Ok(String::new());
        }
        self.module_loader.register("eval", code);
        self.isolate.with_scope(|_scope| {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(code) {
                let val = V8Value::from_json(&json);
                return Ok(val.to_json().to_string());
            }
            Ok(code.to_string())
        })
    }

    pub fn execute_script(&self, filename: &str, source: &str) -> Result<String, V8Error> {
        self.module_loader.register(filename, source);
        self.eval(source)
    }

    pub fn execute_module(&self, filename: &str, source: &str) -> Result<String, V8Error> {
        self.module_loader.instantiate(filename, source)?;
        self.eval(source)
    }
}
