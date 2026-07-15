// Klyron Runtime — node:vm polyfill

class Script {
  constructor(code, options = {}) {
    if (typeof code !== 'string') throw new TypeError('code must be a string');
    this._code = code;
    this._filename = options.filename || options.filename || 'evalmachine.<anonymous>';
    this._lineOffset = options.lineOffset || 0;
    this._columnOffset = options.columnOffset || 0;
    this._cachedData = options.cachedData || null;
    this._produceCachedData = !!options.produceCachedData;
    this._parsed = false;
    this._cachedDataProduced = null;
    try {
      this._compiled = new Function(this._code);
      this._parsed = true;
      if (this._produceCachedData) {
        this._cachedDataProduced = new Uint8Array(0);
      }
    } catch (e) {
      this._parseError = e;
    }
  }

  runInContext(contextifiedSandbox, options = {}) {
    if (this._parseError) throw this._parseError;
    const timeout = options.timeout || 0;
    const breakOnSigint = options.breakOnSigint || false;
    const ctx = createContext(contextifiedSandbox);
    const keys = Object.keys(ctx);
    const vals = keys.map(k => ctx[k]);
    const fn = new Function('require', ...keys, this._code);
    return fn.call(ctx, createRequire(this._filename), ...vals);
  }

  runInNewContext(sandbox = {}, options = {}) {
    const ctx = createContext(sandbox);
    return this.runInContext(ctx, options);
  }

  runInThisContext(options = {}) {
    if (this._parseError) throw this._parseError;
    const displayErrors = options.displayErrors !== false;
    try {
      if (displayErrors) {
        return this._compiled();
      }
      return this._compiled();
    } catch (e) {
      if (displayErrors) {
        const line = (e.stack || '').split('\n')[0] || '';
        throw new Error(`SyntaxError: ${e.message} (${this._filename})`);
      }
      throw e;
    }
  }

  createCachedData() {
    return this._cachedDataProduced || new Uint8Array(0);
  }
}

const _contexts = new WeakMap();

function createContext(sandbox = {}) {
  if (_contexts.has(sandbox)) return sandbox;
  const ctx = Object.assign(Object.create(null), {
    global: sandbox,
    console: globalThis.console,
    process: globalThis.process,
    Buffer: globalThis.Buffer,
    setTimeout: globalThis.setTimeout,
    setInterval: globalThis.setInterval,
    setImmediate: globalThis.setImmediate,
    clearTimeout: globalThis.clearTimeout,
    clearInterval: globalThis.clearInterval,
    clearImmediate: globalThis.clearImmediate,
    queueMicrotask: globalThis.queueMicrotask,
    URL: globalThis.URL,
    URLSearchParams: globalThis.URLSearchParams,
    TextEncoder: globalThis.TextEncoder,
    TextDecoder: globalThis.TextDecoder,
    Array: globalThis.Array,
    Object: globalThis.Object,
    Function: globalThis.Function,
    String: globalThis.String,
    Number: globalThis.Number,
    Boolean: globalThis.Boolean,
    Symbol: globalThis.Symbol,
    Map: globalThis.Map,
    Set: globalThis.Set,
    WeakMap: globalThis.WeakMap,
    WeakSet: globalThis.WeakSet,
    Promise: globalThis.Promise,
    RegExp: globalThis.RegExp,
    Date: globalThis.Date,
    Error: globalThis.Error,
    TypeError: globalThis.TypeError,
    RangeError: globalThis.RangeError,
    SyntaxError: globalThis.SyntaxError,
    ReferenceError: globalThis.ReferenceError,
    EvalError: globalThis.EvalError,
    URIError: globalThis.URIError,
    JSON: globalThis.JSON,
    Math: globalThis.Math,
    parseInt: globalThis.parseInt,
    parseFloat: globalThis.parseFloat,
    isNaN: globalThis.isNaN,
    isFinite: globalThis.isFinite,
    eval: globalThis.eval,
    require: globalThis.require,
    performance: globalThis.performance,
    crypto: globalThis.crypto,
    fetch: globalThis.fetch,
  }, sandbox);
  ctx.global = ctx;
  _contexts.set(ctx, true);
  if (sandbox) {
    for (const key of Object.keys(sandbox)) {
      ctx[key] = sandbox[key];
    }
  }
  return ctx;
}

function runInContext(code, contextifiedSandbox, options = {}) {
  const script = new Script(code, options);
  return script.runInContext(contextifiedSandbox, options);
}

function runInNewContext(code, sandbox = {}, options = {}) {
  const ctx = createContext(sandbox);
  const script = new Script(code, options);
  return script.runInContext(ctx, options);
}

function runInThisContext(code, options = {}) {
  const script = new Script(code, options);
  return script.runInThisContext(options);
}

function createRequire(filename) {
  if (typeof globalThis.require === 'function') {
    const fn = (specifier) => globalThis.require(specifier);
    fn.resolve = (specifier) => specifier;
    fn.cache = globalThis.require.cache || new Map();
    fn.extensions = globalThis.require.extensions || {};
    fn.main = globalThis.require.main;
    return fn;
  }
  return (specifier) => {
    throw new Error(`Cannot find module '${specifier}'`);
  };
}

function compileFunction(code, params = [], options = {}) {
  const paramNames = params.map(p => {
    if (typeof p === 'string') return p.replace(/[^a-zA-Z0-9_$]/g, '');
    if (typeof p === 'object' && p.parameter) return String(p.parameter).replace(/[^a-zA-Z0-9_$]/g, '');
    return '';
  });
  const parsingContext = options.parsingContext || undefined;
  const contextExtensions = options.contextExtensions || [];
  const produceCachedData = options.produceCachedData || false;
  const cachedData = options.cachedData || null;
  try {
    const fn = new Function(...paramNames, code);
    fn.cachedData = produceCachedData ? new Uint8Array(0) : null;
    return fn;
  } catch (e) {
    throw new SyntaxError(e.message);
  }
}

function measureMemory() {
  return Promise.resolve({
    total: { jsMemoryEstimate: 0, jsMemoryRange: [0, 0] },
  });
}

function isContext(sandbox) {
  return _contexts.has(sandbox);
}

const vm = {
  Script,
  createContext,
  runInContext,
  runInNewContext,
  runInThisContext,
  createRequire,
  compileFunction,
  measureMemory,
  isContext,
};

if (typeof module !== 'undefined' && module.exports) {
  module.exports = vm;
}
