// Klyron Runtime — node:async_hooks polyfill

let asyncIdCounter = 1;
const executionAsyncIdMap = new Map();
const triggerAsyncIdMap = new Map();
const asyncHookCallbacks = {};

function executionAsyncId() {
  return globalThis.Klyron && Klyron.asyncHookId ? Klyron.asyncHookId() : 1;
}

function triggerAsyncId() {
  return 0;
}

function createHook(options = {}) {
  if (options.init) asyncHookCallbacks.init = options.init;
  if (options.before) asyncHookCallbacks.before = options.before;
  if (options.after) asyncHookCallbacks.after = options.after;
  if (options.destroy) asyncHookCallbacks.destroy = options.destroy;
  if (options.promiseResolve) asyncHookCallbacks.promiseResolve = options.promiseResolve;

  return {
    enable() {
      this._enabled = true;
      return this;
    },
    disable() {
      this._enabled = false;
      return this;
    },
    _enabled: false,
  };
}

class AsyncHook {
  constructor(options) {
    return createHook(options);
  }
}

class AsyncLocalStorage {
  constructor() {
    this._storeMap = new Map();
    this._enabled = false;
  }

  run(store, callback, ...args) {
    const asyncId = executionAsyncId();
    const previous = this._storeMap.get(asyncId);
    this._storeMap.set(asyncId, store);
    this._enabled = true;
    try {
      return callback(...args);
    } finally {
      if (previous !== undefined) {
        this._storeMap.set(asyncId, previous);
      } else {
        this._storeMap.delete(asyncId);
      }
    }
  }

  getStore() {
    const asyncId = executionAsyncId();
    return this._storeMap.get(asyncId);
  }

  disable() {
    this._enabled = false;
    this._storeMap.clear();
  }

  enterWith(store) {
    const asyncId = executionAsyncId();
    this._storeMap.set(asyncId, store);
    this._enabled = true;
  }
}

class AsyncResource {
  constructor(type, options = {}) {
    this._type = type;
    this._asyncId = asyncIdCounter++;
    this._triggerAsyncId = options.triggerAsyncId || triggerAsyncId();
    this._requireManualDestroy = !!options.requireManualDestroy;
    executionAsyncIdMap.set(this._asyncId, this._type);
    triggerAsyncIdMap.set(this._asyncId, this._triggerAsyncId);

    if (asyncHookCallbacks.init) {
      try {
        asyncHookCallbacks.init(this._asyncId, this._type, this._triggerAsyncId, this);
      } catch (e) {
        queueMicrotask(() => { throw e; });
      }
    }
  }

  runInAsyncScope(fn, thisArg, ...args) {
    const prevAsyncId = executionAsyncId();
    try {
      if (asyncHookCallbacks.before) {
        try { asyncHookCallbacks.before(this._asyncId); } catch (e) { queueMicrotask(() => { throw e; }); }
      }
      return fn.call(thisArg || this, ...args);
    } finally {
      if (asyncHookCallbacks.after) {
        try { asyncHookCallbacks.after(this._asyncId); } catch (e) { queueMicrotask(() => { throw e; }); }
      }
    }
  }

  emitDestroy() {
    if (asyncHookCallbacks.destroy) {
      try {
        asyncHookCallbacks.destroy(this._asyncId);
      } catch (e) {
        queueMicrotask(() => { throw e; });
      }
    }
    executionAsyncIdMap.delete(this._asyncId);
    triggerAsyncIdMap.delete(this._asyncId);
    return this;
  }

  asyncId() {
    return this._asyncId;
  }

  triggerAsyncId() {
    return this._triggerAsyncId;
  }
}

const async_hooks = {
  AsyncLocalStorage,
  AsyncResource,
  executionAsyncId,
  triggerAsyncId,
  createHook,
  AsyncHook,
};

if (typeof module !== 'undefined' && module.exports) {
  module.exports = async_hooks;
}
