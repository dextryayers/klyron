//! FFI bindings for klyron_config
use crate::types::ConfigConfig;

#[no_mangle]
pub extern "C" fn config_create_config() -> *mut ConfigConfig {
    Box::into_raw(Box::new(ConfigConfig::default()))
}

#[no_mangle]
pub extern "C" fn config_free_config(ptr: *mut ConfigConfig) {
    if !ptr.is_null() { unsafe { let _ = Box::from_raw(ptr); } }
}
