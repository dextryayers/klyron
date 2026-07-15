// Klyron Runtime — node:cluster polyfill

const EventEmitter = (typeof require !== 'undefined' ? require('events') : globalThis).EventEmitter || class {};

let _currentWorker = null;
let _workers = {};
let _nextId = 1;
let _settings = {};

class Worker {
  constructor(id) {
    this._events = new Map();
    this.id = id;
    this.process = {
      pid: -1,
      ppid: -1,
      exitCode: null,
      signalCode: null,
      spawnfile: '',
      spawnargs: [],
    };
    this.exitedAfterDisconnect = false;
    this.state = 'none';
    this.suicide = false;
  }

  on(event, handler) {
    if (!this._events.has(event)) this._events.set(event, []);
    this._events.get(event).push(handler);
    return this;
  }

  once(event, handler) {
    const wrapper = (...args) => { handler(...args); this.removeListener(event, wrapper); };
    return this.on(event, wrapper);
  }

  removeListener(event, handler) {
    const handlers = this._events.get(event);
    if (handlers) {
      this._events.set(event, handlers.filter(h => h !== handler));
    }
    return this;
  }

  emit(event, ...args) {
    const handlers = this._events.get(event) || [];
    for (const h of [...handlers]) {
      try { h.call(this, ...args); } catch (e) { queueMicrotask(() => { throw e; }); }
    }
    return true;
  }

  send(message, sendHandle, callback) {
    if (typeof sendHandle === 'function') { callback = sendHandle; sendHandle = undefined; }
    queueMicrotask(() => {
      this.emit('message', message);
      if (callback) callback(null);
      cluster.emit('message', this, message, sendHandle);
    });
    return true;
  }

  kill(signal = 'SIGTERM') {
    this.state = 'dead';
    this.process.exitCode = 1;
    this.process.signalCode = signal;
    queueMicrotask(() => {
      this.emit('exit', this.process.exitCode, this.process.signalCode);
      cluster.emit('exit', this, this.process.exitCode, this.process.signalCode);
    });
  }

  disconnect() {
    this.state = 'disconnected';
    this.exitedAfterDisconnect = true;
    this.suicide = true;
    queueMicrotask(() => {
      this.emit('disconnect');
      cluster.emit('disconnect', this);
    });
  }

  isConnected() { return this.state === 'connected' || this.state === 'online'; }
  isDead() { return this.state === 'dead'; }
  toString() { return `Worker ${this.id}`; }
}

const cluster = {
  _events: new Map(),

  isMaster: true,
  isPrimary: true,
  isWorker: false,
  isChild: false,

  get workers() { return { ..._workers }; },
  get settings() { return { ..._settings }; },

  fork(env) {
    const id = _nextId++;
    const worker = new Worker(id);
    worker.state = 'online';
    _workers[id] = worker;
    _currentWorker = worker;
    queueMicrotask(() => {
      worker.emit('online');
      cluster.emit('online', worker);
      worker.emit('listening', { address: '0.0.0.0', port: 0, addressType: 4, fd: -1 });
      cluster.emit('listening', worker, { address: '0.0.0.0', port: 0, addressType: 4, fd: -1 });
    });
    return worker;
  },

  setupMaster(settings = {}) {
    _settings = { ..._settings, ...settings };
  },

  disconnect(callback) {
    for (const id of Object.keys(_workers)) {
      const w = _workers[id];
      w.disconnect();
    }
    if (callback) queueMicrotask(callback);
  },

  on(event, handler) {
    if (!cluster._events.has(event)) cluster._events.set(event, []);
    cluster._events.get(event).push(handler);
    return cluster;
  },

  once(event, handler) {
    const wrapper = (...args) => { handler(...args); cluster.removeListener(event, wrapper); };
    return cluster.on(event, wrapper);
  },

  removeListener(event, handler) {
    const handlers = cluster._events.get(event);
    if (handlers) {
      cluster._events.set(event, handlers.filter(h => h !== handler));
    }
    return cluster;
  },

  emit(event, ...args) {
    const handlers = cluster._events.get(event) || [];
    for (const h of [...handlers]) {
      try { h.call(cluster, ...args); } catch (e) { queueMicrotask(() => { throw e; }); }
    }
    return true;
  },

  get worker() { return _currentWorker; },
  get isMaster() { return true; },
  get isPrimary() { return true; },
  get isWorker() { return false; },
  get isChild() { return false; },

  SCHED_NONE: 1,
  SCHED_RR: 2,
  SCHED_NET: 3,
};

if (typeof module !== 'undefined' && module.exports) {
  module.exports = cluster;
}
