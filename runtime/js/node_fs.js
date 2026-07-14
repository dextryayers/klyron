// Klyron Runtime — node:fs polyfill
// Synchronous + callback + promises API

const kFs = Klyron.fs;

function throwIfError(err) {
  if (err) throw err;
}

class Stats {
  constructor(stat) {
    if (typeof stat === 'object' && stat !== null) {
      this.dev = stat.dev || 0;
      this.ino = stat.ino || 0;
      this.mode = stat.mode || 0;
      this.nlink = stat.nlink || 0;
      this.uid = stat.uid || 0;
      this.gid = stat.gid || 0;
      this.rdev = stat.rdev || 0;
      this.size = stat.size || 0;
      this.blksize = stat.blksize || 0;
      this.blocks = stat.blocks || 0;
      this.atimeMs = stat.atimeMs || 0;
      this.mtimeMs = stat.mtimeMs || 0;
      this.ctimeMs = stat.ctimeMs || 0;
      this.birthtimeMs = stat.birthtimeMs || 0;
      this.atime = new Date(this.atimeMs);
      this.mtime = new Date(this.mtimeMs);
      this.ctime = new Date(this.ctimeMs);
      this.birthtime = new Date(this.birthtimeMs);
    } else {
      this.size = 0;
      this.mode = 0;
    }
  }
  isFile() { return true; }
  isDirectory() { return false; }
  isSymbolicLink() { return false; }
  isBlockDevice() { return false; }
  isCharacterDevice() { return false; }
  isFIFO() { return false; }
  isSocket() { return false; }
}

function readFileSync(path, options = {}) {
  const encoding = typeof options === 'string' ? options : options.encoding || null;
  const data = kFs.readFileSync(path);
  if (encoding === 'utf8' || encoding === 'utf-8') {
    return new TextDecoder().decode(data);
  }
  return data;
}

function writeFileSync(path, data) {
  if (typeof data === 'string') data = new TextEncoder().encode(data);
  kFs.writeFileSync(path, data);
}

function existsSync(path) {
  try { return kFs.existsSync(path); }
  catch { return false; }
}

function readdirSync(path, options = {}) {
  const withFileTypes = typeof options === 'object' && options.withFileTypes;
  const entries = kFs.readDirSync(path);
  if (withFileTypes) {
    return entries.map(name => ({
      name,
      isFile() { return true; },
      isDirectory() { return false; },
      isSymbolicLink() { return false; },
    }));
  }
  return entries;
}

function mkdirSync(path, options = {}) {
  const recursive = typeof options === 'object' ? options.recursive : false;
  try {
    kFs.mkdirSync(path);
  } catch (e) {
    if (recursive && existsSync(path)) return;
    throw e;
  }
}

function statSync(path) {
  const raw = kFs.statSync(path);
  return new Stats(raw);
}

function unlinkSync(path) {
  try { kFs.removeSync(path); }
  catch (e) { throw e; }
}

function rmdirSync(path) {
  try { kFs.removeSync(path); }
  catch (e) { throw e; }
}

function renameSync(oldPath, newPath) {
  try {
    const data = kFs.readFileSync(oldPath);
    kFs.writeFileSync(newPath, data);
    kFs.removeSync(oldPath);
  } catch (e) { throw e; }
}

function copyFileSync(src, dest) {
  try { kFs.copySync(src, dest); }
  catch (e) { throw e; }
}

function appendFileSync(path, data) {
  try {
    if (typeof data === 'string') data = new TextEncoder().encode(data);
    const existing = kFs.existsSync(path) ? kFs.readFileSync(path) : new Uint8Array(0);
    const combined = new Uint8Array(existing.length + data.length);
    combined.set(existing);
    combined.set(data, existing.length);
    kFs.writeFileSync(path, combined);
  } catch (e) { throw e; }
}

function readFile(path, options, callback) {
  if (typeof options === 'function') { callback = options; options = {}; }
  if (!callback) return new Promise((resolve, reject) => {
    readFile(path, options, (err, data) => err ? reject(err) : resolve(data));
  });
  const encoding = typeof options === 'string' ? options : options.encoding || null;
  try {
    const data = readFileSync(path, encoding);
    callback(null, data);
  } catch (err) { callback(err); }
}

