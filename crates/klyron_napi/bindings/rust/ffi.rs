use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[repr(C)]
pub struct napi_module_t {
    pub name: *const c_char,
    pub exports: *const c_char,
    pub napi_version: u32,
}

#[no_mangle]
pub extern "C" fn napi_load_module(name: *const c_char) -> *mut napi_module_t {
    let name_str = unsafe { CStr::from_ptr(name) }.to_string_lossy().into_owned();
    let mut loader = super::NapiLoader::new();
    match loader.load(&name_str) {
        Ok(module) => {
            let c_name = CString::new(module.name.clone()).unwrap();
            let c_exports = CString::new("{}").unwrap();
            Box::into_raw(Box::new(napi_module_t {
                name: c_name.into_raw(),
                exports: c_exports.into_raw(),
                napi_version: 9,
            }))
        }
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn napi_free_module(module: *mut napi_module_t) {
    if !module.is_null() {
        unsafe {
            let _ = CString::from_raw((*module).name as *mut c_char);
            let _ = CString::from_raw((*module).exports as *mut c_char);
            drop(Box::from_raw(module));
        }
    }
}
