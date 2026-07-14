pub mod runtime;
pub mod isolate;
pub mod module_loader;
pub mod bindings;
pub mod value;
pub mod promise;
pub mod error;
pub mod snapshot;
pub mod permissions;

pub use runtime::JSCRuntime;
pub use isolate::JSCIsolate;
pub use error::JSCError;
pub use value::JSCValue;

pub struct JSCEngine {
    pub runtime: JSCRuntime,
}

impl JSCEngine {
    pub fn new() -> Result<Self, JSCError> {
        Ok(Self {
            runtime: JSCRuntime::new(),
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
}