function writeFile(path, data, options, callback) {
  if (typeof options === 'function') { callback = options; options = {}; }
  if (!callback) return new Promise((resolve, reject) => {
    writeFile(path, data, options, err => err ? reject(err) : resolve());
  });
  try {
    writeFileSync(path, data);
    callback(null);
  } catch (err) { callback(err); }
}

function readdir(path, options, callback) {
  if (typeof options === 'function') { callback = options; options = {}; }
  if (!callback) return new Promise((resolve, reject) => {
    readdir(path, options, (err, files) => err ? reject(err) : resolve(files));
  });
  try {
    const files = readdirSync(path, options);
    callback(null, files);
  } catch (err) { callback(err); }
}

function mkdir(path, options, callback) {
  if (typeof options === 'function') { callback = options; options = {}; }
  if (!callback) return new Promise((resolve, reject) => {
    mkdir(path, options, err => err ? reject(err) : resolve());
  });
  try {
    mkdirSync(path, options);
    callback(null);
  } catch (err) { callback(err); }
}

function unlink(path, callback) {
  if (!callback) return new Promise((resolve, reject) => {
    unlink(path, err => err ? reject(err) : resolve());
  });
  try { unlinkSync(path); callback(null); }
  catch (err) { callback(err); }
}

function rmdir(path, callback) {
  if (!callback) return new Promise((resolve, reject) => {
    rmdir(path, err => err ? reject(err) : resolve());
  });
  try { rmdirSync(path); callback(null); }
  catch (err) { callback(err); }
}

function rename(oldPath, newPath, callback) {
  if (!callback) return new Promise((resolve, reject) => {
    rename(oldPath, newPath, err => err ? reject(err) : resolve());
  });
  try { renameSync(oldPath, newPath); callback(null); }
  catch (err) { callback(err); }
}

function stat(path, callback) {
  if (!callback) return new Promise((resolve, reject) => {
    stat(path, (err, s) => err ? reject(err) : resolve(s));
  });
  try { callback(null, statSync(path)); }
  catch (err) { callback(err); }
}

function lstat(path, callback) {
  return stat(path, callback);
}

function accessSync(path) {
  if (!existsSync(path)) throw new Error(`ENOENT: no such file or directory, access '${path}'`);
}

function access(path, callback) {
  if (!callback) return new Promise((resolve, reject) => {
    access(path, err => err ? reject(err) : resolve());
  });
  try { accessSync(path); callback(null); }
  catch (err) { callback(err); }
}

function appendFile(path, data, options, callback) {
  if (typeof options === 'function') { callback = options; options = {}; }
  if (!callback) return new Promise((resolve, reject) => {
    appendFile(path, data, options, err => err ? reject(err) : resolve());
  });
  try { appendFileSync(path, data); callback(null); }
  catch (err) { callback(err); }
}

function copyFile(src, dest, flags, callback) {
  if (typeof flags === 'function') { callback = flags; flags = 0; }
  if (!callback) return new Promise((resolve, reject) => {
    copyFile(src, dest, flags, err => err ? reject(err) : resolve());
  });
  try { copyFileSync(src, dest); callback(null); }
  catch (err) { callback(err); }
}

// Stream stubs
function createReadStream(path, options = {}) {
  const { EventEmitter } = require('./node_events') || globalThis;
  const stream = new (globalThis.EventEmitter || Object)();
  stream.readable = true;
  stream.path = path;
  stream.bytesRead = 0;
  stream.push = function(chunk) {
    stream.emit('data', chunk);
  };
  stream.pipe = function(dest) {
    stream.on('data', chunk => dest.write(chunk));
    stream.on('end', () => dest.end());
    return dest;
  };
  stream.close = function() {};
  stream.destroy = function() {};
  stream._read = function() {};
  queueMicrotask(() => {
    try {
      const data = readFileSync(path);
      stream.bytesRead = data.length;
      stream.push(data);
      stream.emit('end');
      stream.emit('close');
    } catch (e) {
      stream.emit('error', e);
    }
  });
  return stream;
}

