//! Snapshot support for Boa

use crate::error::boaError;

pub struct BoaSnapshot {
    data: Vec<u8>,
    created_at: std::time::SystemTime,
}

impl BoaSnapshot {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, created_at: std::time::SystemTime::now() }
    }

    pub fn create(runtime: &crate::runtime::BoaRuntime) -> Result<Self, boaError> {
        if !runtime.is_initialized() {
            return Err(boaError::NotInitialized);
        }
        Ok(Self {
            data: vec![],
            created_at: std::time::SystemTime::now(),
        })
    }

    pub fn load(data: &[u8]) -> Result<crate::runtime::BoaRuntime, boaError> {
        let runtime = crate::runtime::BoaRuntime::new();
        Ok(runtime)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
}
