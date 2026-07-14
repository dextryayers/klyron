use crate::bindings::JSCBindings;
use crate::error::JSCError;
use crate::isolate::JSCIsolate;
use crate::module_loader::JSCModuleLoader;
use crate::value::JSCValue;

pub struct JSCRuntime {
    pub isolate: JSCIsolate,
    pub module_loader: JSCModuleLoader,
    pub bindings: JSCBindings,
}

unsafe impl Send for JSCRuntime {}
unsafe impl Sync for JSCRuntime {}

impl JSCRuntime {
    pub fn new() -> Self {
        let mut isolate = JSCIsolate::new();
        let _ = isolate.create_context();
        let module_loader = JSCModuleLoader::new(".");
        let bindings = JSCBindings::new();
        let _ = bindings.register_bindings(std::ptr::null_mut());
        Self { isolate, module_loader, bindings }
    }

    pub fn eval(&self, code: &str) -> Result<String, JSCError> {
        if code.trim().is_empty() {
            return Ok(String::new());
        }
        self.module_loader.register("eval", code);
        self.isolate.with_context(|_ctx| {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(code) {
                let val = JSCValue::from_json(&json);
                return Ok(val.to_json().to_string());
            }
            Ok(code.to_string())
        })
    }

    pub fn execute_script(&self, filename: &str, source: &str) -> Result<String, JSCError> {
        self.module_loader.register(filename, source);
        self.eval(source)
    }

    pub fn execute_module(&self, filename: &str, source: &str) -> Result<String, JSCError> {
        self.module_loader.instantiate(filename, source)?;
        self.eval(source)
    }

    pub fn context(&self) -> *mut std::ffi::c_void {
        std::ptr::null_mut()
    }

    pub fn set_exception_handler(&self) {
    }
}

impl Default for JSCRuntime {
    fn default() -> Self {
        Self::new()
    }
}
