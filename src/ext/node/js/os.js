import { op_process_info } from "ext:core/ops";
import { op_os_totalmem, op_os_freemem, op_os_cpus, op_os_uptime, op_os_network_interfaces, op_os_loadavg } from "ext:core/ops";

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
export function uptime() { return op_os_uptime(); }
export function totalmem() { return op_os_totalmem(); }
export function freemem() { return op_os_freemem(); }
export function cpus() { return JSON.parse(op_os_cpus()); }
export function networkInterfaces() { return JSON.parse(op_os_network_interfaces()); }
export function loadavg() { return JSON.parse(op_os_loadavg()); }
export function userInfo() { return { username: "root", uid: 0, gid: 0, shell: "/bin/bash", homedir: "/root" }; }

export default { platform, arch, type, release, homedir, tmpdir, hostname, uptime, totalmem, freemem, cpus, networkInterfaces, loadavg, userInfo };