pub mod runtime;
pub mod isolate;
pub mod module_loader;
pub mod bindings;
pub mod value;
pub mod promise;
pub mod error;
pub mod snapshot;
pub mod permissions;

pub use runtime::QuickJSRuntime;
pub use isolate::QuickJSIsolate;
pub use error::QuickJSError;
pub use value::QuickJSValue;

pub struct QuickJSEngine {
    pub runtime: QuickJSRuntime,
}

impl QuickJSEngine {
    pub fn new() -> Result<Self, QuickJSError> {
        Ok(Self {
            runtime: QuickJSRuntime::new()?,
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
}
