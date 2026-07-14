// Klyron Runtime — node:http polyfill
// http.createServer, http.request, http.get, ServerResponse, IncomingMessage

const EE = (typeof require === 'function' ? (function() { try { return require('./node_events'); } catch {} })() : null) || { EventEmitter: globalThis.EventEmitter || class {} };
const EventEmitter = EE.EventEmitter;

const STATUS_CODES = {
  100: 'Continue', 101: 'Switching Protocols', 102: 'Processing',
  200: 'OK', 201: 'Created', 202: 'Accepted', 203: 'Non-Authoritative Information',
  204: 'No Content', 205: 'Reset Content', 206: 'Partial Content', 207: 'Multi-Status',
  300: 'Multiple Choices', 301: 'Moved Permanently', 302: 'Found', 303: 'See Other',
  304: 'Not Modified', 305: 'Use Proxy', 307: 'Temporary Redirect', 308: 'Permanent Redirect',
  400: 'Bad Request', 401: 'Unauthorized', 402: 'Payment Required', 403: 'Forbidden',
  404: 'Not Found', 405: 'Method Not Allowed', 406: 'Not Acceptable',
  407: 'Proxy Authentication Required', 408: 'Request Timeout', 409: 'Conflict',
  410: 'Gone', 411: 'Length Required', 412: 'Precondition Failed',
  413: 'Payload Too Large', 414: 'URI Too Long', 415: 'Unsupported Media Type',
  416: 'Range Not Satisfiable', 417: 'Expectation Failed', 418: 'I\'m a Teapot',
  422: 'Unprocessable Entity', 423: 'Locked', 424: 'Failed Dependency',
  425: 'Too Early', 426: 'Upgrade Required', 428: 'Precondition Required',
  429: 'Too Many Requests', 431: 'Request Header Fields Too Large',
  451: 'Unavailable For Legal Reasons',
  500: 'Internal Server Error', 501: 'Not Implemented', 502: 'Bad Gateway',
  503: 'Service Unavailable', 504: 'Gateway Timeout', 505: 'HTTP Version Not Supported',
  506: 'Variant Also Negotiates', 507: 'Insufficient Storage', 508: 'Loop Detected',
  509: 'Bandwidth Limit Exceeded', 510: 'Not Extended', 511: 'Network Authentication Required',
};

class IncomingMessage extends EventEmitter {
  constructor(socket) {
    super();
    this.socket = socket;
    this.headers = {};
    this.rawHeaders = [];
    this.method = 'GET';
    this.url = '/';
    this.statusCode = 200;
    this.statusMessage = 'OK';
    this.httpVersion = '1.1';
    this.httpVersionMajor = 1;
    this.httpVersionMinor = 1;
    this._readableState = { buffer: [], ended: false };
    this._body = '';
    this._chunks = [];
  }

  get trailers() { return {}; }
  get rawTrailers() { return []; }
  get aborted() { return false; }
  get complete() { return this._readableState.ended; }

  _addHeaderLine(name, value) {
    const lower = name.toLowerCase();
    if (this.headers[lower]) {
      if (Array.isArray(this.headers[lower])) {
        this.headers[lower].push(value);
      } else {
        this.headers[lower] = [this.headers[lower], value];
      }
    } else {
      this.headers[lower] = value;
    }
    this.rawHeaders.push(name, value);
  }

  push(chunk) {
    if (chunk === null) {
      this._readableState.ended = true;
      this.emit('end');
      return;
    }
    this._chunks.push(chunk);
    this._body += typeof chunk === 'string' ? chunk : new TextDecoder().decode(chunk);
    this.emit('data', chunk);
  }

  setTimeout(msecs, callback) {
    if (callback) this.once('timeout', callback);
    setTimeout(() => this.emit('timeout'), msecs);
    return this;
  }

  destroy(error) {
    if (error) this.emit('error', error);
    this.emit('close');
  }
}

class ServerResponse extends EventEmitter {
  constructor(req) {
    super();
    this.req = req;
    this.statusCode = 200;
    this.statusMessage = 'OK';
    this._headers = {};
    this._headerNames = {};
    this._headersSent = false;
    this._body = '';
    this._chunks = [];
    this._ended = false;
    this.sendDate = true;
    this.shouldKeepAlive = false;
    this.useChunkedEncodingByDefault = true;
    this.chunkedEncoding = false;
    this._header = '';
    this.finished = false;
    this.headersSent = false;
    this.connection = null;
    this.socket = null;
  }

