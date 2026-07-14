//! JS bindings registration for Jsc

pub fn register_bindings() -> Vec<&'static str> {
    vec!["console", "timers", "fetch"]
}

pub fn get_native_binding(_name: &str) -> Option<fn() -> String> {
    None
}
