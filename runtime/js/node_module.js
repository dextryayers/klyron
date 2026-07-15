// Klyron Runtime — node:module polyfill

const builtinModulesList = [
  'assert', 'async_hooks', 'buffer', 'child_process', 'cluster', 'console',
  'constants', 'crypto', 'diagnostics_channel', 'dns', 'domain', 'events',
  'fs', 'http', 'http2', 'https', 'inspector', 'module', 'net', 'os',
  'path', 'perf_hooks', 'process', 'punycode', 'querystring', 'readline',
  'repl', 'stream', 'string_decoder', 'sys', 'timers', 'tls', 'trace_events',
  'tty', 'url', 'util', 'v8', 'vm', 'wasi', 'worker_threads', 'zlib',
];

let _moduleCache = null;

function getModuleCache() {
  if (!_moduleCache && typeof globalThis.require !== 'undefined' && globalThis.require.cache) {
    _moduleCache = globalThis.require.cache;
  }
  if (!_moduleCache) {
    _moduleCache = new Map();
  }
  return _moduleCache;
}

const _extensions = {
  '.js': function(mod, filename) {
    try {
      const fs = globalThis.require('fs');
      const content = fs.readFileSync(filename, 'utf8');
      const wrapper = new Function('exports', 'require', 'module', '__filename', '__dirname', content);
      const dirname = filename.substring(0, filename.lastIndexOf('/')) || '/';
      wrapper(mod.exports, Module.createRequire(filename), mod, filename, dirname);
      return mod.exports;
    } catch (e) {
      throw e;
    }
  },
  '.json': function(mod, filename) {
    try {
      const fs = globalThis.require('fs');
      const content = fs.readFileSync(filename, 'utf8');
      mod.exports = JSON.parse(content);
      return mod.exports;
    } catch (e) {
      throw e;
    }
  },
  '.node': function(mod, filename) {
    throw new Error("Cannot load native module: " + filename);
  },
};

class Module {
  constructor(filename = '', parent = null) {
    this.id = filename;
    this.filename = filename;
    this.loaded = false;
    this.parent = parent;
    this.children = [];
    this.exports = {};
    this.paths = Module._resolveLookupPaths(filename);
    this.path = filename ? filename.substring(0, filename.lastIndexOf('/')) || '/' : '/';
    if (parent) {
      parent.children.push(this);
    }
  }

  require(id) {
    return Module._load(id, this);
  }

  static _resolveFilename(request, parent, isMain, options) {
    if (typeof globalThis.require !== 'undefined' && typeof globalThis.require.resolve === 'function') {
      return globalThis.require.resolve(request);
    }
    if (request.startsWith('./') || request.startsWith('../')) {
      const parentDir = parent ? parent.path : '/';
      const parts = request.split('/');
      const baseParts = parentDir.split('/');
      for (const part of parts) {
        if (part === '.') continue;
        if (part === '..') { if (baseParts.length > 0) baseParts.pop(); }
        else baseParts.push(part);
      }
      return baseParts.join('/');
    }
    if (request.startsWith('/')) return request;
    return 'node_modules/' + request;
  }

  static _load(request, parent, isMain) {
    const cache = getModuleCache();
    if (cache.has(request)) {
      return cache.get(request);
    }
    const filename = Module._resolveFilename(request, parent, isMain);
    if (cache.has(filename)) {
      return cache.get(filename);
    }
    if (builtinModulesList.includes(request)) {
      let mod;
      try {
        mod = globalThis.require(request);
      } catch (e) {
        mod = {};
      }
      cache.set(request, mod);
      return mod;
    }
    try {
      const req = Module.createRequire(filename);
      const mod = new Module(filename, parent);
      cache.set(filename, mod.exports);
      Module._extensions['.js'](mod, filename);
      mod.loaded = true;
      cache.set(filename, mod.exports);
      return mod.exports;
    } catch (e) {
      cache.set(filename, {});
      return {};
    }
  }

  static _resolveLookupPaths(request, parent) {
    const paths = [];
    if (parent && parent.path) {
      let dir = parent.path;
      while (dir.length > 0) {
        paths.push(dir + '/node_modules');
        const idx = dir.lastIndexOf('/');
        if (idx <= 0) break;
        dir = dir.slice(0, idx);
      }
    }
    paths.push('/usr/local/lib/node_modules');
    return paths;
  }

  static get _cache() {
    return getModuleCache();
  }

  static set _cache(val) {
    _moduleCache = val;
  }

  static get builtinModules() {
    return [...builtinModulesList];
  }

  static createRequire(filename) {
    if (typeof globalThis.require === 'function') {
      const fn = (specifier) => Module._load(specifier, new Module(filename));
      fn.resolve = (specifier) => Module._resolveFilename(specifier, new Module(filename));
      fn.cache = Module._cache;
      fn.extensions = Module._extensions;
      fn.main = globalThis.require.main;
      return fn;
    }
    return (specifier) => {
      throw new Error(`Cannot find module '${specifier}'`);
    };
  }

  static _initPaths() {}
  static _preloadModules() {}

  static _nodeModulePaths(from) {
    const paths = [];
    let dir = from;
    while (true) {
      paths.push(dir + '/node_modules');
      const parent = dir.substring(0, dir.lastIndexOf('/'));
      if (parent === dir || parent === '') break;
      dir = parent;
    }
    return paths;
  }

  static _findPath(request, paths, isMain) {
    for (const p of paths) {
      const fullPath = p + '/' + request;
      try {
        if (typeof globalThis.require !== 'undefined' && globalThis.require.cache && globalThis.require.cache.has(fullPath)) {
          return fullPath;
        }
      } catch (e) {}
    }
    return false;
  }

  static _resolve(request, options) {
    return Module._resolveFilename(request, options ? options.parent : null);
  }

  static _stat(filename) {
    try {
      const fs = globalThis.require('fs');
      return fs.statSync(filename);
    } catch (e) {
      return false;
    }
  }

  static get _extensions() {
    return _extensions;
  }

  static set _extensions(v) {
    for (const key of Object.keys(v)) {
      _extensions[key] = v[key];
    }
  }
}

const moduleModule = {
  Module,
  builtinModules: builtinModulesList,
  createRequire: Module.createRequire,
  syncBuiltinESMExports: () => {},
  findSourceMap: () => undefined,
  register: () => {},
  enableCompileCache: () => {},
  constants: {
    compileCacheDir: undefined,
  },
};

Module._cache = getModuleCache();

if (typeof module !== 'undefined' && module.exports) {
  module.exports = moduleModule;
}
