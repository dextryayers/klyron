// Klyron Runtime — node:net polyfill

const EventEmitter = (typeof require !== 'undefined' ? require('events') : globalThis).EventEmitter || class {};

const kState = Symbol('state');

class Socket {
  constructor(options = {}) {
    this._events = {};
    this._handlers = new Map();
    this.remoteAddress = null;
    this.remotePort = null;
    this.remoteFamily = null;
    this.localAddress = null;
    this.localPort = null;
    this[kState] = 'initial';
    this._encoding = null;
    this._destroyed = false;
    this._pendingData = [];
    this.writable = true;
    this.readable = true;
    if (options && options.handle) {
      this._handle = options.handle;
    }
    if (typeof options === 'function') {
      this.on('connect', options);
    }
  }

  on(event, handler) {
    if (!this._handlers.has(event)) this._handlers.set(event, []);
    this._handlers.get(event).push(handler);
    return this;
  }

  emit(event, ...args) {
    const handlers = this._handlers.get(event) || [];
    for (const h of [...handlers]) {
      try { h.call(this, ...args); } catch (e) { queueMicrotask(() => { throw e; }); }
    }
    return true;
  }

  once(event, handler) {
    const wrapper = (...args) => { handler(...args); this.removeListener(event, wrapper); };
    return this.on(event, wrapper);
  }

  removeListener(event, handler) {
    const handlers = this._handlers.get(event);
    if (handlers) {
      this._handlers.set(event, handlers.filter(h => h !== handler));
    }
    return this;
  }

  addListener(event, handler) { return this.on(event, handler); }
  prependListener(event, handler) { return this.on(event, handler); }

  connect(port, host, callback) {
    if (typeof host === 'function') { callback = host; host = '127.0.0.1'; }
    if (typeof port === 'object') {
      const opts = port;
      port = opts.port;
      host = opts.host || '127.0.0.1';
      callback = callback || opts.callback;
    }
    this.remotePort = port;
    this.remoteAddress = host;
    this.remoteFamily = 'IPv4';
    this[kState] = 'connecting';
    queueMicrotask(() => {
      this[kState] = 'connected';
      this.emit('connect');
      if (callback) callback();
    });
    return this;
  }

  write(data, encoding, callback) {
    if (typeof encoding === 'function') { callback = encoding; encoding = undefined; }
    if (this._encoding && typeof data === 'string') {
      data = this._encode(data);
    }
    this._pendingData.push(data);
    if (callback) queueMicrotask(callback);
    return true;
  }

  end(data, encoding, callback) {
    if (typeof data === 'function') { callback = data; data = undefined; }
    if (typeof encoding === 'function') { callback = encoding; encoding = undefined; }
    if (data !== undefined) this.write(data, encoding);
    this.writable = false;
    queueMicrotask(() => {
      this[kState] = 'closed';
      this.emit('end');
      this.emit('close', false);
      if (callback) callback();
    });
    return this;
  }

  destroy(err) {
    if (this._destroyed) return this;
    this._destroyed = true;
    this[kState] = 'destroyed';
    this.readable = false;
    this.writable = false;
    queueMicrotask(() => {
      if (err) this.emit('error', err);
      this.emit('close', !!err);
    });
    return this;
  }

  setEncoding(encoding) {
    this._encoding = encoding;
    return this;
  }

  _encode(data) {
    if (this._encoding === 'utf8' || this._encoding === 'utf-8') {
      return data;
    }
    return data;
  }

  pause() { return this; }
  resume() { return this; }
  setTimeout(ms, callback) {
    if (callback) this.on('timeout', callback);
    return this;
  }
  setKeepAlive(enable, initialDelay) { return this; }
  setNoDelay(noDelay) { return this; }
  ref() { return this; }
  unref() { return this; }
  address() {
    return { address: this.localAddress || '0.0.0.0', port: this.localPort || 0, family: 'IPv4' };
  }
  get destroyed() { return this._destroyed; }
  get connecting() { return this[kState] === 'connecting'; }
  get pending() { return this[kState] === 'initial'; }
  get localPort() { return this._localPort; }
  set localPort(v) { this._localPort = v; }
  toString() { return `Socket (${this.remoteAddress || 'not connected'}:${this.remotePort || 0})`; }

