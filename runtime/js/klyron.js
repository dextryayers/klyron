// Klyron Runtime — Klyron global bindings
// Klyron.io, Klyron.net, Klyron.fs, Klyron.crypto, Klyron.hrtime, Klyron.timers

globalThis.Klyron = {
  io: {
    stdout: {
      write(str) { Klyron_core.writeStdout(str); },
    },
    stderr: {
      write(str) { Klyron_core.writeStderr(str); },
    },
    stdin: {
      read(size) { return Klyron_core.readStdin(size); },
    },
  },

  net: {
    fetch(opts) {
      return Klyron_core.fetch(opts);
    },
  },

  fs: {
    readFileSync(path) { return Klyron_core.readFileSync(path); },
    writeFileSync(path, data) { Klyron_core.writeFileSync(path, data); },
    existsSync(path) { return Klyron_core.existsSync(path); },
    mkdirSync(path) { Klyron_core.mkdirSync(path); },
    readDirSync(path) { return Klyron_core.readDirSync(path); },
    removeSync(path) { Klyron_core.removeSync(path); },
    copySync(src, dest) { Klyron_core.copySync(src, dest); },
    statSync(path) { return Klyron_core.statSync(path); },
  },

  crypto: {
    digest(algo, data) { return Klyron_core.digest(algo, data); },
    randomBytes(size) { return Klyron_core.randomBytes(size); },
    randomUUID() { return Klyron_core.randomUUID(); },
  },

  timers: {
    setTimeout(fn, ms, ...args) { return Klyron_core.setTimeout(fn, ms, args); },
    clearTimeout(id) { Klyron_core.clearTimeout(id); },
    setInterval(fn, ms, ...args) { return Klyron_core.setInterval(fn, ms, args); },
    clearInterval(id) { Klyron_core.clearInterval(id); },
  },

  process: {
    env() { return Klyron_core.env(); },
    argv() { return Klyron_core.argv(); },
    cwd() { return Klyron_core.cwd(); },
    exit(code) { Klyron_core.exit(code); },
    pid: Klyron_core.pid(),
    ppid: Klyron_core.ppid(),
    platform: Klyron_core.platform(),
    arch: Klyron_core.arch(),
    version: Klyron_core.version(),
    versions: Klyron_core.versions(),
  },

  hrtime() {
    return Klyron_core.hrtime();
  },
};
