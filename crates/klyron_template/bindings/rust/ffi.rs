//! FFI bindings for klyron_template
use crate::types::TemplateConfig;

#[no_mangle]
pub extern "C" fn template_create_config() -> *mut TemplateConfig {
    Box::into_raw(Box::new(TemplateConfig::default()))
}

#[no_mangle]
pub extern "C" fn template_free_config(ptr: *mut TemplateConfig) {
    if !ptr.is_null() { unsafe { let _ = Box::from_raw(ptr); } }
}
