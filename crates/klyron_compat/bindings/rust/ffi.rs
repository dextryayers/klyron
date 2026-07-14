//! FFI bindings for klyron_compat
use crate::types::CompatConfig;

#[no_mangle]
pub extern "C" fn compat_create_config() -> *mut CompatConfig {
    Box::into_raw(Box::new(CompatConfig::default()))
}

#[no_mangle]
pub extern "C" fn compat_free_config(ptr: *mut CompatConfig) {
    if !ptr.is_null() { unsafe { let _ = Box::from_raw(ptr); } }
}
