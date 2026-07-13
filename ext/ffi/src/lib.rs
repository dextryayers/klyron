use deno_core::{extension, op2, Extension};
use deno_error::JsErrorBox;

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
fn op_ffi_open(#[string] _path: String) -> Result<String, JsErrorBox> {
  Err(JsErrorBox::generic("FFI is not yet supported on this platform"))
}

#[op2]
#[string]
fn op_ffi_call(_lib_id: i32, #[string] _fn_name: String, #[string] _args_json: String) -> Result<String, JsErrorBox> {
  Err(JsErrorBox::generic("FFI is not yet supported on this platform"))
}
