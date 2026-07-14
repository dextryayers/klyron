use crate::error::V8Error;
use crate::runtime::V8Runtime;
use std::time::SystemTime;

pub struct V8Snapshot {
    data: Vec<u8>,
    created_at: SystemTime,
}

impl V8Snapshot {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, created_at: SystemTime::now() }
    }

    pub fn create(_runtime: &V8Runtime) -> Result<Self, V8Error> {
        Ok(Self {
            data: Vec::new(),
            created_at: SystemTime::now(),
        })
    }

    pub fn load(_data: &[u8]) -> Result<V8Runtime, V8Error> {
        V8Runtime::new()
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn created_at(&self) -> SystemTime {
        self.created_at
    }
}
