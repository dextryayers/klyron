// Klyron Runtime — node:worker_threads polyfill

const EventEmitter = (typeof require !== 'undefined' ? require('events') : globalThis).EventEmitter || class {};
const SHARE_ENV = Symbol('nodejs.worker_threads.SHARE_ENV');

let _nextWorkerId = 1;
let _workerData = undefined;
let _threadId = 0;
let _isMainThread = true;
let _isWorkerThread = false;

class MessagePort {
  constructor() {
    this._events = new Map();
    this._other = null;
    this._closed = false;
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

  addListener(event, handler) { return this.on(event, handler); }

  emit(event, ...args) {
    const handlers = this._events.get(event) || [];
    for (const h of [...handlers]) {
      try { h.call(this, ...args); } catch (e) { queueMicrotask(() => { throw e; }); }
    }
    return true;
  }

  postMessage(value, transferList) {
    if (this._closed) return;
    if (this._other) {
      queueMicrotask(() => {
        if (!this._other._closed) {
          this._other.emit('message', value);
        }
      });
    }
  }

  start() {}
  close() {
    this._closed = true;
    this.emit('close');
  }

  ref() {}
  unref() {}
}

class MessageChannel {
  constructor() {
    this.port1 = new MessagePort();
    this.port2 = new MessagePort();
    this.port1._other = this.port2;
    this.port2._other = this.port1;
  }
}

class Worker {
  constructor(filename, options = {}) {
    this._events = new Map();
    this._id = _nextWorkerId++;
    this._terminated = false;
    this._online = false;
    this._stdin = null;
    this._stdout = null;
    this._stderr = null;
    this.threadId = this._id;

    const opts = typeof options === 'string' ? { argv: [options] } : options;
    this._options = {
      argv: opts.argv || [],
      env: opts.env || process.env,
      workerData: opts.workerData || undefined,
      eval: opts.eval || false,
      execArgv: opts.execArgv || [],
      stdin: !!opts.stdin,
      stdout: !!opts.stdout,
      stderr: !!opts.stderr,
      trackUnmanagedFds: opts.trackUnmanagedFds !== false,
      transferList: opts.transferList || [],
      resourceLimits: opts.resourceLimits || {},
      name: opts.name || '',
    };

    this._parentPort = new MessagePort();
    this._parentPort._other = new MessagePort();
    this._parentPort._other._other = this._parentPort;

    const that = this;
    this.postMessage = this._parentPort.postMessage.bind(this._parentPort);
    this.on('message', (msg) => that._parentPort.emit('message', msg));

    this._workerPort = this._parentPort._other;

    queueMicrotask(() => {
      this._online = true;
      this.emit('online');
    });
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

  postMessage(value, transferList) {
    this._parentPort.postMessage(value, transferList);
  }

  terminate() {
    if (this._terminated) return Promise.resolve(0);
    this._terminated = true;
    this._online = false;
    queueMicrotask(() => {
      this.emit('exit', 0);
    });
    return Promise.resolve(0);
  }

  getHeapSnapshot() {
    return Promise.resolve(null);
  }

  get stderr() { return this._stderr; }
  get stdout() { return this._stdout; }
  get stdin() { return this._stdin; }
  get threadId() { return this._id; }
  get resourceLimits() { return this._options.resourceLimits; }

  ref() {}
  unref() {}
}

const parentPort = new MessagePort();
const workerData = _workerData;
const isMainThread = _isMainThread;
const isWorkerThread = _isWorkerThread;
const threadId = _threadId;

function receiveMessageOnPort(port) {
  return undefined;
}

function markAsUntransferable(obj) {}
function moveMessagePortToContext(port, context) { return port; }

const worker_threads = {
  Worker,
  MessagePort,
  MessageChannel,
  parentPort,
  workerData,
  isMainThread,
  isWorkerThread,
  threadId,
  SHARE_ENV,
  receiveMessageOnPort,
  markAsUntransferable,
  moveMessagePortToContext,
  getEnvironmentData: (key) => undefined,
  setEnvironmentData: (key, value) => {},
};

if (typeof module !== 'undefined' && module.exports) {
  module.exports = worker_threads;
}
