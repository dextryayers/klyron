import { op_process_info, op_process_env } from "ext:core/ops";

let _info = null;
function info() {
  if (!_info) _info = JSON.parse(op_process_info());
  return _info;
}

export function platform() { return info().platform || "linux"; }
export function arch() { return info().arch || "x64"; }
export function type() { return "Linux"; }
export function release() { return "6.0.0"; }
export function homedir() { return "/root"; }
export function tmpdir() { return "/tmp"; }
export function hostname() { return "klyron"; }
export function uptime() { return 0; }
export function totalmem() { return 8589934592; }
export function freemem() { return 4294967296; }
export function cpus() {
  return [{ model: "Klyron Virtual CPU", speed: 2000, times: { user: 0, nice: 0, sys: 0, idle: 0, irq: 0 } }];
}
export function networkInterfaces() { return {}; }
export function userInfo() { return { username: "root", uid: 0, gid: 0, shell: "/bin/bash", homedir: "/root" }; }
