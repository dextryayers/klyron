//! FFI bindings for klyron_telemetry
use crate::types::TelemetryConfig;

#[no_mangle]
pub extern "C" fn telemetry_create_config() -> *mut TelemetryConfig {
    Box::into_raw(Box::new(TelemetryConfig::default()))
}

#[no_mangle]
pub extern "C" fn telemetry_free_config(ptr: *mut TelemetryConfig) {
    if !ptr.is_null() { unsafe { let _ = Box::from_raw(ptr); } }
}
