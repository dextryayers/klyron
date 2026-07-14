// Klyron Runtime — node:child_process polyfill
// spawn, exec, execFile, fork, execSync, spawnSync

const EE = (typeof require === 'function' ? (function() { try { return require('./node_events'); } catch {} })() : null) || { EventEmitter: globalThis.EventEmitter || class {} };
const EventEmitter = EE.EventEmitter;

class ChildProcess extends EventEmitter {
  constructor() {
    super();
    this.pid = -1;
    this.ppid = -1;
    this.stdout = new EventEmitter();
    this.stderr = new EventEmitter();
    this.stdin = new EventEmitter();
    this.stdio = [this.stdin, this.stdout, this.stderr];
    this.exitCode = null;
    this.signalCode = null;
    this.killed = false;
    this.connected = true;
    this.channel = null;

    this.stdout.setEncoding = () => {};
    this.stderr.setEncoding = () => {};
    this.stdout.read = () => {};
    this.stderr.read = () => {};
    this.stdin.write = () => true;
    this.stdin.end = () => {};
  }

  kill(signal = 'SIGTERM') {
    if (this.killed) return false;
    this.killed = true;
    this.emit('exit', this.exitCode !== null ? this.exitCode : null, signal);
    this.emit('close', this.exitCode !== null ? this.exitCode : null, signal);
    return true;
  }

  disconnect() {
    this.connected = false;
  }

  ref() {}
  unref() {}
}

function spawn(command, args = [], options = {}) {
  const child = new ChildProcess();
  child.pid = Math.floor(Math.random() * 65535) + 1;
  child.ppid = typeof process !== 'undefined' && process.pid ? process.pid : 0;

  const maxBuffer = options.maxBuffer || 1024 * 1024;
  const shell = options.shell || false;
  const cwd = options.cwd || (typeof process !== 'undefined' && process.cwd ? process.cwd() : '.');
  const env = options.env || (typeof process !== 'undefined' && process.env ? process.env : {});

  const isWindows = typeof process !== 'undefined' && process.platform === 'win32';
  const cmd = shell
    ? (isWindows ? process.env.COMSPEC || 'cmd.exe' : '/bin/sh')
    : command;
  const cmdArgs = shell
    ? (isWindows ? ['/d', '/s', '/c', `"${command} ${args.join(' ')}"`] : ['-c', `${command} ${args.join(' ')}`])
    : args;

  let stdout = '';
  let stderr = '';

  if (options.stdio && options.stdio.includes('inherit')) {
  }

  try {
    const result = Klyron_core ? Klyron_core.spawn(cmd, cmdArgs, cwd, env) : null;

    if (result && result.status !== undefined) {
      child.exitCode = result.status;
      if (result.stdout) {
        stdout = typeof result.stdout === 'string' ? result.stdout : new TextDecoder().decode(result.stdout);
        child.stdout.emit('data', stdout);
      }
      if (result.stderr) {
        stderr = typeof result.stderr === 'string' ? result.stderr : new TextDecoder().decode(result.stderr);
        child.stderr.emit('data', stderr);
      }

      queueMicrotask(() => {
        child.stdout.emit('end');
        child.stderr.emit('end');
        child.emit('exit', child.exitCode, null);
        child.emit('close', child.exitCode, null);
      });
    } else {
      queueMicrotask(() => {
        child.exitCode = 0;
        child.emit('exit', 0, null);
        child.emit('close', 0, null);
      });
    }
  } catch (e) {
    queueMicrotask(() => {
      child.exitCode = 1;
      const errMsg = e.message || String(e);
      stderr = errMsg;
      child.stderr.emit('data', errMsg);
      child.stderr.emit('end');
      child.emit('error', e);
      child.emit('exit', 1, null);
      child.emit('close', 1, null);
    });
  }

  if (options.stdio && options.stdio.includes('pipe')) {
  }

  child.stdout.on = child.stdout.on || child.stdout.addEventListener;
  child.stderr.on = child.stderr.on || child.stderr.addEventListener;

  return child;
}

function spawnSync(command, args = [], options = {}) {
  const maxBuffer = options.maxBuffer || 1024 * 1024;
  const shell = options.shell || false;
  const cwd = options.cwd || (typeof process !== 'undefined' && process.cwd ? process.cwd() : '.');
  const env = options.env || (typeof process !== 'undefined' && process.env ? process.env : {});
  const input = options.input || null;

  const isWindows = typeof process !== 'undefined' && process.platform === 'win32';
  const cmd = shell
    ? (isWindows ? process.env.COMSPEC || 'cmd.exe' : '/bin/sh')
    : command;
  const cmdArgs = shell
    ? (isWindows ? ['/d', '/s', '/c', `"${command} ${args.join(' ')}"`] : ['-c', `${command} ${args.join(' ')}`])
    : args;

  let stdout = '';
  let stderr = '';
  let status = 0;
  let error = null;

  try {
    const result = Klyron_core ? Klyron_core.spawn(cmd, cmdArgs, cwd, env) : null;
    if (result) {
      status = result.status || 0;
      stdout = result.stdout || '';
      stderr = result.stderr || '';
    }
  } catch (e) {
    status = 1;
    error = e;
    stderr = e.message || String(e);
  }

  return {
    pid: Math.floor(Math.random() * 65535) + 1,
    output: [null, stdout, stderr],
    stdout,
    stderr,
    status,
    signal: null,
    error,
    pid_floating: false,
  };
}

function exec(command, options, callback) {
  if (typeof options === 'function') { callback = options; options = {}; }
  if (!options) options = {};

  const child = spawn(options.shell || '/bin/sh', ['-c', command], options);
  let stdout = '';
  let stderr = '';
  let error = null;

  child.stdout.on('data', data => { stdout += data; });
  child.stderr.on('data', data => { stderr += data; });
  child.on('error', err => { error = err; });

  child.on('exit', (code) => {
    if (callback) {
      if (error) {
        error.code = code;
        error.stdout = stdout;
        error.stderr = stderr;
        callback(error, stdout, stderr);
      } else {
        callback(null, stdout, stderr);
      }
    }
  });

  return child;
}

function execFile(file, args, options, callback) {
  if (typeof args === 'function') { callback = args; args = []; }
  if (typeof options === 'function') { callback = options; options = {}; }
  if (!args) args = [];
  if (!options) options = {};
  return spawn(file, args, options);
}

function fork(modulePath, args = [], options = {}) {
  const child = spawn(process.execPath || 'node', [modulePath, ...args], options);
  child.send = (msg) => {
    child.emit('message', msg);
    return true;
  };
  return child;
}

const child_process = {
  spawn,
  spawnSync,
  exec,
  execSync: (command, options) => {
    const result = spawnSync('/bin/sh', ['-c', command], options);
    if (result.error) throw result.error;
    return result.stdout;
  },
  execFile,
  execFileSync: (file, args, options) => {
    const result = spawnSync(file, args, options);
    if (result.error) throw result.error;
    return result.stdout;
  },
  fork,
  ChildProcess,
};

if (typeof module !== 'undefined' && module.exports) {
  module.exports = child_process;
}
