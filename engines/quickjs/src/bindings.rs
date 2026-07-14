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

pub fn register_bindings() -> Vec<&'static str> {
    vec!["console", "timers"]
}

pub fn get_native_binding(_name: &str) -> Option<fn() -> String> {
    None
}
