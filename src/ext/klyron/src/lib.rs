use deno_core::{extension, op2, Extension};

extension!(
  klyron_klyron,
  ops = [op_klyron_version, op_klyron_arch, op_klyron_platform],
  esm_entry_point = "ext:klyron_klyron/klyron.js",
  esm = [dir "js", "klyron.js"],
);

pub fn init() -> Extension {
  klyron_klyron::init()
}

#[op2]
#[string]
fn op_klyron_version() -> String {
  env!("CARGO_PKG_VERSION").to_string()
}

#[op2]
#[string]
fn op_klyron_arch() -> String {
  std::env::consts::ARCH.to_string()
}

#[op2]
#[string]
fn op_klyron_platform() -> String {
  std::env::consts::OS.to_string()
}
