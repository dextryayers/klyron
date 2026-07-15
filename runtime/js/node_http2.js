// Klyron Runtime — node:http2 polyfill

const EventEmitter = (typeof require !== 'undefined' ? require('events') : globalThis).EventEmitter || class {};

const kDefaultSettings = {
  headerTableSize: 4096,
  enablePush: true,
  initialWindowSize: 65535,
  maxFrameSize: 16384,
  maxConcurrentStreams: 100,
  maxHeaderListSize: 65536,
  enableConnectProtocol: false,
};

const constants = {
  NGHTTP2_SESSION_SERVER: 0,
  NGHTTP2_SESSION_CLIENT: 1,
  NGHTTP2_STREAM_STATE_IDLE: 0,
  NGHTTP2_STREAM_STATE_OPEN: 1,
  NGHTTP2_STREAM_STATE_RESERVED_LOCAL: 2,
  NGHTTP2_STREAM_STATE_RESERVED_REMOTE: 3,
  NGHTTP2_STREAM_STATE_HALF_CLOSED_LOCAL: 4,
  NGHTTP2_STREAM_STATE_HALF_CLOSED_REMOTE: 5,
  NGHTTP2_STREAM_STATE_CLOSED: 6,
  NGHTTP2_NO_ERROR: 0,
  NGHTTP2_PROTOCOL_ERROR: 1,
  NGHTTP2_INTERNAL_ERROR: 2,
  NGHTTP2_FLOW_CONTROL_ERROR: 3,
  NGHTTP2_SETTINGS_TIMEOUT: 4,
  NGHTTP2_STREAM_CLOSED: 5,
  NGHTTP2_FRAME_SIZE_ERROR: 6,
  NGHTTP2_REFUSED_STREAM: 7,
  NGHTTP2_CANCEL: 8,
  NGHTTP2_COMPRESSION_ERROR: 9,
  NGHTTP2_CONNECT_ERROR: 10,
  NGHTTP2_ENHANCE_YOUR_CALM: 11,
  NGHTTP2_INADEQUATE_SECURITY: 12,
  NGHTTP2_HTTP_1_1_REQUIRED: 13,
  HTTP2_HEADER_STATUS: ':status',
  HTTP2_HEADER_METHOD: ':method',
  HTTP2_HEADER_PATH: ':path',
  HTTP2_HEADER_AUTHORITY: ':authority',
  HTTP2_HEADER_SCHEME: ':scheme',
  HTTP2_HEADER_PROTOCOL: ':protocol',
  HTTP2_HEADER_ACCEPT_ENCODING: 'accept-encoding',
  HTTP2_HEADER_ACCEPT_LANGUAGE: 'accept-language',
  HTTP2_HEADER_CACHE_CONTROL: 'cache-control',
  HTTP2_HEADER_CONTENT_LENGTH: 'content-length',
  HTTP2_HEADER_CONTENT_TYPE: 'content-type',
  HTTP2_HEADER_COOKIE: 'cookie',
  HTTP2_HEADER_DATE: 'date',
  HTTP2_HEADER_ETAG: 'etag',
  HTTP2_HEADER_LAST_MODIFIED: 'last-modified',
  HTTP2_HEADER_LINK: 'link',
  HTTP2_HEADER_LOCATION: 'location',
  HTTP2_HEADER_SERVER: 'server',
  HTTP2_HEADER_SET_COOKIE: 'set-cookie',
  HTTP2_HEADER_USER_AGENT: 'user-agent',
  HTTP2_HEADER_VARY: 'vary',
  HTTP2_METHOD_GET: 'GET',
  HTTP2_METHOD_HEAD: 'HEAD',
  HTTP2_METHOD_POST: 'POST',
  HTTP2_METHOD_PUT: 'PUT',
  HTTP2_METHOD_DELETE: 'DELETE',
  HTTP2_METHOD_OPTIONS: 'OPTIONS',
  HTTP2_METHOD_CONNECT: 'CONNECT',
  NGHTTP2_SETTINGS_HEADER_TABLE_SIZE: 1,
  NGHTTP2_SETTINGS_ENABLE_PUSH: 2,
  NGHTTP2_SETTINGS_MAX_CONCURRENT_STREAMS: 3,
  NGHTTP2_SETTINGS_INITIAL_WINDOW_SIZE: 4,
  NGHTTP2_SETTINGS_MAX_FRAME_SIZE: 5,
  NGHTTP2_SETTINGS_MAX_HEADER_LIST_SIZE: 6,
  NGHTTP2_SETTINGS_ENABLE_CONNECT_PROTOCOL: 8,
};

