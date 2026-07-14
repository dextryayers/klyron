//! FFI bindings for klyron_updater

use std::ffi::CStr;
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn klyron_updater_init() -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn klyron_updater_version() -> *const c_char {
    concat!("klyron_updater v", env!("CARGO_PKG_VERSION"), "\0").as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn klyron_updater_process(input: *const c_char) -> *mut c_char {
    if input.is_null() {
        return std::ffi::CString::new("error: null input").unwrap().into_raw();
    }
    let s = unsafe { CStr::from_ptr(input) };
    let _msg = s.to_string_lossy();
    std::ffi::CString::new("ok").unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn klyron_updater_free_string(s: *mut c_char) {
    if !s.is_null() { unsafe { let _ = std::ffi::CString::from_raw(s); } }
}
