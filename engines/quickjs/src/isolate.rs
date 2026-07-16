use crate::error::QuickJSError;

pub struct QuickJSIsolate;

impl QuickJSIsolate {
    pub fn create_isolate() -> Result<Self, QuickJSError> {
        Ok(Self)
    }

    pub fn create_context(&mut self) -> Result<(), QuickJSError> {
        Ok(())
    }
}