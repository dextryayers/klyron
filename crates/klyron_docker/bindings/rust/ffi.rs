//! FFI bindings for klyron_docker
use crate::types::DockerConfig;

#[no_mangle]
pub extern "C" fn docker_create_config() -> *mut DockerConfig {
    Box::into_raw(Box::new(DockerConfig::default()))
}

#[no_mangle]
pub extern "C" fn docker_free_config(ptr: *mut DockerConfig) {
    if !ptr.is_null() { unsafe { let _ = Box::from_raw(ptr); } }
}
