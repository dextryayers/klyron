// Klyron Runtime — node:wasi polyfill

class WASI {
  constructor(options = {}) {
    this._options = {
      args: options.args || [],
      env: options.env || {},
      preopens: options.preopens || {},
      returnOnExit: !!options.returnOnExit,
      stdin: options.stdin || 0,
      stdout: options.stdout || 1,
      stderr: options.stderr || 2,
    };
    this._started = false;
    this._instance = null;
    this._wasiImport = this._createWasiImport();
  }

  _createWasiImport() {
    const wasi = this;
    const fdMap = new Map();
    let nextFd = 3;
    for (const [path, realPath] of Object.entries(this._options.preopens)) {
      fdMap.set(nextFd++, { path, realPath, type: 'dir' });
    }
    fdMap.set(0, { fd: 0, type: 'char' });
    fdMap.set(1, { fd: 1, type: 'char' });
    fdMap.set(2, { fd: 2, type: 'char' });

    function encodeString(str) {
      const encoder = new TextEncoder();
      return encoder.encode(str + '\0');
    }

    return {
      args_sizes_get(argcPtr, bufSizePtr) {
        const args = wasi._options.args;
        const memory = wasi._instance.exports.memory;
        const view = new DataView(memory.buffer);
        view.setUint32(argcPtr, args.length, true);
        const totalSize = args.reduce((s, a) => s + encodeString(a).length, 0);
        view.setUint32(bufSizePtr, totalSize, true);
        return 0;
      },

      args_get(argvPtr, argvBufPtr) {
        const args = wasi._options.args;
        const memory = wasi._instance.exports.memory;
        const view = new DataView(memory.buffer);
        let bufOffset = argvBufPtr;
        for (let i = 0; i < args.length; i++) {
          view.setUint32(argvPtr + i * 4, bufOffset, true);
          const encoded = encodeString(args[i]);
          const arr = new Uint8Array(memory.buffer);
          arr.set(encoded, bufOffset);
          bufOffset += encoded.length;
        }
        return 0;
      },

      environ_sizes_exit(environcPtr, bufSizePtr) {
        const env = wasi._options.env;
        const envEntries = typeof env === 'object' && env !== null
          ? Object.entries(env).map(([k, v]) => `${k}=${v}`)
          : [];
        const memory = wasi._instance.exports.memory;
        const view = new DataView(memory.buffer);
        view.setUint32(environcPtr, envEntries.length, true);
        const totalSize = envEntries.reduce((s, e) => s + encodeString(e).length, 0);
        view.setUint32(bufSizePtr, totalSize, true);
        return 0;
      },

      environ_get(environPtr, environBufPtr) {
        const env = wasi._options.env;
        const envEntries = typeof env === 'object' && env !== null
          ? Object.entries(env).map(([k, v]) => `${k}=${v}`)
          : [];
        const memory = wasi._instance.exports.memory;
        const view = new DataView(memory.buffer);
        let bufOffset = environBufPtr;
        for (let i = 0; i < envEntries.length; i++) {
          view.setUint32(environPtr + i * 4, bufOffset, true);
          const encoded = encodeString(envEntries[i]);
          const arr = new Uint8Array(memory.buffer);
          arr.set(encoded, bufOffset);
          bufOffset += encoded.length;
        }
        return 0;
      },

      fd_prestat_get(fd, bufPtr) {
        const entry = fdMap.get(fd);
        if (!entry || entry.type !== 'dir') return 8;
        const memory = wasi._instance.exports.memory;
        const view = new DataView(memory.buffer);
        view.setUint8(bufPtr, 0);
        view.setUint32(bufPtr + 4, encodeString(entry.realPath).length, true);
        return 0;
      },

      fd_prestat_dir_name(fd, pathPtr, pathLen) {
        const entry = fdMap.get(fd);
        if (!entry || entry.type !== 'dir') return 8;
        const encoded = encodeString(entry.realPath);
        const memory = wasi._instance.exports.memory;
        const arr = new Uint8Array(memory.buffer);
        if (encoded.length <= pathLen) {
          arr.set(encoded, pathPtr);
        }
        return 0;
      },

      fd_read(fd, iovsPtr, iovsLen, nreadPtr) {
        if (fd === 0) {
          const memory = wasi._instance.exports.memory;
          const view = new DataView(memory.buffer);
          let totalRead = 0;
          for (let i = 0; i < iovsLen; i++) {
            const bufPtr = view.getUint32(iovsPtr + i * 8, true);
            const bufLen = view.getUint32(iovsPtr + i * 8 + 4, true);
            const arr = new Uint8Array(memory.buffer);
            for (let j = 0; j < Math.min(bufLen, 1); j++) {
              arr[bufPtr + j] = 0;
            }
          }
          view.setUint32(nreadPtr, totalRead, true);
          return 0;
        }
        return 8;
      },

      fd_write(fd, iovsPtr, iovsLen, nwrittenPtr) {
        if (fd === 1 || fd === 2) {
          const memory = wasi._instance.exports.memory;
          const view = new DataView(memory.buffer);
          let totalWritten = 0;
          for (let i = 0; i < iovsLen; i++) {
            const bufPtr = view.getUint32(iovsPtr + i * 8, true);
            const bufLen = view.getUint32(iovsPtr + i * 8 + 4, true);
            const arr = new Uint8Array(memory.buffer, bufPtr, bufLen);
            const str = new TextDecoder().decode(arr);
            if (fd === 1 && typeof process !== 'undefined' && process.stdout) {
              process.stdout.write(str);
            } else if (typeof process !== 'undefined' && process.stderr) {
              process.stderr.write(str);
            }
            totalWritten += bufLen;
          }
          view.setUint32(nwrittenPtr, totalWritten, true);
          return 0;
        }
        return 8;
      },

      fd_close(fd) {
        fdMap.delete(fd);
        return 0;
      },

      fd_seek(fd, offset, whence, newoffsetPtr) {
        const memory = wasi._instance.exports.memory;
        const view = new DataView(memory.buffer);
        if (newoffsetPtr) view.setUint64(newoffsetPtr, BigInt(0), true);
        return 0;
      },

      proc_exit(code) {
        if (wasi._options.returnOnExit) {
          throw new Error(`WASI exit with code ${code}`);
        }
        if (typeof process !== 'undefined' && process.exit) {
          process.exit(code);
        }
      },

      clock_time_get(id, precision, timePtr) {
        const memory = wasi._instance.exports.memory;
        const view = new DataView(memory.buffer);
        const now = BigInt(Date.now()) * BigInt(1000000);
        view.setBigUint64(timePtr, now, true);
        return 0;
      },

      clock_res_get(id, resolutionPtr) {
        const memory = wasi._instance.exports.memory;
        const view = new DataView(memory.buffer);
        view.setBigUint64(resolutionPtr, BigInt(1000000), true);
        return 0;
      },

      random_get(bufPtr, bufLen) {
        const memory = wasi._instance.exports.memory;
        const arr = new Uint8Array(memory.buffer, bufPtr, bufLen);
        for (let i = 0; i < bufLen; i++) {
          arr[i] = Math.floor(Math.random() * 256);
        }
        return 0;
      },
    };
  }

  start(instance) {
    if (this._started) throw new Error('WASI instance already started');
    this._instance = instance;
    this._started = true;
    if (instance.exports._start) {
      try {
        instance.exports._start();
      } catch (e) {
        if (this._options.returnOnExit && e.message && e.message.includes('WASI exit')) {
          return;
        }
        throw e;
      }
    }
  }

  initialize(instance) {
    if (this._started) throw new Error('WASI instance already initialized');
    this._instance = instance;
    this._started = true;
  }

  get wasiImport() {
    return this._wasiImport;
  }

  get started() { return this._started; }
}

const wasi = {
  WASI,
};

if (typeof module !== 'undefined' && module.exports) {
  module.exports = wasi;
}
