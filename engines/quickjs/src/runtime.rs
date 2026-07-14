//! Quickjs runtime implementation

use crate::error::quickjsError;

pub struct QuickjsRuntime {
    pub isolate: crate::isolate::QuickjsIsolate,
    initialized: bool,
}

impl QuickjsRuntime {
    pub fn new() -> Self {
        Self {
            isolate: crate::isolate::QuickjsIsolate::new(),
            initialized: false,
        }
    }

    pub fn init(&mut self) -> Result<(), quickjsError> {
        self.initialized = true;
        Ok(())
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    pub fn execute(&self, _code: &str) -> Result<String, quickjsError> {
        if !self.initialized {
            return Err(quickjsError::NotInitialized);
        }
        Ok(String::new())
    }
}

impl Default for QuickjsRuntime {
    fn default() -> Self {
        Self::new()
    }
}
