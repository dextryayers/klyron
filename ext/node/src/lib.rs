use std::collections::HashMap;

use deno_core::{extension, op2, Extension};

extension!(
  klyron_node,
  ops = [
    op_process_info, op_process_args, op_process_env, op_process_exit,
    op_process_cwd, op_process_hrtime, op_process_spawn, op_process_exec,
  ],
  esm_entry_point = "ext:klyron_node/index.js",
  esm = [dir "js", "index.js", "assert.js", "buffer.js", "child_process.js", "crypto.js", "events.js", "fs.js", "os.js", "path.js", "process.js", "querystring.js", "stream.js", "url.js", "util.js"],
);

pub fn init() -> Extension {
  klyron_node::init()
}

#[op2]
#[string]
fn op_process_info() -> String {
  serde_json::json!({
    "pid": std::process::id(),
    "ppid": 0,
    "platform": std::env::consts::OS,
    "arch": std::env::consts::ARCH,
    "v8_version": "11.0",
  })
  .to_string()
}

#[op2]
#[string]
fn op_process_args() -> String {
  let args: Vec<String> = std::env::args().collect();
  serde_json::to_string(&args).unwrap_or_else(|_| "[]".to_string())
}

#[op2]
#[string]
fn op_process_env() -> String {
  let env: HashMap<String, String> = std::env::vars().collect();
  serde_json::to_string(&env).unwrap_or_else(|_| "{}".to_string())
}

#[op2(fast)]
fn op_process_exit(#[number] code: i32) {
  std::process::exit(code);
}

#[op2]
#[string]
fn op_process_cwd() -> String {
  std::env::current_dir().map(|p| p.to_string_lossy().to_string()).unwrap_or_default()
}

#[op2]
#[string]
fn op_process_hrtime() -> String {
  let dur = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap_or_default();
  serde_json::json!([dur.as_secs(), dur.subsec_nanos()]).to_string()
}

#[op2]
#[string]
fn op_process_spawn(#[string] cmd: String, #[string] args_json: String) -> Result<String, String> {
  let args: Vec<String> = serde_json::from_str(&args_json).map_err(|e| format!("spawn args: {e}"))?;
  match std::process::Command::new(&cmd).args(&args).output() {
    Ok(output) => Ok(serde_json::json!({
      "pid": 0,
      "stdout": String::from_utf8_lossy(&output.stdout),
      "stderr": String::from_utf8_lossy(&output.stderr),
      "code": output.status.code(),
    }).to_string()),
    Err(e) => Err(format!("spawn {cmd}: {e}")),
  }
}

#[op2]
#[string]
fn op_process_exec(#[string] cmd: String) -> Result<String, String> {
  let shell = if cfg!(windows) { "cmd" } else { "sh" };
  let flag = if cfg!(windows) { "/c" } else { "-c" };
  match std::process::Command::new(shell).arg(flag).arg(&cmd).output() {
    Ok(output) => Ok(serde_json::json!({
      "stdout": String::from_utf8_lossy(&output.stdout),
      "stderr": String::from_utf8_lossy(&output.stderr),
      "code": output.status.code(),
    }).to_string()),
    Err(e) => Err(format!("exec {cmd}: {e}")),
  }
}
