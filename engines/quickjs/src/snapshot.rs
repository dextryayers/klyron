use crate::error::QuickJSError;

pub struct QuickJSSnapshot;

impl QuickJSSnapshot {
    pub fn create() -> Result<Vec<u8>, QuickJSError> {
        Ok(Vec::new())
    }

    pub fn load(_data: &[u8]) -> Result<Self, QuickJSError> {
        Ok(Self)
    }
}