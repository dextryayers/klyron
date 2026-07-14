use std::ffi::CStr;
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn linter_version() -> u32 {
    1
}
