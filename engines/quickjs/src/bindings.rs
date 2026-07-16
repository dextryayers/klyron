use crate::error::QuickJSError;

pub struct QuickJSBindings;

impl QuickJSBindings {
    pub fn new() -> Self {
        Self
    }

    pub fn register_bindings(&self, _ctx: *mut std::ffi::c_void) -> Result<(), QuickJSError> {
        Ok(())
    }
}