use crate::error::JSCError;

pub struct JSCBindings;

impl JSCBindings {
    pub fn new() -> Self {
        Self
    }

    pub fn register_bindings(&self, _ctx: *mut std::ffi::c_void) -> Result<(), JSCError> {
        Ok(())
    }
}

pub fn register_bindings() -> Vec<&'static str> {
    vec!["console", "timers"]
}

pub fn get_native_binding(_name: &str) -> Option<fn() -> String> {
    None
}
