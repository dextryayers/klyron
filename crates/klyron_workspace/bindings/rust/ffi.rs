//! FFI bindings for klyron_workspace
use crate::types::WorkspaceConfig;

#[no_mangle]
pub extern "C" fn workspace_create_config() -> *mut WorkspaceConfig {
    Box::into_raw(Box::new(WorkspaceConfig::default()))
}

#[no_mangle]
pub extern "C" fn workspace_free_config(ptr: *mut WorkspaceConfig) {
    if !ptr.is_null() { unsafe { let _ = Box::from_raw(ptr); } }
}
