use std::ffi::CString;
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn klyron_cache_version() -> *mut c_char {
    CString::new(env!("CARGO_PKG_VERSION")).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn klyron_cache_string_free(s: *mut c_char) {
    if !s.is_null() { unsafe { let _ = CString::from_raw(s); } }
}
