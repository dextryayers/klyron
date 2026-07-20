import { op_process_info, op_process_args, op_process_env, op_process_exit, op_process_cwd, op_process_hrtime, op_process_memory_usage, op_process_uptime, op_process_cpu_usage } from "ext:core/ops";

const _args = JSON.parse(op_process_args());
const _env = JSON.parse(op_process_env());
const _info = JSON.parse(op_process_info());

const process = {
  pid: _info.pid,
  ppid: _info.ppid || 0,
  platform: _info.platform || "linux",
  arch: _info.arch || "x64",
  version: "v18.0.0",
  versions: { node: "18.0.0", v8: _info.v8_version || "11.0", uv: "1.44", zlib: "1.2", openssl: "3.0" },
  argv: _args,
  argv0: _args[0] || "",
  env: _env,
  cwd() { return op_process_cwd(); },
  chdir(dir) {},
  exit(code) { op_process_exit(code || 0); },
  hrtime(prev) {
    const t = JSON.parse(op_process_hrtime());
    if (prev) return [t[0] - prev[0], t[1] - prev[1]];
    return t;
  },
  memoryUsage() { return JSON.parse(op_process_memory_usage()); },
  cpuUsage() { return JSON.parse(op_process_cpu_usage()); },
  uptime() { return op_process_uptime(); },
  nextTick(cb, ...args) { queueMicrotask(() => cb(...args)); },
  stdout: { write(s) { console.log(s); }, writable: true },
  stderr: { write(s) { console.error(s); }, writable: true },
  stdin: { readable: false },
  exitCode: 0,
};

globalThis.setImmediate = (fn, ...args) => setTimeout(fn, 0, ...args);
globalThis.clearImmediate = clearTimeout;

export default process;
