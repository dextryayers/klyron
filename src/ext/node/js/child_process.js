import { op_process_spawn, op_process_exec } from "ext:core/ops";
import { EventEmitter } from "./events.js";
import { Writable, Readable } from "./stream.js";

class ChildProcess extends EventEmitter {
  constructor(pid) {
    super();
    this.pid = pid;
    this.stdout = new Readable();
    this.stderr = new Readable();
    this.stdin = new Writable();
  }
  kill(signal) {}
}

export function spawn(cmd, args, opts) {
  try {
    const result = JSON.parse(op_process_spawn(cmd, JSON.stringify(args || [])));
    const cp = new ChildProcess(result.pid);
    if (result.stdout) cp.stdout.push(result.stdout);
    cp.stdout.push(null);
    if (result.stderr) cp.stderr.push(result.stderr);
    cp.stderr.push(null);
    setTimeout(() => { cp.emit("close", result.code || 0, null); }, 0);
    return cp;
  } catch (e) {
    const cp = new ChildProcess(-1);
    setTimeout(() => cp.emit("error", e), 0);
    return cp;
  }
}

export function exec(cmd, opts, cb) {
  if (typeof opts === "function") { cb = opts; opts = {}; }
  try {
    const result = JSON.parse(op_process_exec(cmd));
    cb?.(null, result.stdout || "", result.stderr || "");
  } catch (e) { cb?.(e, "", ""); }
}

export function execSync(cmd, opts) {
  try {
    const result = JSON.parse(op_process_exec(cmd));
    if (result.code !== 0) throw new Error(`Command failed: ${cmd}\n${result.stderr}`);
    return result.stdout || "";
  } catch (e) { throw e; }
}

export function spawnSync(cmd, args, opts) {
  try {
    const result = JSON.parse(op_process_spawn(cmd, JSON.stringify(args || [])));
    return { pid: result.pid, output: [null, result.stdout || "", result.stderr || ""], stdout: result.stdout || "", stderr: result.stderr || "", status: result.code || 0, signal: null, error: null };
  } catch (e) { return { pid: -1, output: [null, "", ""], stdout: "", stderr: "", status: -1, signal: null, error: e }; }
}

export default { spawn, exec, execSync, spawnSync, ChildProcess };
