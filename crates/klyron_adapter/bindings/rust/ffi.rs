//! FFI bindings for klyron_adapter
use crate::types::AdapterConfig;

#[no_mangle]
pub extern "C" fn adapter_create_config() -> *mut AdapterConfig {
    Box::into_raw(Box::new(AdapterConfig::default()))
}

#[no_mangle]
pub extern "C" fn adapter_free_config(ptr: *mut AdapterConfig) {
    if !ptr.is_null() { unsafe { let _ = Box::from_raw(ptr); } }
}
