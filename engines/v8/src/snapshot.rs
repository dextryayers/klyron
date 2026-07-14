//! Snapshot support for V8

use crate::error::v8Error;

pub struct V8Snapshot {
    data: Vec<u8>,
    created_at: std::time::SystemTime,
}

impl V8Snapshot {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, created_at: std::time::SystemTime::now() }
    }

    pub fn create(runtime: &crate::runtime::V8Runtime) -> Result<Self, v8Error> {
        if !runtime.is_initialized() {
            return Err(v8Error::NotInitialized);
        }
        Ok(Self {
            data: vec![],
            created_at: std::time::SystemTime::now(),
        })
    }

    pub fn load(data: &[u8]) -> Result<crate::runtime::V8Runtime, v8Error> {
        let runtime = crate::runtime::V8Runtime::new();
        Ok(runtime)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
}
