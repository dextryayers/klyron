//! Jsc runtime implementation

use crate::error::jscError;

pub struct JscRuntime {
    pub isolate: crate::isolate::JscIsolate,
    initialized: bool,
}

impl JscRuntime {
    pub fn new() -> Self {
        Self {
            isolate: crate::isolate::JscIsolate::new(),
            initialized: false,
        }
    }

    pub fn init(&mut self) -> Result<(), jscError> {
        self.initialized = true;
        Ok(())
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    pub fn execute(&self, _code: &str) -> Result<String, jscError> {
        if !self.initialized {
            return Err(jscError::NotInitialized);
        }
        Ok(String::new())
    }
}

impl Default for JscRuntime {
    fn default() -> Self {
        Self::new()
    }
}
