//! Boa runtime implementation

use crate::error::boaError;

pub struct BoaRuntime {
    pub isolate: crate::isolate::BoaIsolate,
    initialized: bool,
}

impl BoaRuntime {
    pub fn new() -> Self {
        Self {
            isolate: crate::isolate::BoaIsolate::new(),
            initialized: false,
        }
    }

    pub fn init(&mut self) -> Result<(), boaError> {
        self.initialized = true;
        Ok(())
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    pub fn execute(&self, _code: &str) -> Result<String, boaError> {
        if !self.initialized {
            return Err(boaError::NotInitialized);
        }
        Ok(String::new())
    }
}

impl Default for BoaRuntime {
    fn default() -> Self {
        Self::new()
    }
}
