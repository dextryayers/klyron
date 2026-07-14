// Klyron Runtime — node:util polyfill

function inspect(obj, opts = {}) {
  const depth = opts.depth === undefined ? 2 : opts.depth;
  const colors = opts.colors || false;
  const maxArrayLength = opts.maxArrayLength || 100;
  const showHidden = opts.showHidden || false;
  const compact = opts.compact !== false;

  function format(val, d, seen) {
    if (seen.has(val)) return '[Circular]';
    if (val === null) return 'null';
    if (val === undefined) return 'undefined';
    if (typeof val === 'boolean' || typeof val === 'number') return String(val);
    if (typeof val === 'string') return colors ? `\x1b[32m'${val}'\x1b[39m` : `'${val}'`;
    if (typeof val === 'function') return `[Function: ${val.name || 'anonymous'}]`;
    if (typeof val === 'symbol') return val.toString();
    if (val instanceof Date) return val.toISOString();
    if (val instanceof RegExp) return val.toString();
    if (val instanceof Error) return val.stack || val.message;
    if (val instanceof ArrayBuffer || ArrayBuffer.isView(val)) {
      const arr = new Uint8Array(val.buffer || val);
      return `Uint8Array(${arr.length}) [${Array.from(arr.slice(0, 10)).join(', ')}${arr.length > 10 ? ', ...' : ''}]`;
    }
    if (d <= 0) return typeof val === 'object' ? (Array.isArray(val) ? `[Array(${val.length})]` : '[Object]') : String(val);
    seen.add(val);
    try {
      if (Array.isArray(val)) {
        const items = val.slice(0, maxArrayLength).map(v => format(v, d - 1, seen));
        if (val.length > maxArrayLength) items.push(`... ${val.length - maxArrayLength} more items`);
        const open = compact ? '[' : '[\n';
        const close = compact ? ']' : '\n]';
        const sep = compact ? ', ' : ',\n';
        return open + items.join(sep) + close;
      }
      const keys = Object.keys(val);
      const entries = keys.map(k => `${k}: ${format(val[k], d - 1, seen)}`);
      const open = compact ? '{ ' : '{\n';
      const close = compact ? ' }' : '\n}';
      const sep = compact ? ', ' : ',\n';
      return open + entries.join(sep) + close;
    } finally {
      seen.delete(val);
    }
  }
  return format(obj, depth, new Set());
}

function format(f, ...args) {
  if (typeof f !== 'string') {
    return args.map(a => inspect(a)).join(' ');
  }
  let i = 0;
  return f.replace(/%[sdifoOc]/g, match => {
    if (i >= args.length) return match;
    const val = args[i++];
    switch (match) {
      case '%s': return String(val);
      case '%d':
      case '%i': return Number(val).toString();
      case '%f': return Number(val).toFixed(6);
      case '%o':
      case '%O': return inspect(val, { depth: Infinity });
      case '%c': return '';
      default: return match;
    }
  }) + (i < args.length ? ' ' + args.slice(i).map(a => inspect(a)).join(' ') : '');
}

function deprecate(fn, msg, code) {
  const warned = new Set();
  const deprecated = function(...args) {
    if (!warned.has(msg)) {
      warned.add(msg);
      if (typeof process !== 'undefined' && process.emitWarning) {
        process.emitWarning(msg, 'DeprecationWarning', code);
      }
    }
    return fn.apply(this, args);
  };
  return deprecated;
}

function callbackify(fn) {
  return function(...args) {
    const callback = args.pop();
    if (typeof callback !== 'function') throw new TypeError('Last argument must be a callback');
    fn.apply(this, args).then(
      result => callback(null, result),
      err => callback(err)
    );
  };
}

function promisify(fn) {
  if (fn[promisify.custom]) return fn[promisify.custom];
  return function(...args) {
    return new Promise((resolve, reject) => {
      fn.call(this, ...args, (err, ...results) => {
        if (err) return reject(err);
        if (results.length <= 1) resolve(results[0]);
        else resolve(results);
      });
    });
  };
}
promisify.custom = Symbol('util.promisify.custom');

