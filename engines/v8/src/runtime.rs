use crate::error::V8Error;
use crate::isolate::V8Isolate;

pub struct V8Runtime {
    isolate: V8Isolate,
}

impl V8Runtime {
    pub fn new() -> Result<Self, V8Error> {
        let isolate = V8Isolate::create_isolate()?;
        Ok(Self { isolate })
    }

    pub fn eval(&self, code: &str) -> Result<String, V8Error> {
        if code.trim().is_empty() {
            return Ok(String::new());
        }
        self.isolate.with_scope(|_scope| {
            Err(V8Error::ExecutionFailed("V8 native engine not linked; enable the 'v8_native' feature".into()))
        })
    }

    pub fn execute_script(&self, _filename: &str, source: &str) -> Result<String, V8Error> {
        self.eval(source)
    }

    pub fn execute_module(&self, _filename: &str, source: &str) -> Result<String, V8Error> {
        self.eval(source)
    }
}