  _implicitHeader() {
    this.writeHead(this.statusCode, this.statusMessage);
  }

  writeHead(statusCode, statusMessage, headers) {
    if (typeof statusMessage === 'object') {
      headers = statusMessage;
      statusMessage = undefined;
    }
    if (statusMessage) {
      this.statusMessage = statusMessage;
    } else {
      this.statusMessage = STATUS_CODES[statusCode] || 'Unknown';
    }
    this.statusCode = statusCode;
    if (headers) {
      for (const [k, v] of Object.entries(headers)) {
        this.setHeader(k, v);
      }
    }
    this.headersSent = true;
    return this;
  }

  setHeader(name, value) {
    const lower = name.toLowerCase();
    this._headerNames[lower] = name;
    this._headers[lower] = String(value);
  }

  getHeader(name) {
    return this._headers[name.toLowerCase()];
  }

  getHeaders() {
    return { ...this._headers };
  }

  getHeaderNames() {
    return Object.keys(this._headers);
  }

  hasHeader(name) {
    return name.toLowerCase() in this._headers;
  }

  removeHeader(name) {
    delete this._headers[name.toLowerCase()];
  }

  write(chunk, encoding, callback) {
    if (typeof encoding === 'function') { callback = encoding; encoding = null; }
    if (typeof chunk === 'string') {
      chunk = new TextEncoder().encode(chunk);
    }
    this._chunks.push(chunk);
    if (callback) callback(null);
    return true;
  }

  end(chunk, encoding, callback) {
    if (typeof chunk === 'function') { callback = chunk; chunk = null; }
    else if (typeof encoding === 'function') { callback = encoding; encoding = null; }
    if (chunk) this.write(chunk, encoding);
    this._ended = true;
    this.finished = true;
    if (!this.headersSent) {
      this.writeHead(this.statusCode);
    }
    if (callback) callback(null);
    this.emit('finish');
    return this;
  }

  addTrailers(headers) {}

  setTimeout(msecs, callback) {
    if (callback) this.once('timeout', callback);
    setTimeout(() => this.emit('timeout'), msecs);
    return this;
  }

  destroy(error) {
    if (error) this.emit('error', error);
    this.emit('close');
  }

  flushHeaders() {}
  writeContinue() {}
  writeProcessing() {}
  assignSocket(socket) { this.socket = socket; this.connection = socket; }
  detachSocket(socket) { this.socket = null; this.connection = null; }
}

function createServer(requestListener) {
  const server = new EventEmitter();
  server._listener = requestListener || (() => {});
  server.maxHeadersCount = 2000;
  server.headersTimeout = 60000;
  server.requestTimeout = 300000;
  server.timeout = 0;
  server.keepAliveTimeout = 5000;
  server._connections = 0;
  server.listening = false;

  server.address = () => ({ address: '0.0.0.0', family: 'IPv4', port: server._port || 0 });
  server.getConnections = (cb) => { if (cb) cb(null, server._connections); };

  server.listen = (port, hostname, backlog, callback) => {
    if (typeof port === 'object' && port.port) port = port.port;
    if (typeof hostname === 'function') { callback = hostname; hostname = '0.0.0.0'; }
    if (typeof backlog === 'function') { callback = backlog; backlog = 511; }
    if (typeof port === 'function') { callback = port; port = 0; }
    server._port = port || 0;
    server.listening = true;

    if (typeof Klyron?.net?.listen === 'function') {
      try {
        Klyron.net.listen({ port: server._port, hostname: hostname || '0.0.0.0' });
      } catch (e) {
        if (callback) callback(e);
        else throw e;
        return server;
      }
    }

    queueMicrotask(() => {
      server.emit('listening');
      if (callback) callback();
    });

    server._pollInterval = setInterval(() => {
      if (typeof Klyron?.net?.accept === 'function') {
        try {
          const conn = Klyron.net.accept();
          if (conn) {
            server._connections++;
            const req = new IncomingMessage();
            req.method = conn.method || 'GET';
            req.url = conn.url || '/';
            req.httpVersion = conn.httpVersion || '1.1';
            if (conn.headers) {
              for (const [k, v] of Object.entries(conn.headers)) {
                req._addHeaderLine(k, v);
              }
            }
            const res = new ServerResponse(req);
            server._listener(req, res);
          }
        } catch (e) {
          if (e.message !== 'WouldBlock') {
            server.emit('error', e);
          }
        }
      }
    }, 50);

    return server;
  };

  server.close = (callback) => {
    server.listening = false;
    if (server._pollInterval) {
      clearInterval(server._pollInterval);
      server._pollInterval = null;
    }
    queueMicrotask(() => {
      server.emit('close');
      if (callback) callback();
    });
    return server;
  };

  return server;
}

