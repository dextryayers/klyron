import { op_process_spawn, op_process_exec, op_process_kill } from "ext:core/ops";

export function spawn(cmd, args = []) {
  const result = op_process_spawn(cmd, JSON.stringify(args));
  return { pid: result.pid, id: result.id };
}

export function exec(cmd, args = []) {
  const result = op_process_exec(cmd, JSON.stringify(args));
  return {
    stdout: result.stdout,
    stderr: result.stderr,
    code: result.code,
    success: result.success,
  };
}

export function kill(id, signal = "SIGTERM") {
  op_process_kill(id, signal);
}

export default { spawn, exec, kill };
