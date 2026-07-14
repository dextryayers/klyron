use crate::bindings::QuickJSBindings;
use crate::error::QuickJSError;
use crate::isolate::QuickJSIsolate;
use crate::module_loader::QuickJSModuleLoader;
use crate::value::QuickJSValue;

pub struct QuickJSRuntime {
    pub isolate: QuickJSIsolate,
    pub module_loader: QuickJSModuleLoader,
    pub bindings: QuickJSBindings,
}

unsafe impl Send for QuickJSRuntime {}
unsafe impl Sync for QuickJSRuntime {}

impl QuickJSRuntime {
    pub fn new() -> Result<Self, QuickJSError> {
        let mut isolate = QuickJSIsolate::create_isolate()?;
        isolate.create_context()?;
        let module_loader = QuickJSModuleLoader::new(".");
        let bindings = QuickJSBindings::new();
        bindings.register_bindings(std::ptr::null_mut())?;
        Ok(Self { isolate, module_loader, bindings })
    }

    pub fn eval(&self, code: &str) -> Result<String, QuickJSError> {
        if code.trim().is_empty() {
            return Ok(String::new());
        }
        self.module_loader.register("eval", code);
        self.isolate.with_ctx(|_ctx| {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(code) {
                let val = QuickJSValue::from_json(&json);
                return Ok(val.to_json().to_string());
            }
            Ok(code.to_string())
        })
    }

    pub fn execute_script(&self, filename: &str, source: &str) -> Result<String, QuickJSError> {
        self.module_loader.register(filename, source);
        self.eval(source)
    }

    pub fn execute_module(&self, filename: &str, source: &str) -> Result<String, QuickJSError> {
        self.module_loader.instantiate(filename, source)?;
        self.eval(source)
    }

    pub fn ctx(&self) -> *mut std::ffi::c_void {
        std::ptr::null_mut()
    }

    pub fn rt(&self) -> *mut std::ffi::c_void {
        std::ptr::null_mut()
    }
}

impl Drop for QuickJSRuntime {
    fn drop(&mut self) {
    }
}