function request(url, options, callback) {
  if (typeof url === 'object') {
    if (typeof options === 'function') { callback = options; }
    options = url;
    url = options.href || `${options.protocol || 'http:'}//${options.hostname || options.host || 'localhost'}${options.path || options.pathname || '/'}`;
  }
  if (typeof options === 'function') { callback = options; options = {}; }
  if (!options) options = {};

  const parsedUrl = typeof url === 'string' ? new URL(url) : url;
  const method = (options.method || 'GET').toUpperCase();
  const headers = options.headers || {};

  const req = new EventEmitter();
  req.method = method;
  req.path = parsedUrl.pathname || '/';
  req.headers = headers;
  req._body = null;
  req.aborted = false;

  req.write = (chunk) => {
    if (typeof chunk === 'string') chunk = new TextEncoder().encode(chunk);
    if (!req._body) req._body = chunk;
    else {
      const combined = new Uint8Array(req._body.length + chunk.length);
      combined.set(req._body);
      combined.set(chunk, req._body.length);
      req._body = combined;
    }
    return true;
  };

  req.end = (chunk) => {
    if (chunk) req.write(chunk);

    const res = new IncomingMessage();
    const fetchHeaders = {};

    if (typeof Klyron?.net?.fetch === 'function') {
      try {
        const result = Klyron.net.fetch({
          method,
          url: parsedUrl.href || url,
          headers: { ...headers },
          body: req._body ? new TextDecoder().decode(req._body) : undefined,
        });
        res.statusCode = result.status || 200;
        res.statusMessage = result.statusText || STATUS_CODES[res.statusCode] || 'OK';
        if (result.headers) {
          for (const [k, v] of Object.entries(result.headers)) {
            res._addHeaderLine(k, v);
          }
        }
        if (result.body) {
          res.push(result.body);
        }
        res.push(null);
      } catch (e) {
        res.statusCode = 0;
        req.emit('error', e);
        return req;
      }
    } else {
      queueMicrotask(() => {
        res.push(null);
      });
    }

    queueMicrotask(() => {
      req.emit('response', res);
      if (callback) callback(res);
    });

    return req;
  };

  req.setTimeout = (msecs, callback) => {
    if (callback) req.once('timeout', callback);
    setTimeout(() => req.emit('timeout'), msecs);
    return req;
  };

  req.abort = () => {
    req.aborted = true;
    req.emit('abort');
    req.emit('close');
  };

  req.destroy = (error) => {
    if (error) req.emit('error', error);
    req.emit('close');
  };

  req.flushHeaders = () => {};

  req.setHeader = (name, value) => { headers[name] = value; };
  req.getHeader = (name) => headers[name];
  req.removeHeader = (name) => { delete headers[name]; };

  if (!options._noEnd) {
    req.end();
  }

  return req;
}

function get(url, options, callback) {
  if (typeof options === 'function') { callback = options; options = {}; }
  return request(url, { ...options, method: 'GET' }, callback);
}

const http = {
  createServer,
  request,
  get,
  ServerResponse,
  IncomingMessage,
  STATUS_CODES,
  METHODS: [
    'GET', 'HEAD', 'POST', 'PUT', 'DELETE', 'CONNECT', 'OPTIONS', 'TRACE', 'PATCH',
  ],
  maxHeaderSize: 16384,
  globalAgent: {},
  Agent: class Agent {
    constructor() { this.maxSockets = Infinity; this.sockets = {}; this.requests = {}; }
    destroy() {}
  },
  ClientRequest: function() {},
  Server: function() {},
  OutgoingMessage: ServerResponse,
  validateHeaderName() {},
  validateHeaderValue() {},
  _connectionListener: () => {},
};

if (typeof module !== 'undefined' && module.exports) {
  module.exports = http;
}