function inherits(ctor, superCtor) {
  if (superCtor) {
    ctor.super_ = superCtor;
    ctor.prototype = Object.create(superCtor.prototype, {
      constructor: { value: ctor, writable: true, configurable: true },
    });
  }
}

function isArray(arr) { return Array.isArray(arr); }
function isBoolean(b) { return typeof b === 'boolean'; }
function isNull(v) { return v === null; }
function isNullOrUndefined(v) { return v === null || v === undefined; }
function isNumber(n) { return typeof n === 'number' && !isNaN(n); }
function isString(s) { return typeof s === 'string'; }
function isSymbol(s) { return typeof s === 'symbol'; }
function isUndefined(v) { return v === undefined; }
function isRegExp(re) { return re instanceof RegExp; }
function isObject(v) { return v !== null && typeof v === 'object'; }
function isDate(d) { return d instanceof Date; }
function isError(e) { return e instanceof Error; }
function isFunction(f) { return typeof f === 'function'; }
function isPrimitive(v) { return v === null || ['string','number','boolean','symbol','undefined','bigint'].includes(typeof v); }
function isBuffer(b) { return Buffer && Buffer.isBuffer(b); }

function toUSVString(str) {
  return typeof str === 'string' ? str : String(str);
}

function getSystemErrorName(errno) {
  const names = {
    1: 'EPERM', 2: 'ENOENT', 3: 'ESRCH', 4: 'EINTR', 5: 'EIO', 6: 'ENXIO',
    7: 'E2BIG', 8: 'ENOEXEC', 9: 'EBADF', 10: 'ECHILD', 11: 'EAGAIN',
    12: 'ENOMEM', 13: 'EACCES', 14: 'EFAULT', 15: 'ENOTBLK', 16: 'EBUSY',
    17: 'EEXIST', 18: 'EXDEV', 19: 'ENODEV', 20: 'ENOTDIR', 21: 'EISDIR',
    22: 'EINVAL', 23: 'ENFILE', 24: 'EMFILE', 25: 'ENOTTY', 26: 'ETXTBSY',
    27: 'EFBIG', 28: 'ENOSPC', 29: 'ESPIPE', 30: 'EROFS', 31: 'EMLINK',
    32: 'EPIPE', 33: 'EDOM', 34: 'ERANGE',
  };
  return names[errno] || 'UNKNOWN';
}

function getSystemErrorMap() {
  return new Map();
}

const types = {
  isArray, isBoolean, isNull, isNullOrUndefined, isNumber, isString,
  isSymbol, isUndefined, isRegExp, isObject, isDate, isError, isFunction,
  isPrimitive, isBuffer,
};

const util = {
  inspect,
  format,
  deprecate,
  callbackify,
  promisify,
  inherits,
  isArray, isBoolean, isNull, isNullOrUndefined, isNumber, isString,
  isSymbol, isUndefined, isRegExp, isObject, isDate, isError, isFunction,
  isPrimitive, isBuffer,
  toUSVString,
  getSystemErrorName,
  getSystemErrorMap,
  types,
  TextDecoder: globalThis.TextDecoder,
  TextEncoder: globalThis.TextEncoder,
  types: {
    isNativeError: isError,
    ...types,
  },
  debuglog: (section) => {
    return (...args) => {};
  },
  debug: () => {},
  log: () => {},
  inspect: {
    defaultOptions: {},
    custom: Symbol('util.inspect.custom'),
    stylizeWithColor: (str, type) => str,
    stylizeNoColor: (str) => str,
    replacer: null,
    sorted: false,
  },
  stripVTControlCharacters: (str) => str.replace(/\x1B\[[0-9;]*m/g, ''),
};

if (typeof module !== 'undefined' && module.exports) {
  module.exports = util;
}