class Http2Session {
  constructor(type) {
    this._events = new Map();
    this._type = type;
    this._destroyed = false;
    this._connecting = false;
    this._pendingSettingsAck = false;
    this._localSettings = { ...kDefaultSettings };
    this._remoteSettings = { ...kDefaultSettings };
    this._streams = new Map();
    this._nextStreamId = 1;
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

  ping(payload, callback) {
    if (typeof payload === 'function') { callback = payload; payload = undefined; }
    if (!callback) return new Promise((resolve) => setTimeout(resolve, 0));
    queueMicrotask(() => callback(null, payload || Buffer.alloc(8)));
    return true;
  }

  settings(settings) {
    this._localSettings = { ...this._localSettings, ...settings };
    this._pendingSettingsAck = true;
    queueMicrotask(() => {
      this._pendingSettingsAck = false;
      this.emit('localSettings', this._localSettings);
    });
  }

  ref() { return this; }
  unref() { return this; }

  close() {
    if (this._destroyed) return;
    this._destroyed = true;
    for (const [id, stream] of this._streams) {
      stream.close(constants.NGHTTP2_CANCEL);
    }
    this._streams.clear();
    queueMicrotask(() => this.emit('close'));
  }

  destroy() { this.close(); }

  get destroyed() { return this._destroyed; }
  get connecting() { return this._connecting; }
  get pendingSettingsAck() { return this._pendingSettingsAck; }
  get localSettings() { return { ...this._localSettings }; }
  get remoteSettings() { return { ...this._remoteSettings }; }
  get type() { return this._type; }
}

class Http2Stream {
  constructor(session, id) {
    this._events = new Map();
    this._session = session;
    this._id = id;
    this._sentHeaders = null;
    this._rstCode = undefined;
    this._state = {
      state: constants.NGHTTP2_STREAM_STATE_IDLE,
      weight: 16,
      sumDependencyWeight: 0,
      localClose: false,
      remoteClose: false,
      localWindowSize: 65535,
    };
    this._headers = {};
    this._data = [];
    this._pushAllowed = true;
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

  respond(headers, options) {
    this._sentHeaders = { ...headers };
    this._state.state = constants.NGHTTP2_STREAM_STATE_HALF_CLOSED_LOCAL;
    queueMicrotask(() => {
      this.emit('response', headers);
    });
  }

  end(data) {
    if (data) {
      this._data.push(data);
    }
    this._state.state = constants.NGHTTP2_STREAM_STATE_HALF_CLOSED_LOCAL;
    this._state.localClose = true;
    queueMicrotask(() => {
      this.emit('close');
    });
    return this;
  }

  pushStream(headers, options, callback) {
    if (typeof options === 'function') { callback = options; options = {}; }
    const pushId = this._session._nextStreamId += 2;
    const pushStream = new Http2Stream(this._session, pushId);
    this._session._streams.set(pushId, pushStream);
    pushStream._headers = { ...headers };
    pushStream._state.state = constants.NGHTTP2_STREAM_STATE_RESERVED_LOCAL;
    if (callback) queueMicrotask(() => callback(null, pushStream));
    return pushStream;
  }

  rstStream(code) {
    this._rstCode = code || constants.NGHTTP2_NO_ERROR;
    this._state.state = constants.NGHTTP2_STREAM_STATE_CLOSED;
    this._state.localClose = true;
    this._state.remoteClose = true;
    queueMicrotask(() => this.emit('close'));
  }

  priority(options) {
    if (options.weight) this._state.weight = options.weight;
  }

  close() {
    this.rstStream(constants.NGHTTP2_CANCEL);
  }

  get id() { return this._id; }
  get sentHeaders() { return this._sentHeaders; }
  get pushAllowed() { return this._pushAllowed; }
  get rstCode() { return this._rstCode; }
  get state() { return { ...this._state }; }
  get session() { return this._session; }
}

function connect(url, options, callback) {
  if (typeof options === 'function') { callback = options; options = {}; }
  if (!callback) {
    return new Promise((resolve) => {
      const session = connect(url, options, (err, sess) => resolve(sess));
    });
  }
  const session = new Http2Session(constants.NGHTTP2_SESSION_CLIENT);
  session._connecting = true;
  queueMicrotask(() => {
    session._connecting = false;
    session.emit('connect', session, {});
    if (callback) callback(null, session);
  });
  return session;
}

function createServer(options, onRequestHandler) {
  if (typeof options === 'function') { onRequestHandler = options; options = {}; }
  const server = new (class Http2Server {
    constructor() {
      this._events = new Map();
      this._sessions = new Set();
      this._listening = false;
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

    listen(port, host, callback) {
      if (typeof host === 'function') { callback = host; host = '0.0.0.0'; }
      if (typeof port === 'object') {
        const opts = port;
        port = opts.port;
        host = opts.host || '0.0.0.0';
        callback = callback || opts.callback;
      }
      this._port = port || 0;
      this._host = host || '0.0.0.0';
      this._listening = true;
      this._server = {
        on: () => {},
        emit: () => {},
      };
      queueMicrotask(() => {
        this.emit('listening');
        if (callback) callback();
      });
      return this;
    }

    close(callback) {
      if (callback) this.on('close', callback);
      this._listening = false;
      for (const session of this._sessions) {
        session.close();
      }
      this._sessions.clear();
      queueMicrotask(() => this.emit('close'));
      return this;
    }

    setTimeout(msecs, callback) {
      if (callback) this.on('timeout', callback);
      return this;
    }

    get listening() { return this._listening; }

    emit(event, ...args) {
      const handlers = this._events.get(event) || [];
      for (const h of [...handlers]) {
        try { h.call(this, ...args); } catch (e) { queueMicrotask(() => { throw e; }); }
      }
      return true;
    }
  })();

  if (onRequestHandler) server.on('request', onRequestHandler);
  return server;
}

function createSecureServer(options, onRequestHandler) {
  return createServer(options, onRequestHandler);
}

class Http2ServerRequest {
  constructor(stream, headers) {
    this._stream = stream;
    this._headers = headers || {};
    this._method = (headers && headers[':method']) || 'GET';
    this._url = (headers && headers[':path']) || '/';
    this._httpVersion = '2.0';
    this._events = new Map();
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

  get headers() { return this._headers; }
  get httpVersion() { return this._httpVersion; }
  get method() { return this._method; }
  get url() { return this._url; }
  get stream() { return this._stream; }
}

class Http2ServerResponse {
  constructor(stream) {
    this._stream = stream;
    this._headers = {};
    this._statusCode = 200;
    this._headerSent = false;
    this._ended = false;
    this._chunks = [];
    this._events = new Map();
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

  setHeader(name, value) {
    this._headers[name.toLowerCase()] = value;
  }

  getHeader(name) {
    return this._headers[name.toLowerCase()];
  }

  getHeaders() {
    return { ...this._headers };
  }

  hasHeader(name) {
    return name.toLowerCase() in this._headers;
  }

  removeHeader(name) {
    delete this._headers[name.toLowerCase()];
  }

  write(chunk, encoding, callback) {
    if (typeof encoding === 'function') { callback = encoding; encoding = 'utf8'; }
    if (!this._headerSent) this._headerSent = true;
    this._chunks.push(chunk);
    if (callback) queueMicrotask(callback);
    return true;
  }

  end(data, encoding, callback) {
    if (typeof data === 'function') { callback = data; data = undefined; }
    if (typeof encoding === 'function') { callback = encoding; encoding = 'utf8'; }
    if (data) this.write(data, encoding);
    this._ended = true;
    const headers = { ':status': this._statusCode, ...this._headers };
    this._stream.respond(headers);
    this._stream.end(Buffer.concat(this._chunks.map(c => typeof c === 'string' ? new TextEncoder().encode(c) : c)));
    if (callback) queueMicrotask(callback);
    return this;
  }

  writeHead(statusCode, headers) {
    this._statusCode = statusCode;
    if (headers) {
      for (const [k, v] of Object.entries(headers)) {
        this.setHeader(k, v);
      }
    }
    this._headerSent = true;
    return this;
  }

  get stream() { return this._stream; }
  get statusCode() { return this._statusCode; }
  set statusCode(code) { this._statusCode = code; }
  get headersSent() { return this._headerSent; }
  get finished() { return this._ended; }
}

const http2 = {
  connect,
  createServer,
  createSecureServer,
  Http2Session,
  Http2Stream,
  Http2ServerRequest,
  Http2ServerResponse,
  getDefaultSettings: () => ({ ...kDefaultSettings }),
  getPackedSettings: (settings) => Buffer.from(JSON.stringify(settings)),
  getUnpackedSettings: (buf) => JSON.parse(new TextDecoder().decode(buf)),
  sensitiveHeaders: [],
  constants,
};

if (typeof module !== 'undefined' && module.exports) {
  module.exports = http2;
}
