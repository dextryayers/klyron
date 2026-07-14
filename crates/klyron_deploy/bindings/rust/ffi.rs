//! FFI bindings for klyron_deploy
use crate::types::DeployConfig;

#[no_mangle]
pub extern "C" fn deploy_create_config() -> *mut DeployConfig {
    Box::into_raw(Box::new(DeployConfig::default()))
}

#[no_mangle]
pub extern "C" fn deploy_free_config(ptr: *mut DeployConfig) {
    if !ptr.is_null() { unsafe { let _ = Box::from_raw(ptr); } }
}
