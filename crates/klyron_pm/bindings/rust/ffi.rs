use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn pm_detect(path: *const c_char) -> i32 {
    let path_str = unsafe { CStr::from_ptr(path) }.to_string_lossy().into_owned();
    let pm = super::detect(std::path::Path::new(&path_str));
    pm as i32
}

#[no_mangle]
pub extern "C" fn pm_install_cmd(pm: i32) -> *mut c_char {
    let manager = match pm {
        0 => super::PackageManager::Npm,
        1 => super::PackageManager::Yarn,
        2 => super::PackageManager::Pnpm,
        3 => super::PackageManager::Bun,
        4 => super::PackageManager::Composer,
        5 => super::PackageManager::Cargo,
        6 => super::PackageManager::Go,
        7 => super::PackageManager::Pip,
        8 => super::PackageManager::Gem,
        _ => super::PackageManager::Npm,
    };
    CString::new(super::install_cmd(manager)).unwrap().into_raw()
}
