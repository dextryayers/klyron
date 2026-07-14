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
