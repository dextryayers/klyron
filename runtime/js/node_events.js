// Klyron Runtime — node:events polyfill
// EventEmitter class with full Node.js compatible API

const kDefaultMaxListeners = 10;

class EventEmitter {
  constructor() {
    this._events = new Map();
    this._maxListeners = EventEmitter.defaultMaxListeners;
    this._onNewListener = null;
    this._onRemoveListener = null;
  }

  _addListener(eventName, listener, prepend = false, once = false) {
    if (typeof listener !== 'function') throw new TypeError('listener must be a function');
    if (!this._events.has(eventName)) {
      this._events.set(eventName, []);
    }
    const listeners = this._events.get(eventName);
    const wrapped = once ? this._onceWrap(eventName, listener) : listener;
    wrapped._isOnce = once;
    wrapped._listener = listener;
    if (prepend) {
      listeners.unshift(wrapped);
    } else {
      listeners.push(wrapped);
    }
    if (this._onNewListener) {
      this._onNewListener(eventName, listener);
    }
    if (listeners.length > this._maxListeners && !this._warned) {
      this._warned = true;
      if (typeof process !== 'undefined' && process.emitWarning) {
        process.emitWarning(
          `Possible EventEmitter memory leak detected. ${listeners.length} ${eventName} listeners added. Use emitter.setMaxListeners() to increase limit`,
          'MaxListenersExceededWarning'
        );
      }
    }
    return this;
  }

  _onceWrap(eventName, listener) {
    const wrapped = (...args) => {
      this.removeListener(eventName, wrapped);
      return listener.apply(this, args);
    };
    return wrapped;
  }

  addListener(eventName, listener) {
    return this._addListener(eventName, listener);
  }

  on(eventName, listener) {
    return this._addListener(eventName, listener);
  }

  prependListener(eventName, listener) {
    return this._addListener(eventName, listener, true);
  }

  once(eventName, listener) {
    return this._addListener(eventName, listener, false, true);
  }

  prependOnceListener(eventName, listener) {
    return this._addListener(eventName, listener, true, true);
  }

  removeListener(eventName, listener) {
    const listeners = this._events.get(eventName);
    if (!listeners) return this;
    const filtered = listeners.filter(l => {
      if (l === listener) return false;
      if (l._listener === listener) return false;
      return true;
    });
    if (filtered.length === 0) {
      this._events.delete(eventName);
    } else {
      this._events.set(eventName, filtered);
    }
    if (this._onRemoveListener) {
      this._onRemoveListener(eventName, listener);
    }
    return this;
  }

  off(eventName, listener) {
    return this.removeListener(eventName, listener);
  }

  removeAllListeners(eventName) {
    if (eventName) {
      this._events.delete(eventName);
    } else {
      this._events.clear();
    }
    return this;
  }

  setMaxListeners(n) {
    if (typeof n !== 'number' || n < 0) throw new RangeError('MaxListeners must be a non-negative number');
    this._maxListeners = n;
    this._warned = false;
    return this;
  }

  getMaxListeners() {
    return this._maxListeners;
  }

  listeners(eventName) {
    const listeners = this._events.get(eventName);
    if (!listeners) return [];
    return listeners.map(l => l._listener || l);
  }

  rawListeners(eventName) {
    const listeners = this._events.get(eventName);
    if (!listeners) return [];
    return [...listeners];
  }

  emit(eventName, ...args) {
    const listeners = this._events.get(eventName);
    if (!listeners) return false;
    const copy = [...listeners];
    for (const listener of copy) {
      try {
        listener.apply(this, args);
      } catch (e) {
        queueMicrotask(() => { throw e; });
      }
      if (listener._isOnce) {
        this.removeListener(eventName, listener);
      }
    }
    return true;
  }

  eventNames() {
    return Array.from(this._events.keys());
  }

  listenerCount(eventName) {
    const listeners = this._events.get(eventName);
    return listeners ? listeners.length : 0;
  }

  static defaultMaxListeners = kDefaultMaxListeners;

  static get defaultMaxListeners() {
    return this._defaultMaxListeners || kDefaultMaxListeners;
  }

  static set defaultMaxListeners(val) {
    this._defaultMaxListeners = val;
  }

  static listenerCount(emitter, eventName) {
    return emitter.listenerCount(eventName);
  }

  static once(emitter, eventName, options) {
    return new Promise((resolve, reject) => {
      const signal = options?.signal;
      if (signal && signal.aborted) {
        reject(signal.reason || new Error('Aborted'));
        return;
      }
      const handler = (...args) => {
        emitter.removeListener(eventName, handler);
        if (signal) signal.removeEventListener('abort', abortHandler);
        resolve(args.length === 1 ? args[0] : args);
      };
      const abortHandler = () => {
        emitter.removeListener(eventName, handler);
        reject(signal?.reason || new Error('Aborted'));
      };
      emitter.once(eventName, handler);
      if (signal) signal.addEventListener('abort', abortHandler);
    });
  }

  static getEventListeners(emitter, eventName) {
    return emitter.listeners(eventName);
  }

  static on(emitter, eventName, options) {
    const signal = options?.signal;
    const buffer = [];
    const iterator = {
      next() {
        if (signal && signal.aborted) {
          return Promise.reject(signal.reason || new Error('Aborted'));
        }
        if (buffer.length > 0) {
          return Promise.resolve({ value: buffer.shift(), done: false });
        }
        return new Promise((resolve, reject) => {
          emitter.once(eventName, (...args) => {
            const value = args.length === 1 ? args[0] : args;
            resolve({ value, done: false });
          });
          if (signal) {
            const abortHandler = () => {
              reject(signal?.reason || new Error('Aborted'));
            };
            emitter.once(eventName, abortHandler);
          }
        });
      },
      [Symbol.asyncIterator]() { return this; },
    };
    return iterator;
  }

  [Symbol.rawListenerCount]() {
    let count = 0;
    for (const listeners of this._events.values()) {
      count += listeners.length;
    }
    return count;
  }
}

EventEmitter.prototype.addListener = EventEmitter.prototype.on;

const events = {
  EventEmitter,
  once: EventEmitter.once,
  on: EventEmitter.on,
  getEventListeners: EventEmitter.getEventListeners,
  listenerCount: EventEmitter.listenerCount,
  defaultMaxListeners: kDefaultMaxListeners,
  init(): void {},
  captureRejections: false,
  captureRejectionSymbol: Symbol.for('nodejs.rejection'),
  errorMonitor: Symbol.for('events.errorMonitor'),
  EventEmitterAsyncResource: class EventEmitterAsyncResource extends EventEmitter {
    constructor(options) { super(); }
    asyncId() { return -1; }
    triggerAsyncId() { return -1; }
    emitDestroy() {}
  },
};

if (typeof module !== 'undefined' && module.exports) {
  module.exports = events;
}