  [Symbol.asyncIterator]() {
    let _resolve;
    let _reject;
    const buffer = [];
    const done = false;
    this.on('data', chunk => {
      if (_resolve) { _resolve({ value: chunk, done: false }); _resolve = null; }
      else buffer.push(chunk);
    });
    this.on('end', () => {
      if (_resolve) { _resolve({ value: undefined, done: true }); _resolve = null; }
    });
    this.on('error', err => {
      if (_reject) { _reject(err); _reject = null; }
    });
    return {
      next() {
        if (buffer.length > 0) return Promise.resolve({ value: buffer.shift(), done: false });
        if (done) return Promise.resolve({ value: undefined, done: true });
        return new Promise((resolve, reject) => {
          _resolve = resolve;
          _reject = reject;
        });
      },
      [Symbol.asyncIterator]() { return this; },
    };
  }
}

function createServer(options, connectionListener) {
  if (typeof options === 'function') { connectionListener = options; options = {}; }
  const server = new (class Server {
    constructor() {
      this._handlers = new Map();
      this._listening = false;
      this._connections = new Set();
      this._maxConnections = 0;
    }

    on(event, handler) {
      if (!this._handlers.has(event)) this._handlers.set(event, []);
      this._handlers.get(event).push(handler);
      return this;
    }

    once(event, handler) {
      const wrapper = (...args) => { handler(...args); this.removeListener(event, wrapper); };
      return this.on(event, wrapper);
    }

    addListener(event, handler) { return this.on(event, handler); }
    prependListener(event, handler) { return this.on(event, handler); }

    removeListener(event, handler) {
      const handlers = this._handlers.get(event);
      if (handlers) {
        this._handlers.set(event, handlers.filter(h => h !== handler));
      }
      return this;
    }

    emit(event, ...args) {
      const handlers = this._handlers.get(event) || [];
      for (const h of [...handlers]) {
        try { h.call(this, ...args); } catch (e) { queueMicrotask(() => { throw e; }); }
      }
      return true;
    }

    listen(port, host, callback) {
      if (typeof host === 'function') { callback = host; host = '0.0.0.0'; }
      if (typeof port === 'object') {
        const opts = port;
        port = opts.port || 0;
        host = opts.host || '0.0.0.0';
        callback = callback || opts.callback;
      }
      this._port = port || 0;
      this._host = host || '0.0.0.0';
      queueMicrotask(() => {
        this._listening = true;
        this.emit('listening');
        if (callback) callback();
      });
      return this;
    }

    close(callback) {
      if (callback) this.on('close', callback);
      queueMicrotask(() => {
        this._listening = false;
        for (const conn of this._connections) {
          conn.destroy();
        }
        this._connections.clear();
        this.emit('close');
      });
      return this;
    }

    address() {
      if (!this._listening) return null;
      return { address: this._host, port: this._port || 0, family: 'IPv4' };
    }

    getConnections(callback) {
      queueMicrotask(() => callback(null, this._connections.size));
    }

    ref() { return this; }
    unref() { return this; }

    get listening() { return this._listening; }
    get maxConnections() { return this._maxConnections; }
    set maxConnections(v) { this._maxConnections = v; }

    async [Symbol.asyncIterator]() {
      const buffer = [];
      let resolve;
      this.on('connection', conn => {
        if (resolve) { resolve(conn); resolve = null; }
        else buffer.push(conn);
      });
      return {
        next() {
          if (buffer.length > 0) return Promise.resolve({ value: buffer.shift(), done: false });
          return new Promise(r => resolve = r);
        },
        [Symbol.asyncIterator]() { return this; },
      };
    }
  })();

  if (connectionListener) server.on('connection', connectionListener);
  return server;
}

function connect(port, host, callback) {
  const socket = new Socket();
  socket.connect(port, host, callback);
  return socket;
}

function createConnection(port, host, callback) {
  return connect(port, host, callback);
}

function isIP(input) {
  if (typeof input !== 'string') return 0;
  const ipv4 = /^(\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})$/;
  const m = input.match(ipv4);
  if (m) {
    for (let i = 1; i <= 4; i++) {
      const octet = parseInt(m[i], 10);
      if (octet < 0 || octet > 255) return 0;
    }
    return 4;
  }
  const ipv6 = /^([0-9a-fA-F]{0,4}:){2,7}[0-9a-fA-F]{0,4}(%\w+)?$/;
  if (ipv6.test(input)) return 6;
  return 0;
}

function isIPv4(input) {
  return isIP(input) === 4;
}

function isIPv6(input) {
  return isIP(input) === 6;
}

const net = {
  Server: createServer,
  Socket,
  createServer,
  connect,
  createConnection,
  isIP,
  isIPv4,
  isIPv6,
};

if (typeof module !== 'undefined' && module.exports) {
  module.exports = net;
}
