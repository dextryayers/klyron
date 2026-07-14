use crate::error::V8Error;

pub struct V8Bindings;

impl V8Bindings {
    pub fn new() -> Self {
        Self
    }

    pub fn register_bindings(&self) -> Result<(), V8Error> {
        Ok(())
    }
}

pub fn register_bindings() -> Vec<&'static str> {
    vec!["console", "timers", "fetch"]
}

pub fn get_native_binding(_name: &str) -> Option<fn() -> String> {
    None
}
