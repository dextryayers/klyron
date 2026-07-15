// Klyron Runtime — node:inspector polyfill

let _open = false;
let _port = 0;
let _host = '127.0.0.1';
let _url = '';

function open(port, host, wait) {
  if (typeof port === 'object') {
    wait = port.wait;
    host = port.host;
    port = port.port;
  }
  _port = port || 9229;
  _host = host || '127.0.0.1';
  _open = true;
  _url = `ws://${_host}:${_port}/${Math.random().toString(36).slice(2)}`;
  return _url;
}

function close() {
  _open = false;
  _url = '';
}

function url() {
  return _open ? _url : undefined;
}

const inspectorConsole = {
  log(...args) {
    if (_open) {
      const msg = args.map(a => typeof a === 'object' ? JSON.stringify(a, null, 2) : String(a)).join(' ');
      if (typeof process !== 'undefined' && process.stdout) {
        process.stdout.write(`[inspector] ${msg}\n`);
      }
    }
  },
  warn(...args) {
    if (_open) {
      const msg = args.map(a => typeof a === 'object' ? JSON.stringify(a, null, 2) : String(a)).join(' ');
      if (typeof process !== 'undefined' && process.stderr) {
        process.stderr.write(`[inspector] ${msg}\n`);
      }
    }
  },
  error(...args) {
    const msg = args.map(a => typeof a === 'object' ? JSON.stringify(a, null, 2) : String(a)).join(' ');
    if (typeof process !== 'undefined' && process.stderr) {
      process.stderr.write(`[inspector] ${msg}\n`);
    }
  },
};

class Session {
  constructor() {
    this._events = new Map();
    this._connected = false;
    this._id = 1;
    this._pending = new Map();
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

  connect() {
    this._connected = true;
    this.emit('Session.connect');
  }

  disconnect() {
    this._connected = false;
    this._pending.clear();
    this.emit('Session.disconnect');
  }

  post(method, params, callback) {
    if (typeof params === 'function') { callback = params; params = {}; }
    if (!callback) {
      return new Promise((resolve, reject) => {
        this.post(method, params, (err, result) => err ? reject(err) : resolve(result));
      });
    }
    if (!this._connected) {
      callback(new Error('Session is not connected.'));
      return;
    }
    const id = this._id++;
    this._pending.set(id, callback);
    queueMicrotask(() => {
      const result = this._handleMethod(method, params);
      callback(null, result);
      this._pending.delete(id);
    });
  }

  _handleMethod(method, params) {
    switch (method) {
      case 'Runtime.evaluate':
        try {
          const result = eval(params.expression);
          return {
            result: { type: typeof result, value: result, description: String(result) },
          };
        } catch (e) {
          return {
            exceptionDetails: { text: e.message, exception: { type: 'object', description: e.stack } },
          };
        }
      case 'Runtime.getProperties':
        return { result: [] };
      case 'Debugger.enable':
        return {};
      case 'Debugger.disable':
        return {};
      case 'Profiler.enable':
        return {};
      case 'Profiler.disable':
        return {};
      case 'HeapProfiler.enable':
        return {};
      case 'HeapProfiler.disable':
        return {};
      case 'Runtime.runIfWaitingForDebugger':
        return {};
      default:
        return {};
    }
  }
}

const inspector = {
  open,
  close,
  url,
  console: inspectorConsole,
  Session,
};

if (typeof module !== 'undefined' && module.exports) {
  module.exports = inspector;
}
