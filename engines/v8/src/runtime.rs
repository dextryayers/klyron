//! V8 runtime implementation

use crate::error::v8Error;

pub struct V8Runtime {
    pub isolate: crate::isolate::V8Isolate,
    initialized: bool,
}

impl V8Runtime {
    pub fn new() -> Self {
        Self {
            isolate: crate::isolate::V8Isolate::new(),
            initialized: false,
        }
    }

    pub fn init(&mut self) -> Result<(), v8Error> {
        self.initialized = true;
        Ok(())
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    pub fn execute(&self, _code: &str) -> Result<String, v8Error> {
        if !self.initialized {
            return Err(v8Error::NotInitialized);
        }
        Ok(String::new())
    }
}

impl Default for V8Runtime {
    fn default() -> Self {
        Self::new()
    }
}
