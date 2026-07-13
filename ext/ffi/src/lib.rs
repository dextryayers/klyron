use deno_core::{extension, op2, Extension};

extension!(
  klyron_ffi,
  ops = [op_ffi_open, op_ffi_call],
  esm_entry_point = "ext:klyron_ffi/ffi.js",
  esm = [dir "js", "ffi.js"],
);

pub fn init() -> Extension {
  klyron_ffi::init()
}

#[op2]
#[string]
fn op_ffi_open(#[string] _path: String) -> Result<String, String> {
  Err("FFI is not yet supported on this platform".to_string())
}

#[op2]
#[string]
fn op_ffi_call(#[number] _lib_id: u32, #[string] _fn_name: String, #[string] _args_json: String) -> Result<String, String> {
  Err("FFI is not yet supported on this platform".to_string())
}
