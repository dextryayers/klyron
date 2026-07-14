//! FFI bindings for klyron_plugin
use crate::types::PluginConfig;

#[no_mangle]
pub extern "C" fn plugin_create_config() -> *mut PluginConfig {
    Box::into_raw(Box::new(PluginConfig::default()))
}

#[no_mangle]
pub extern "C" fn plugin_free_config(ptr: *mut PluginConfig) {
    if !ptr.is_null() { unsafe { let _ = Box::from_raw(ptr); } }
}
