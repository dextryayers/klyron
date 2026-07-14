//! FFI bindings for klyron_shell
use crate::types::ShellConfig;

#[no_mangle]
pub extern "C" fn shell_create_config() -> *mut ShellConfig {
    Box::into_raw(Box::new(ShellConfig::default()))
}

#[no_mangle]
pub extern "C" fn shell_free_config(ptr: *mut ShellConfig) {
    if !ptr.is_null() { unsafe { let _ = Box::from_raw(ptr); } }
}
