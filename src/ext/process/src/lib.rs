use std::collections::HashMap;
use std::sync::atomic::AtomicU64;

use deno_core::{extension, op2, Extension, OpState};
use deno_error::JsErrorBox;
use ::klyron_process::ProcessManager;

static NEXT_CHILD_ID: AtomicU64 = AtomicU64::new(1);

#[derive(serde::Serialize)]
struct SpawnResult {
  pid: u32,
  id: u64,
}

#[derive(serde::Serialize)]
struct ExecResult {
  stdout: String,
  stderr: String,
  code: Option<i32>,
  success: bool,
}

struct ProcessState {
  children: HashMap<u64, ::klyron_process::ChildProcess>,
}

extension!(
  klyron_process_ext,
  ops = [op_process_spawn, op_process_exec, op_process_kill],
  esm_entry_point = "ext:klyron_process/process.js",
  esm = [dir "js", "process.js"],
  state = |state| { state.put::<ProcessState>(ProcessState { children: HashMap::new() }); },
);

pub fn init() -> Extension {
  klyron_process_ext::init()
}

#[op2]
#[serde]
fn op_process_spawn(
  state: &mut OpState,
  #[string] cmd: String,
  #[string] args_json: String,
) -> Result<SpawnResult, JsErrorBox> {
  let args: Vec<String> = serde_json::from_str(&args_json)
    .map_err(|e| JsErrorBox::generic(format!("spawn args: {e}")))?;
  let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
  let pm = ProcessManager::new();
  let child = pm
    .spawn(&cmd, &args_refs)
    .map_err(|e| JsErrorBox::generic(format!("spawn {cmd}: {e}")))?;
  let pid = child.pid();
  let id = NEXT_CHILD_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
  let ps = state.borrow_mut::<ProcessState>();
  ps.children.insert(id, child);
  Ok(SpawnResult { pid, id })
}

#[op2]
#[serde]
fn op_process_exec(
  #[string] cmd: String,
  #[string] args_json: String,
) -> Result<ExecResult, JsErrorBox> {
  let args: Vec<String> = serde_json::from_str(&args_json)
    .map_err(|e| JsErrorBox::generic(format!("exec args: {e}")))?;
  let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
  let pm = ProcessManager::new();
  let result = pm
    .exec(&cmd, &args_refs)
    .map_err(|e| JsErrorBox::generic(format!("exec {cmd}: {e}")))?;
  Ok(ExecResult {
    stdout: result.stdout,
    stderr: result.stderr,
    code: result.exit_code,
    success: result.success,
  })
}

#[op2(fast)]
fn op_process_kill(
  state: &mut OpState,
  id: f64,
  #[string] signal: String,
) -> Result<(), JsErrorBox> {
  let child_id = id as u64;
  let ps = state.borrow_mut::<ProcessState>();
  if let Some(child) = ps.children.get_mut(&child_id) {
    if signal.eq_ignore_ascii_case("sigkill") || signal == "9" {
      child
        .kill()
        .map_err(|e| JsErrorBox::generic(format!("kill: {e}")))?;
    } else {
      let sig_num = match signal.to_lowercase().as_str() {
        "sigterm" | "term" | "15" => 15,
        "sighup" | "hup" | "1" => 1,
        "sigint" | "int" | "2" => 2,
        "sigquit" | "quit" | "3" => 3,
        "sigstop" | "stop" | "19" => 19,
        "sigcont" | "cont" | "18" => 18,
        _ => 15,
      };
      let pm = ProcessManager::new();
      pm
        .signal(child.pid(), sig_num)
        .map_err(|e| JsErrorBox::generic(format!("signal: {e}")))?;
    }
    Ok(())
  } else {
    Err(JsErrorBox::generic(format!("Child process {child_id} not found")))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_init_returns_extension() {
    let ext = init();
    assert_eq!(ext.name, "klyron_process_ext");
  }
}
