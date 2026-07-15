// Klyron Runtime — node:util polyfill

const customInspectSymbol = Symbol('util.inspect.custom');

function inspect(obj, opts = {}) {
  const depth = opts.depth === undefined ? 2 : opts.depth;
  const colors = opts.colors || false;
  const maxArrayLength = opts.maxArrayLength || 100;
  const maxStringLength = opts.maxStringLength || 10000;
  const showHidden = opts.showHidden || false;
  const compact = opts.compact !== false;
  const sorted = opts.sorted || false;

  function format(val, d, seen) {
    if (val === null) return 'null';
    if (val === undefined) return 'undefined';
    if (typeof val === 'boolean' || typeof val === 'number') return String(val);
    if (typeof val === 'bigint') return String(val) + 'n';
    if (typeof val === 'string') {
      if (val.length > maxStringLength) val = val.slice(0, maxStringLength) + '...';
      return colors ? `\x1b[32m'${val}'\x1b[39m` : `'${val}'`;
    }
    if (typeof val === 'function') {
      return colors ? `\x1b[33m[Function: ${val.name || 'anonymous'}]\x1b[39m` : `[Function: ${val.name || 'anonymous'}]`;
    }
    if (typeof val === 'symbol') return val.toString();
    if (seen.has(val)) return '[Circular]';
    if (val instanceof Date) return val.toISOString();
    if (val instanceof RegExp) return val.toString();
    if (val instanceof Error) return val.stack || val.message;
    if (val instanceof ArrayBuffer || ArrayBuffer.isView(val)) {
      const arr = new Uint8Array(val.buffer || val);
      const prefix = colors ? `\x1b[36mUint8Array(${arr.length}) \x1b[39m` : `Uint8Array(${arr.length}) `;
      return prefix + '[' + Array.from(arr.slice(0, Math.min(20, arr.length))).join(', ') + (arr.length > 20 ? ', ...' : '') + ']';
    }
    if (val instanceof Map) {
      if (d <= 0) return '[Map]';
      seen.add(val);
      const entries = Array.from(val.entries()).map(([k, v]) => `${format(k, d - 1, seen)} => ${format(v, d - 1, seen)}`);
      seen.delete(val);
      return `Map(${val.size}) { ${entries.join(', ')} }`;
    }
    if (val instanceof Set) {
      if (d <= 0) return '[Set]';
      seen.add(val);
      const items = Array.from(val).map(v => format(v, d - 1, seen));
      seen.delete(val);
      return `Set(${val.size}) { ${items.join(', ')} }`;
    }
    if (d <= 0) return Array.isArray(val) ? `[Array(${val.length})]` : '[Object]';
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
      let keys = showHidden ? Object.getOwnPropertyNames(val) : Object.keys(val);
      if (typeof sorted === 'function') keys = keys.sort(sorted);
      else if (sorted) keys = keys.sort();
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

inspect.custom = customInspectSymbol;
inspect.defaultOptions = {};
inspect.replacer = null;
inspect.stylizeWithColor = (str, styleType) => str;
inspect.stylizeNoColor = (str) => str;

function format(f, ...args) {
  if (typeof f !== 'string') {
    return [f, ...args].map(a => inspect(a)).join(' ');
  }
  let i = 0;
  let result = f.replace(/%[sdifoOc%]/g, match => {
    if (match === '%%') return '%';
    if (i >= args.length) return match;
    const val = args[i++];
    switch (match) {
      case '%s': return String(val);
      case '%d':
      case '%i': return Number(val).toString();
      case '%f': return Number(val).toFixed(6);
      case '%o':
      case '%O': return inspect(val, { depth: Infinity, colors: false });
      case '%c': return '';
      default: return match;
    }
  });
  if (i < args.length) {
    result += ' ' + args.slice(i).map(a => inspect(a)).join(' ');
  }
  return result;
}

function deprecate(fn, msg, code) {
  const warned = new Set();
  const deprecated = function(...args) {
    if (!warned.has(msg)) {
      warned.add(msg);
      if (typeof process !== 'undefined' && process.emitWarning) {
        process.emitWarning(msg, 'DeprecationWarning', code);
      } else {
        console.warn(`DeprecationWarning: ${msg}`);
      }
    }
    return fn.apply(this, args);
  };
  deprecated.toString = () => fn.toString();
  return deprecated;
}

function callbackify(fn) {
  return function(...args) {
    const callback = args.pop();
    if (typeof callback !== 'function') throw new TypeError('The last argument must be of type function');
    fn.apply(this, args).then(
      result => queueMicrotask(() => callback(null, result)),
      err => queueMicrotask(() => callback(err))
    );
  };
}

function promisify(original) {
  if (typeof original !== 'function') throw new TypeError('The "original" argument must be of type function');
  function fn(...args) {
    return new Promise((resolve, reject) => {
      try {
        original.call(this, ...args, (err, ...values) => {
          if (err) return reject(err);
          if (values.length <= 1) resolve(values[0]);
          else resolve(values);
        });
      } catch (e) {
        reject(e);
      }
    });
  }
  fn.__proto__ = original;
  Object.defineProperty(fn, 'name', { value: original.name ? `promisified(${original.name})` : 'promisified' });
  return fn;
}
promisify.custom = Symbol('util.promisify.custom');

function inherits(ctor, superCtor) {
  if (superCtor) {
    ctor.super_ = superCtor;
    ctor.prototype = Object.create(superCtor.prototype, {
      constructor: { value: ctor, writable: true, configurable: true, enumerable: false },
    });
  }
}

function debuglog(section) {
  const debugEnv = (typeof process !== 'undefined' && process.env && process.env.NODE_DEBUG) || '';
  const sections = debugEnv.split(',').map(s => s.trim().toLowerCase());
  const enabled = sections.includes(section.toLowerCase()) || sections.includes('*');
  return function(...args) {
    if (enabled) {
      const msg = args.map(a => typeof a === 'string' ? a : inspect(a)).join(' ');
      const timestamp = new Date().toISOString();
      process.stderr.write(`${timestamp} [${section}] ${msg}\n`);
    }
  };
}

function getSystemErrorName(errno) {
  const names = {
    -1: 'EPERM', -2: 'ENOENT', -3: 'ESRCH', -4: 'EINTR', -5: 'EIO', -6: 'ENXIO',
    -7: 'E2BIG', -8: 'ENOEXEC', -9: 'EBADF', -10: 'ECHILD', -11: 'EAGAIN',
    -12: 'ENOMEM', -13: 'EACCES', -14: 'EFAULT', -15: 'ENOTBLK', -16: 'EBUSY',
    -17: 'EEXIST', -18: 'EXDEV', -19: 'ENODEV', -20: 'ENOTDIR', -21: 'EISDIR',
    -22: 'EINVAL', -23: 'ENFILE', -24: 'EMFILE', -25: 'ENOTTY', -26: 'ETXTBSY',
    -27: 'EFBIG', -28: 'ENOSPC', -29: 'ESPIPE', -30: 'EROFS', -31: 'EMLINK',
    -32: 'EPIPE', -33: 'EDOM', -34: 'ERANGE', -35: 'EDEADLK', -36: 'ENAMETOOLONG',
    -37: 'ENOLCK', -38: 'ENOSYS', -39: 'ENOTEMPTY', -40: 'ELOOP', -42: 'ENOMSG',
    -43: 'EIDRM', -44: 'ECHRNG', -45: 'EL2NSYNC', -46: 'EL3HLT', -47: 'EL3RST',
    -48: 'ELNRNG', -49: 'EUNATCH', -50: 'ENOCSI', -51: 'EL2HLT', -52: 'EBADE',
    -53: 'EBADR', -54: 'EXFULL', -55: 'ENOANO', -56: 'EBADRQC', -57: 'EBADSLT',
    -59: 'EBFONT', -60: 'ENOSTR', -61: 'ENODATA', -62: 'ETIME', -63: 'ENOSR',
    -64: 'ENONET', -65: 'ENOPKG', -66: 'EREMOTE', -67: 'ENOLINK', -68: 'EADV',
    -69: 'ESRMNT', -70: 'ECOMM', -71: 'EPROTO', -72: 'EMULTIHOP', -73: 'EDOTDOT',
    -74: 'EBADMSG', -75: 'EOVERFLOW', -76: 'ENOTUNIQ', -77: 'EBADFD', -78: 'EREMCHG',
    -79: 'ELIBACC', -80: 'ELIBBAD', -81: 'ELIBSCN', -82: 'ELIBMAX', -83: 'ELIBEXEC',
    -84: 'EILSEQ', -85: 'ERESTART', -86: 'ESTRPIPE', -87: 'EUSERS', -88: 'ENOTSOCK',
    -89: 'EDESTADDRREQ', -90: 'EMSGSIZE', -91: 'EPROTOTYPE', -92: 'ENOPROTOOPT',
    -93: 'EPROTONOSUPPORT', -94: 'ESOCKTNOSUPPORT', -95: 'EOPNOTSUPP',
    -96: 'EPFNOSUPPORT', -97: 'EAFNOSUPPORT', -98: 'EADDRINUSE', -99: 'EADDRNOTAVAIL',
    -100: 'ENETDOWN', -101: 'ENETUNREACH', -102: 'ENETRESET', -103: 'ECONNABORTED',
    -104: 'ECONNRESET', -105: 'ENOBUFS', -106: 'EISCONN', -107: 'ENOTCONN',
    -108: 'ESHUTDOWN', -109: 'ETOOMANYREFS', -110: 'ETIMEDOUT', -111: 'ECONNREFUSED',
    -112: 'EHOSTDOWN', -113: 'EHOSTUNREACH', -114: 'EALREADY', -115: 'EINPROGRESS',
    -116: 'ESTALE', -117: 'EUCLEAN', -118: 'ENOTNAM', -119: 'ENAVAIL', -120: 'EISNAM',
    -121: 'EREMOTEIO', -122: 'EDQUOT', -123: 'ENOMEDIUM', -124: 'EMEDIUMTYPE',
    -125: 'ECANCELED', -126: 'ENOKEY', -127: 'EKEYEXPIRED', -128: 'EKEYREVOKED',
    -129: 'EKEYREJECTED', -130: 'EOWNERDEAD', -131: 'ENOTRECOVERABLE',
    -132: 'ERFKILL', -133: 'EHWPOISON',
  };
  return names[errno] || `UNKNOWN (${errno})`;
}

const types = {
  isNumber(v) { return typeof v === 'number' && !isNaN(v); },
  isString(v) { return typeof v === 'string'; },
  isPromise(v) { return v instanceof Promise; },
  isProxy(v) { return false; },
  isDate(v) { return v instanceof Date; },
  isMap(v) { return v instanceof Map; },
  isSet(v) { return v instanceof Set; },
  isArrayBuffer(v) { return v instanceof ArrayBuffer; },
  isTypedArray(v) { return ArrayBuffer.isView(v) && !(v instanceof DataView); },
  isRegExp(v) { return v instanceof RegExp; },
  isError(v) { return v instanceof Error; },
  isNativeError(v) { return v instanceof Error; },
  isArray(v) { return Array.isArray(v); },
  isBoolean(v) { return typeof v === 'boolean'; },
  isNull(v) { return v === null; },
  isNullOrUndefined(v) { return v === null || v === undefined; },
  isSymbol(v) { return typeof v === 'symbol'; },
  isUndefined(v) { return v === undefined; },
  isObject(v) { return v !== null && typeof v === 'object'; },
  isFunction(v) { return typeof v === 'function'; },
  isPrimitive(v) { return v === null || ['string', 'number', 'boolean', 'symbol', 'undefined', 'bigint'].includes(typeof v); },
  isBuffer(v) { return globalThis.Buffer && globalThis.Buffer.isBuffer(v); },
  isDataView(v) { return v instanceof DataView; },
  isSharedArrayBuffer(v) { return v instanceof SharedArrayBuffer; },
  isWeakMap(v) { return v instanceof WeakMap; },
  isWeakSet(v) { return v instanceof WeakSet; },
  isInt8Array(v) { return v instanceof Int8Array; },
  isUint8Array(v) { return v instanceof Uint8Array; },
  isUint8ClampedArray(v) { return v instanceof Uint8ClampedArray; },
  isInt16Array(v) { return v instanceof Int16Array; },
  isUint16Array(v) { return v instanceof Uint16Array; },
  isInt32Array(v) { return v instanceof Int32Array; },
  isUint32Array(v) { return v instanceof Uint32Array; },
  isFloat32Array(v) { return v instanceof Float32Array; },
  isFloat64Array(v) { return v instanceof Float64Array; },
  isBigInt64Array(v) { return v instanceof BigInt64Array; },
  isBigUint64Array(v) { return v instanceof BigUint64Array; },
  isBoxedPrimitive(v) {
    if (v === null || typeof v !== 'object') return false;
    const proto = Object.getPrototypeOf(v);
    return proto === Boolean.prototype || proto === Number.prototype ||
           proto === String.prototype || proto === Symbol.prototype || proto === BigInt.prototype;
  },
  isAnyArrayBuffer(v) { return v instanceof ArrayBuffer || v instanceof SharedArrayBuffer; },
  isArgumentsObject(v) { return v !== null && typeof v === 'object' && v[Symbol.toStringTag] === 'Arguments'; },
  isGeneratorObject(v) { return v !== null && typeof v === 'object' && v[Symbol.toStringTag] === 'GeneratorFunction'; },
  isAsyncFunction(v) { return typeof v === 'function' && v[Symbol.toStringTag] === 'AsyncFunction'; },
  isGeneratorFunction(v) { return typeof v === 'function' && v[Symbol.toStringTag] === 'GeneratorFunction'; },
  isMapIterator(v) { return v !== null && typeof v === 'object' && v[Symbol.toStringTag] === 'Map Iterator'; },
  isSetIterator(v) { return v !== null && typeof v === 'object' && v[Symbol.toStringTag] === 'Set Iterator'; },
  isModuleNamespaceObject(v) { return v !== null && typeof v === 'object' && v[Symbol.toStringTag] === 'Module'; },
  isExternal(v) { return false; },
  isNumberObject(v) { return v instanceof Number; },
  isStringObject(v) { return v instanceof String; },
  isBooleanObject(v) { return v instanceof Boolean; },
  isBigIntObject(v) { return v instanceof BigInt; },
  isSymbolObject(v) { return v instanceof Symbol; },
};

function toUSVString(str) {
  return typeof str === 'string' ? str : String(str);
}

function getSystemErrorMap() {
  return new Map(Object.entries(getSystemErrorName).map(([k, v]) => [Number(k), v]));
}

function stripVTControlCharacters(str) {
  return String(str).replace(/\x1B\[[0-9;]*m/g, '');
}

function isDeepStrictEqual(a, b) {
  if (a === b) return true;
  if (typeof a !== typeof b) return false;
  if (typeof a !== 'object' || a === null || b === null) return a === b;
  if (a instanceof Date && b instanceof Date) return a.getTime() === b.getTime();
  if (a instanceof RegExp && b instanceof RegExp) return a.toString() === b.toString();
  const aKeys = Object.keys(a);
  const bKeys = Object.keys(b);
  if (aKeys.length !== bKeys.length) return false;
  aKeys.sort();
  bKeys.sort();
  for (let i = 0; i < aKeys.length; i++) {
    if (aKeys[i] !== bKeys[i]) return false;
    if (!isDeepStrictEqual(a[aKeys[i]], b[bKeys[i]])) return false;
  }
  return true;
}

const util = {
  inspect,
  format,
  deprecate,
  callbackify,
  promisify,
  inherits,
  debuglog,
  getSystemErrorName,
  getSystemErrorMap,
  toUSVString,
  stripVTControlCharacters,
  isDeepStrictEqual,
  types,
  TextDecoder: globalThis.TextDecoder,
  TextEncoder: globalThis.TextEncoder,
  isArray: types.isArray,
  isBoolean: types.isBoolean,
  isNull: types.isNull,
  isNullOrUndefined: types.isNullOrUndefined,
  isNumber: types.isNumber,
  isString: types.isString,
  isSymbol: types.isSymbol,
  isUndefined: types.isUndefined,
  isRegExp: types.isRegExp,
  isObject: types.isObject,
  isDate: types.isDate,
  isError: types.isError,
  isFunction: types.isFunction,
  isPrimitive: types.isPrimitive,
  isBuffer: types.isBuffer,
  debug: debuglog('util'),
  log: (...args) => { process.stdout.write(args.map(a => inspect(a)).join(' ') + '\n'); },
};

if (typeof module !== 'undefined' && module.exports) {
  module.exports = util;
}
