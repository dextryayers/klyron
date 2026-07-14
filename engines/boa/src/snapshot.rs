use crate::error::BoaError;
use crate::runtime::BoaRuntime;
use std::time::SystemTime;

pub struct BoaSnapshot {
    data: Vec<u8>,
    created_at: SystemTime,
}

impl BoaSnapshot {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, created_at: SystemTime::now() }
    }

    pub fn create(_runtime: &BoaRuntime) -> Result<Self, BoaError> {
        Ok(Self {
            data: Vec::new(),
            created_at: SystemTime::now(),
        })
    }

    pub fn load(_data: &[u8]) -> Result<BoaRuntime, BoaError> {
        Ok(BoaRuntime::new())
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn created_at(&self) -> SystemTime {
        self.created_at
    }
}
