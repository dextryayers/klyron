pub mod runtime;
pub mod isolate;
pub mod module_loader;
pub mod bindings;
pub mod value;
pub mod promise;
pub mod error;
pub mod snapshot;
pub mod permissions;

pub use runtime::V8Runtime;
pub use isolate::V8Isolate;
pub use error::V8Error;

pub struct V8Engine {
    pub runtime: V8Runtime,
}

impl V8Engine {
    pub fn new() -> Result<Self, V8Error> {
        Ok(Self {
            runtime: V8Runtime::new()?,
        })
    }

    pub fn eval(&self, code: &str) -> Result<String, V8Error> {
        self.runtime.eval(code)
    }

    pub fn execute_script(&self, filename: &str, source: &str) -> Result<String, V8Error> {
        self.runtime.execute_script(filename, source)
    }

    pub fn execute_module(&self, filename: &str, source: &str) -> Result<String, V8Error> {
        self.runtime.execute_module(filename, source)
    }

    pub fn snapshot(&self) -> Result<snapshot::V8Snapshot, V8Error> {
        snapshot::V8Snapshot::create(&self.runtime)
    }
}