function createWriteStream(path, options = {}) {
  const chunks = [];
  const stream = new (globalThis.EventEmitter || Object)();
  stream.writable = true;
  stream.path = path;
  stream.bytesWritten = 0;
  stream.write = function(chunk) {
    if (typeof chunk === 'string') chunk = new TextEncoder().encode(chunk);
    chunks.push(chunk);
    stream.bytesWritten += chunk.length;
    return true;
  };
  stream.end = function(chunk) {
    if (chunk) stream.write(chunk);
    try {
      const data = chunks.length === 1 ? chunks[0] : (() => {
        const total = chunks.reduce((s, c) => s + c.length, 0);
        const buf = new Uint8Array(total);
        let off = 0;
        for (const c of chunks) { buf.set(c, off); off += c.length; }
        return buf;
      })();
      writeFileSync(path, data);
      stream.emit('finish');
      stream.emit('close');
    } catch (e) {
      stream.emit('error', e);
    }
  };
  stream.close = stream.end;
  stream.destroy = function() {};
  return stream;
}

// Promises API
const promises = {
  readFile: (path, options) => new Promise((resolve, reject) => {
    readFile(path, options, (err, data) => err ? reject(err) : resolve(data));
  }),
  writeFile: (path, data, options) => new Promise((resolve, reject) => {
    writeFile(path, data, options, err => err ? reject(err) : resolve());
  }),
  readdir: (path, options) => new Promise((resolve, reject) => {
    readdir(path, options, (err, files) => err ? reject(err) : resolve(files));
  }),
  mkdir: (path, options) => new Promise((resolve, reject) => {
    mkdir(path, options, err => err ? reject(err) : resolve());
  }),
  stat: (path) => new Promise((resolve, reject) => {
    stat(path, (err, s) => err ? reject(err) : resolve(s));
  }),
  unlink: (path) => new Promise((resolve, reject) => {
    unlink(path, err => err ? reject(err) : resolve());
  }),
  rmdir: (path) => new Promise((resolve, reject) => {
    rmdir(path, err => err ? reject(err) : resolve());
  }),
  rename: (oldPath, newPath) => new Promise((resolve, reject) => {
    rename(oldPath, newPath, err => err ? reject(err) : resolve());
  }),
  access: (path) => new Promise((resolve, reject) => {
    access(path, err => err ? reject(err) : resolve());
  }),
  appendFile: (path, data, options) => new Promise((resolve, reject) => {
    appendFile(path, data, options, err => err ? reject(err) : resolve());
  }),
  copyFile: (src, dest, flags) => new Promise((resolve, reject) => {
    copyFile(src, dest, flags, err => err ? reject(err) : resolve());
  }),
  readlink: () => Promise.reject(new Error('readlink not implemented')),
  symlink: () => Promise.reject(new Error('symlink not implemented')),
  lstat: (path) => promises.stat(path),
  realpath: (path) => Promise.resolve(path),
};

const fs = {
  readFileSync,
  writeFileSync,
  existsSync,
  readdirSync,
  mkdirSync,
  statSync,
  lstatSync: statSync,
  unlinkSync,
  rmdirSync,
  renameSync,
  copyFileSync,
  appendFileSync,
  readFile,
  writeFile,
  readdir,
  mkdir,
  stat,
  lstat,
  unlink,
  rmdir,
  rename,
  access,
  accessSync,
  appendFile,
  copyFile,
  createReadStream,
  createWriteStream,
  promises,
  Stats,
  constants: {
    F_OK: 0, R_OK: 4, W_OK: 2, X_OK: 1,
    O_RDONLY: 0, O_WRONLY: 1, O_RDWR: 2,
    O_CREAT: 64, O_EXCL: 128, O_TRUNC: 512,
    O_APPEND: 1024, O_DIRECTORY: 65536,
    S_IFMT: 0o170000, S_IFREG: 0o100000, S_IFDIR: 0o040000,
    S_IRUSR: 0o400, S_IWUSR: 0o200, S_IXUSR: 0o100,
  },
  promises: {
    [Symbol.toStringTag]: 'PromiseFs',
    ...promises,
  },
};

if (typeof module !== 'undefined' && module.exports) {
  module.exports = fs;
}
