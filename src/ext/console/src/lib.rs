use deno_core::{extension, op2, Extension};

extension!(
  klyron_console,
  ops = [op_console_log, op_console_error, op_console_warn, op_console_info],
  esm_entry_point = "ext:klyron_console/console.js",
  esm = [dir "js", "console.js"],
);

pub fn init() -> Extension {
  klyron_console::init()
}

#[op2(fast)]
fn op_console_log(#[string] msg: String) {
  println!("{}", msg);
}

#[op2(fast)]
fn op_console_error(#[string] msg: String) {
  eprintln!("{}", msg);
}

#[op2(fast)]
fn op_console_warn(#[string] msg: String) {
  eprintln!("{}", msg);
}

#[op2(fast)]
fn op_console_info(#[string] msg: String) {
  println!("{}", msg);
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_init_returns_extension() {
    let ext = init();
    assert_eq!(ext.name, "klyron_console");
  }

  #[test]
  fn test_console_integration() {
    let runtime = klyron_core::Runtime::builder()
      .enable_typescript(false)
      .extension(init())
      .build()
      .unwrap();
    assert_eq!(runtime.eval("console.log('hi')").unwrap(), "null");
  }
}
