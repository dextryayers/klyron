import { op_fs_read_file } from "ext:core/ops";
import { op_os_info } from "ext:core/ops";

let _info = null;
function info() {
  if (!_info) _info = JSON.parse(op_os_info());
  return _info;
}

export function platform() { return process.platform || "linux"; }
export function arch() { return process.arch || "x64"; }
export function type() { return "Linux"; }
export function release() { return "6.0.0"; }
export function homedir() { return process.env.HOME || "/root"; }
export function tmpdir() { return "/tmp"; }
export function hostname() { return "klyron"; }
export function uptime() { return 0; }
export function totalmem() { return 8589934592; }
export function freemem() { return 4294967296; }
export function cpus() {
  return [{ model: "Klyron Virtual CPU", speed: 2400, times: { user: 0, nice: 0, sys: 0, idle: 0, irq: 0 } }];
}
export function endianness() { return "LE"; }
export function loadavg() { return [0, 0, 0]; }
export function networkInterfaces() { return {}; }
export function userInfo() { return { username: "root", uid: 0, gid: 0, shell: "/bin/bash", homedir: homedir() }; }
export function EOL() { return "\n"; }
export function version() { return "0.1.0"; }

export default { platform, arch, type, release, homedir, tmpdir, hostname, uptime, totalmem, freemem, cpus, endianness, loadavg, networkInterfaces, userInfo, EOL, version };
