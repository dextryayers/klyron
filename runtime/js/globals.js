// Klyron Runtime — GlobalThis Bootstrapping
// Web + Node compatible globals

globalThis.global = globalThis;

// ── Timers ────────────────────────────────────────────────────────────────────

if (typeof globalThis.setTimeout !== 'function') {
  globalThis.setTimeout = (fn, ms, ...args) => {
    if (typeof fn === 'string') fn = new Function(fn);
    return Klyron.timers.setTimeout(fn, ms, ...args);
  };
}

if (typeof globalThis.clearTimeout !== 'function') {
  globalThis.clearTimeout = (id) => Klyron.timers.clearTimeout(id);
}

if (typeof globalThis.setInterval !== 'function') {
  globalThis.setInterval = (fn, ms, ...args) => {
    if (typeof fn === 'string') fn = new Function(fn);
    return Klyron.timers.setInterval(fn, ms, ...args);
  };
}

if (typeof globalThis.clearInterval !== 'function') {
  globalThis.clearInterval = (id) => Klyron.timers.clearInterval(id);
}

if (typeof globalThis.setImmediate !== 'function') {
  globalThis.setImmediate = (fn, ...args) => {
    if (typeof fn === 'string') fn = new Function(fn);
    return setTimeout(fn, 0, ...args);
  };
}

if (typeof globalThis.clearImmediate !== 'function') {
  globalThis.clearImmediate = (id) => clearTimeout(id);
}

// ── Microtasks & Async ────────────────────────────────────────────────────────

if (typeof globalThis.queueMicrotask !== 'function') {
  globalThis.queueMicrotask = (fn) => Promise.resolve().then(fn);
}

if (typeof globalThis.structuredClone !== 'function') {
  globalThis.structuredClone = (obj) => JSON.parse(JSON.stringify(obj));
}

// ── Buffer ────────────────────────────────────────────────────────────────────

if (typeof globalThis.TextEncoder !== 'function') {
  globalThis.TextEncoder = class TextEncoder {
    encode(str = '') {
      const u8 = new Uint8Array(str.length * 3);
      let len = 0;
      for (let i = 0; i < str.length; i++) {
        let c = str.charCodeAt(i);
        if (c < 0x80) { u8[len++] = c; }
        else if (c < 0x800) { u8[len++] = 0xC0 | (c >> 6); u8[len++] = 0x80 | (c & 0x3F); }
        else { u8[len++] = 0xE0 | (c >> 12); u8[len++] = 0x80 | ((c >> 6) & 0x3F); u8[len++] = 0x80 | (c & 0x3F); }
      }
      return u8.slice(0, len);
    }
    get encoding() { return 'utf-8'; }
  };
}

if (typeof globalThis.TextDecoder !== 'function') {
  globalThis.TextDecoder = class TextDecoder {
    constructor(label = 'utf-8') { this._label = label.toLowerCase(); }
    get encoding() { return this._label; }
    decode(buf, opts = {}) {
      const bytes = buf instanceof Uint8Array ? buf : new Uint8Array(buf);
      let out = '';
      for (let i = 0; i < bytes.length; i++) {
        if (bytes[i] < 0x80) { out += String.fromCharCode(bytes[i]); }
        else if (bytes[i] < 0xE0 && i + 1 < bytes.length) {
          out += String.fromCharCode((bytes[i] & 0x1F) << 6 | (bytes[i + 1] & 0x3F));
          i++;
        } else if (bytes[i] < 0xF0 && i + 2 < bytes.length) {
          out += String.fromCharCode((bytes[i] & 0x0F) << 12 | (bytes[i + 1] & 0x3F) << 6 | (bytes[i + 2] & 0x3F));
          i += 2;
        } else { i++; }
      }
      if (opts.stream !== true && bytes.length > 0 && out.length === 0) return '';
      return out;
    }
  };
}

if (typeof globalThis.atob !== 'function') {
  globalThis.atob = (str) => {
    str = str.replace(/[^A-Za-z0-9+/=]/g, '');
    let result = '';
    for (let i = 0; i < str.length; i += 4) {
      const c1 = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/'.indexOf(str[i]);
      const c2 = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/'.indexOf(str[i + 1]);
      const c3 = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/'.indexOf(str[i + 2]);
      const c4 = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/'.indexOf(str[i + 3]);
      result += String.fromCharCode((c1 << 2) | (c2 >> 4));
      if (c3 >= 0) result += String.fromCharCode(((c2 & 15) << 4) | (c3 >> 2));
      if (c4 >= 0) result += String.fromCharCode(((c3 & 3) << 6) | c4);
    }
    return result;
  };
}

if (typeof globalThis.btoa !== 'function') {
  globalThis.btoa = (str) => {
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';
    let result = '';
    const bytes = new TextEncoder().encode(str);
    for (let i = 0; i < bytes.length; i += 3) {
      const b1 = bytes[i];
      const b2 = i + 1 < bytes.length ? bytes[i + 1] : 0;
      const b3 = i + 2 < bytes.length ? bytes[i + 2] : 0;
      result += chars[b1 >> 2];
      result += chars[((b1 & 3) << 4) | (b2 >> 4)];
      result += i + 1 < bytes.length ? chars[((b2 & 15) << 2) | (b3 >> 6)] : '=';
      result += i + 2 < bytes.length ? chars[b3 & 63] : '=';
    }
    return result;
  };
}

if (typeof globalThis.Buffer === 'undefined' || typeof globalThis.Buffer !== 'function') {
  class Buffer extends Uint8Array {
    constructor(arg, encodingOrOffset, length) {
      if (typeof arg === 'number') super(arg);
      else if (typeof arg === 'string') {
        const enc = encodingOrOffset || 'utf8';
        if (enc === 'hex') { super(arg.length / 2); for (let i = 0; i < arg.length; i += 2) this[i/2] = parseInt(arg.substr(i, 2), 16); }
        else if (enc === 'base64') { const bin = atob(arg); super(bin.length); for (let i = 0; i < bin.length; i++) this[i] = bin.charCodeAt(i); }
        else { const u8 = new TextEncoder().encode(arg); super(u8); }
      } else { super(arg); }
    }
    toString(encoding = 'utf8', start = 0, end = this.length) {
      const s = this.subarray(start, end);
      if (encoding === 'hex') return Array.from(s).map(b => b.toString(16).padStart(2, '0')).join('');
      if (encoding === 'base64') return btoa(String.fromCharCode(...s));
      if (encoding === 'utf8' || encoding === 'utf-8') return new TextDecoder().decode(s);
      return new TextDecoder(encoding).decode(s);
    }
    toJSON() { return { type: 'Buffer', data: Array.from(this) }; }
    equals(other) { return this.length === other.length && this.every((v, i) => v === other[i]); }
    copy(target, targetStart = 0, sourceStart = 0, sourceEnd = this.length) {
      const copied = Math.min(sourceEnd - sourceStart, target.length - targetStart);
      for (let i = 0; i < copied; i++) target[targetStart + i] = this[sourceStart + i];
      return copied;
    }
    slice(start = 0, end = this.length) { return Buffer.from(super.slice(start, end)); }
    subarray(start = 0, end = this.length) { return Buffer.from(super.subarray(start, end)); }
    write(str, offset = 0, length = str.length, encoding = 'utf8') {
      const bytes = new TextEncoder().encode(str).slice(0, Math.min(length, this.length - offset));
      this.set(bytes, offset);
      return bytes.length;
    }
    static from(value, encodingOrOffset, length) { return new Buffer(value, encodingOrOffset, length); }
    static alloc(size, fill = 0) {
      const b = new Buffer(size);
      if (fill !== 0) b.fill(fill);
      return b;
    }
    static allocUnsafe(size) { return new Buffer(size); }
    static byteLength(str, encoding = 'utf8') {
      if (typeof str === 'string') return new TextEncoder().encode(str).length;
      return str.length;
    }
    static isBuffer(obj) { return obj instanceof Buffer; }
    static concat(list, totalLength) {
      if (!totalLength) totalLength = list.reduce((s, b) => s + b.length, 0);
      const buf = Buffer.alloc(totalLength);
      let offset = 0;
      for (const b of list) { buf.set(b, offset); offset += b.length; }
      return buf;
    }
    static isEncoding(enc) { return ['utf8', 'utf-8', 'hex', 'base64', 'ascii', 'latin1'].includes(enc?.toLowerCase()); }
    entries() { return super.entries(); }
    keys() { return super.keys(); }
    values() { return super.values(); }
  }
  Object.defineProperty(Buffer.prototype, 'parent', { get() { return this.buffer; } });
  Object.defineProperty(Buffer.prototype, 'offset', { get() { return this.byteOffset; } });
  globalThis.Buffer = Buffer;
}

// ── atob / btoa ───────────────────────────────────────────────────────────────

// (already defined above)

// ── Performance ───────────────────────────────────────────────────────────────

if (typeof globalThis.performance !== 'object') {
  class Performance {
    constructor() {
      this._marks = new Map();
      this._measures = new Map();
    }
    now() {
      const [sec, nsec] = Klyron.hrtime();
      return sec * 1000 + nsec / 1e6;
    }
    mark(name) { this._marks.set(name, this.now()); }
    measure(name, startMark, endMark) {
      const start = startMark ? this._marks.get(startMark) : 0;
      const end = endMark ? this._marks.get(endMark) : this.now();
      const duration = end - start;
      this._measures.set(name, { name, duration, startTime: start, entryType: 'measure' });
      return duration;
    }
    clearMarks(name) { if (name) this._marks.delete(name); else this._marks.clear(); }
    clearMeasures(name) { if (name) this._measures.delete(name); else this._measures.clear(); }
    getEntriesByType(type) {
      if (type === 'mark') return Array.from(this._marks.entries()).map(([n, t]) => ({ name: n, startTime: t, entryType: 'mark', duration: 0 }));
      if (type === 'measure') return Array.from(this._measures.values());
      return [];
    }
    toJSON() { return {}; }
  }
  globalThis.performance = new Performance();
}

// ── Event, EventTarget ────────────────────────────────────────────────────────

class Event {
  constructor(type, opts = {}) {
    this._type = type;
    this._bubbles = opts.bubbles || false;
    this._cancelable = opts.cancelable || false;
    this._composed = opts.composed || false;
    this._defaultPrevented = false;
    this._propagationStopped = false;
    this._immediatePropagationStopped = false;
    this._target = null;
    this._currentTarget = null;
    this._timeStamp = Date.now();
  }
  get type() { return this._type; }
  get target() { return this._target; }
  get currentTarget() { return this._currentTarget; }
  get bubbles() { return this._bubbles; }
  get cancelable() { return this._cancelable; }
  get defaultPrevented() { return this._defaultPrevented; }
  get composed() { return this._composed; }
  get timeStamp() { return this._timeStamp; }
  get eventPhase() { return 0; }
  get srcElement() { return this._target; }
  get returnValue() { return !this._defaultPrevented; }
  get cancelBubble() { return this._propagationStopped; }
  set cancelBubble(v) { if (v) this._propagationStopped = true; }
  composedPath() { return []; }
  preventDefault() { if (this._cancelable) this._defaultPrevented = true; }
  stopPropagation() { this._propagationStopped = true; }
  stopImmediatePropagation() { this._immediatePropagationStopped = true; }
  initEvent(type, bubbles, cancelable) {
    this._type = type;
    this._bubbles = !!bubbles;
    this._cancelable = !!cancelable;
  }
}

class CustomEvent extends Event {
  constructor(type, opts = {}) {
    super(type, opts);
    this._detail = opts.detail || null;
  }
  get detail() { return this._detail; }
}

class EventTarget {
  constructor() { this._listeners = new Map(); }
  addEventListener(type, callback, options) {
    if (!callback) return;
    const capture = typeof options === 'object' ? !!options.capture : !!options;
    const once = typeof options === 'object' ? !!options.once : false;
    const key = type + ':' + capture;
    if (!this._listeners.has(key)) this._listeners.set(key, []);
    this._listeners.get(key).push({ callback, once, passive: typeof options === 'object' ? !!options.passive : false, signal: options?.signal || null });
  }
  removeEventListener(type, callback, options) {
    const capture = typeof options === 'object' ? !!options.capture : !!options;
    const key = type + ':' + capture;
    const listeners = this._listeners.get(key);
    if (!listeners) return;
    this._listeners.set(key, listeners.filter(l => l.callback !== callback));
  }
  dispatchEvent(event) {
    event._target = this;
    event._currentTarget = this;
    const key = event.type + ':false';
    const listeners = [...(this._listeners.get(key) || [])];
    for (const l of listeners) {
      if (event._immediatePropagationStopped) break;
      try {
        if (typeof l.callback === 'function') l.callback.call(this, event);
        else if (typeof l.callback.handleEvent === 'function') l.callback.handleEvent(event);
      } catch (e) {
        queueMicrotask(() => { throw e; });
      }
      if (l.once) this.removeEventListener(event.type, l.callback, false);
      if (l.signal && l.signal.aborted) this.removeEventListener(event.type, l.callback, false);
    }
    return !event._defaultPrevented;
  }
}

if (typeof globalThis.Event !== 'function') {
  globalThis.EventTarget = EventTarget;
  globalThis.Event = Event;
  globalThis.CustomEvent = CustomEvent;
}

// ── AbortController, AbortSignal ──────────────────────────────────────────────

class AbortSignal extends EventTarget {
  constructor() {
    super();
    this._aborted = false;
    this._reason = undefined;
  }
  get aborted() { return this._aborted; }
  get reason() { return this._reason; }
  static abort(reason) {
    const signal = new AbortSignal();
    signal._aborted = true;
    signal._reason = reason || new DOMException('The operation was aborted', 'AbortError');
    return signal;
  }
  static timeout(ms) {
    const signal = new AbortSignal();
    setTimeout(() => {
      signal._aborted = true;
      signal._reason = new DOMException('The operation timed out', 'TimeoutError');
      signal.dispatchEvent(new Event('abort'));
    }, ms);
    return signal;
  }
  onabort() { this.addEventListener('abort', this._onabort); }
}

class AbortController {
  constructor() {
    this._signal = new AbortSignal();
    this._signal._controller = this;
  }
  get signal() { return this._signal; }
  abort(reason) {
    if (this._signal._aborted) return;
    this._signal._aborted = true;
    this._signal._reason = reason || new DOMException('The operation was aborted', 'AbortError');
    this._signal.dispatchEvent(new Event('abort'));
  }
}

if (typeof globalThis.AbortController !== 'function') {
  globalThis.AbortController = AbortController;
  globalThis.AbortSignal = AbortSignal;
}

// ── URL, URLSearchParams ──────────────────────────────────────────────────────

class URLSearchParams {
  constructor(init = '') {
    this._params = new Map();
    if (typeof init === 'string') {
      init = init.replace(/^[?#]/, '');
      for (const pair of init.split('&')) {
        if (!pair) continue;
        const [k, v] = pair.split('=').map(d => decodeURIComponent(d.replace(/\+/g, ' ')));
        if (!this._params.has(k)) this._params.set(k, []);
        this._params.get(k).push(v);
      }
    } else if (init && typeof init.forEach === 'function') {
      init.forEach((v, k) => this.append(k, v));
    } else if (typeof init === 'object') {
      for (const [k, v] of Object.entries(init)) this.append(k, v);
    }
  }
  append(k, v) { if (!this._params.has(k)) this._params.set(k, []); this._params.get(k).push(String(v)); }
  delete(k) { this._params.delete(k); }
  get(k) { const a = this._params.get(k); return a ? a[0] : null; }
  getAll(k) { return this._params.get(k) || []; }
  has(k) { return this._params.has(k); }
  set(k, v) { this._params.set(k, [String(v)]); }
  sort() { this._params = new Map([...this._params].sort((a, b) => a[0].localeCompare(b[0]))); }
  forEach(fn) { this._params.forEach((vals, k) => { for (const v of vals) fn(v, k, this); }); }
  keys() { return this._params.keys(); }
  values() { return Array.from(this._params.values()).flat()[Symbol.iterator](); }
  entries() { return Array.from(this._params.entries()).flatMap(([k, vs]) => vs.map(v => [k, v]))[Symbol.iterator](); }
  [Symbol.iterator]() { return this.entries(); }
  toString() {
    return Array.from(this._params.entries()).flatMap(([k, vs]) => vs.map(v => `${encodeURIComponent(k)}=${encodeURIComponent(v)}`)).join('&');
  }
}

class URLPattern {
  constructor(input, baseURL) {
    this._input = input;
    this._base = baseURL || '';
  }
  test(url) { return true; }
  exec(url) { return null; }
  get protocol() { return ''; }
  get username() { return ''; }
  get password() { return ''; }
  get hostname() { return ''; }
  get port() { return ''; }
  get pathname() { return ''; }
  get search() { return ''; }
  get hash() { return ''; }
}

class URL {
  constructor(url, base) {
    if (base && !base.startsWith('http://') && !base.startsWith('https://') && !base.startsWith('file://')) {
      if (base.startsWith('/')) base = 'file://' + base;
      else base = 'file:///' + base;
    }
    let resolved = base ? new URL(base)._resolve(url) : url;
    const parsed = /^(https?:|file:)?\/\/([^\/?#:]+)(?::(\d+))?([^?#]*)(\?[^#]*)?(#.*)?/.exec(resolved);
    if (!parsed) {
      if (url.startsWith('/')) {
        this._pathname = url;
        this._href = 'file://' + url;
        return;
      }
      throw new TypeError('Invalid URL: ' + url);
    }
    this._protocol = parsed[1] || 'file:';
    this._hostname = parsed[2] || '';
    this._port = parsed[3] || '';
    this._pathname = parsed[4] || '/';
    this._search = parsed[5] || '';
    this._hash = parsed[6] || '';
    this._host = this._hostname + (this._port ? ':' + this._port : '');
    this._origin = this._protocol + '//' + this._host;
    this._href = this._protocol + '//' + this._host + this._pathname + this._search + this._hash;
    this._searchParams = new URLSearchParams(this._search);
  }
  _resolve(rel) {
    if (rel.startsWith('http://') || rel.startsWith('https://') || rel.startsWith('file://')) return rel;
    if (rel.startsWith('/')) return this._protocol + '//' + this._host + rel;
    const basePath = this._pathname.substring(0, this._pathname.lastIndexOf('/') + 1);
    const parts = (basePath + rel).split('/');
    const result = [];
    for (const p of parts) {
      if (p === '.' || p === '') continue;
      if (p === '..') result.pop();
      else result.push(p);
    }
    return this._protocol + '//' + this._host + '/' + result.join('/');
  }
  get protocol() { return this._protocol; }
  get hostname() { return this._hostname; }
  get port() { return this._port; }
  get pathname() { return this._pathname; }
  get search() { return this._search; }
  get hash() { return this._hash; }
  get host() { return this._host; }
  get origin() { return this._origin; }
  get href() { return this._href; }
  get searchParams() { return this._searchParams; }
  toString() { return this._href; }
  toJSON() { return this._href; }
}

if (typeof globalThis.URL !== 'function') {
  globalThis.URL = URL;
  globalThis.URLSearchParams = URLSearchParams;
  globalThis.URLPattern = URLPattern;
}

// ── Headers, Request, Response, fetch ────────────────────────────────────────

class Headers {
  constructor(init = {}) {
    this._map = new Map();
    if (typeof init.forEach === 'function') init.forEach((v, k) => this.set(k, v));
    else if (typeof init === 'object') Object.entries(init).forEach(([k, v]) => this.set(k, v));
  }
  get(name) { return this._map.get(name.toLowerCase()) || null; }
  set(name, value) { this._map.set(name.toLowerCase(), String(value)); }
  has(name) { return this._map.has(name.toLowerCase()); }
  delete(name) { return this._map.delete(name.toLowerCase()); }
  forEach(fn) { this._map.forEach((v, k) => fn(v, k, this)); }
  keys() { return this._map.keys(); }
  values() { return this._map.values(); }
  entries() { return this._map.entries(); }
  [Symbol.iterator]() { return this._map[Symbol.iterator](); }
  get [Symbol.toStringTag]() { return 'Headers'; }
}

class Request {
  constructor(input, init = {}) {
    if (input instanceof Request) {
      this._url = input.url;
      this._method = init.method || input.method;
      this._headers = new Headers(init.headers || input.headers);
      this._body = init.body ?? input._body;
    } else {
      this._url = String(input);
      this._method = (init.method || 'GET').toUpperCase();
      this._headers = new Headers(init.headers || {});
      this._body = init.body ?? null;
    }
  }
  get url() { return this._url; }
  get method() { return this._method; }
  get headers() { return this._headers; }
  get body() { return this._body; }
}

class Response {
  constructor(body, init = {}) {
    this._body = body;
    this._status = init.status || 200;
    this._statusText = init.statusText || '';
    this._headers = new Headers(init.headers || {});
    this._url = init.url || '';
    this._type = 'basic';
  }
  get ok() { return this._status >= 200 && this._status < 300; }
  get status() { return this._status; }
  get statusText() { return this._statusText; }
  get headers() { return this._headers; }
  get url() { return this._url; }
  get type() { return this._type; }
  async text() { return String(this._body || ''); }
  async json() { return JSON.parse(this._body || '{}'); }
  async blob() { return new Blob([this._body || '']); }
  async arrayBuffer() {
    const enc = new TextEncoder();
    return enc.encode(this._body || '').buffer;
  }
}

async function fetch(input, init = {}) {
  const request = new Request(input, init);
  try {
    const result = Klyron.net.fetch({
      method: request.method,
      url: request.url,
      headers: Object.fromEntries(request.headers.entries()),
      body: request.body,
    });
    return new Response(result.body, {
      status: result.status,
      statusText: result.statusText,
      headers: result.headers,
      url: result.url,
    });
  } catch (err) {
    throw new Error(`fetch failed: ${err.message}`);
  }
}

if (typeof globalThis.fetch !== 'function') {
  globalThis.fetch = fetch;
  globalThis.Request = Request;
  globalThis.Response = Response;
  globalThis.Headers = Headers;
}

// ── Console ───────────────────────────────────────────────────────────────────

if (typeof globalThis.console !== 'object') {
  class Console {
    constructor(stdout, stderr) {
      this._stdout = stdout || Klyron.io.stdout;
      this._stderr = stderr || Klyron.io.stderr;
      this._times = new Map();
      this._counters = new Map();
    }
    log(...args) { this._write(this._stdout, args); }
    info(...args) { this._write(this._stdout, args); }
    warn(...args) { this._write(this._stderr, args); }
    error(...args) { this._write(this._stderr, args); }
    debug(...args) { this._write(this._stdout, args); }
    table(data) {
      if (!data || typeof data !== 'object') return this.log(data);
      const isArray = Array.isArray(data);
      const keys = isArray ? Object.keys(data[0] || {}) : Object.keys(data);
      if (keys.length === 0) return this.log(data);
      const rows = isArray ? data : [data];
      const colWidths = keys.map(k => Math.max(k.length, ...rows.map(r => String(r[k] || '').length)));
      const sep = '+' + colWidths.map(w => '-'.repeat(w + 2)).join('+') + '+';
      const header = '| ' + keys.map((k, i) => k.padEnd(colWidths[i])).join(' | ') + ' |';
      this._stdout.write(sep + '\n');
      this._stdout.write(header + '\n');
      this._stdout.write(sep + '\n');
      for (const row of rows) {
        const line = '| ' + keys.map((k, i) => String(row[k] || '').padEnd(colWidths[i])).join(' | ') + ' |';
        this._stdout.write(line + '\n');
      }
      this._stdout.write(sep + '\n');
    }
    time(label = 'default') { this._times.set(label, Date.now()); }
    timeLog(label = 'default') {
      const start = this._times.get(label);
      if (start) this.log(`${label}: ${Date.now() - start}ms`);
    }
    timeEnd(label = 'default') {
      const start = this._times.get(label);
      if (start) { this.log(`${label}: ${Date.now() - start}ms`); this._times.delete(label); }
    }
    count(label = 'default') {
      this._counters.set(label, (this._counters.get(label) || 0) + 1);
      this.log(`${label}: ${this._counters.get(label)}`);
    }
    countReset(label = 'default') { this._counters.delete(label); }
    group(...args) { if (args.length) this.log(...args); this._indent = (this._indent || 0) + 1; }
    groupEnd() { this._indent = Math.max(0, (this._indent || 1) - 1); }
    trace() { this.error(new Error().stack); }
    dir(obj) { this.log(JSON.stringify(obj, null, 2)); }
    assert(condition, ...args) { if (!condition) throw new Error(`Assertion failed: ${args.join(' ')}`); }
    _write(stream, args) {
      const indent = '  '.repeat(this._indent || 0);
      stream.write(indent + args.map(a => typeof a === 'object' ? JSON.stringify(a, null, 2) : String(a)).join(' ') + '\n');
    }
  }
  globalThis.console = new Console();
}

// ── process global ────────────────────────────────────────────────────────────

if (typeof globalThis.process === 'undefined') {
  const kProcess = Klyron.process;
  const process = {
    pid: kProcess.pid,
    ppid: kProcess.ppid,
    platform: kProcess.platform,
    arch: kProcess.arch,
    version: kProcess.version,
    versions: kProcess.versions,
    env: kProcess.env(),
    argv: kProcess.argv(),
    cwd: () => kProcess.cwd(),
    chdir: (dir) => {},
    exit: (code) => kProcess.exit(code || 0),
    nextTick: (fn, ...args) => {
      if (typeof fn !== 'function') throw new TypeError('Callback must be a function');
      queueMicrotask(() => fn(...args));
    },
    hrtime: (time) => {
      const [sec, nsec] = Klyron.hrtime();
      if (time) {
        const [s, ns] = Array.isArray(time) ? time : [time.sec || 0, time.nsec || 0];
        const ds = sec - s;
        const dns = nsec - ns;
        return dns < 0 ? [ds - 1, dns + 1000000000] : [ds, dns];
      }
      return [sec, Math.floor(nsec / 1000)];
    },
    uptime: () => Klyron.hrtime()[0],
    memoryUsage: () => ({ rss: 0, heapTotal: 0, heapUsed: 0, external: 0, arrayBuffers: 0 }),
    cpuUsage: () => ({ user: 0, system: 0 }),
    title: 'klyron',
    stdout: Klyron.io.stdout,
    stderr: Klyron.io.stderr,
    stdin: Klyron.io.stdin,
    argv0: kProcess.argv()[0] || '',
    execPath: '/usr/local/bin/klyron',
    config: { target_defaults: {}, variables: {} },
    features: {},
    release: { name: 'klyron', sourceUrl: '', headersUrl: '', libUrl: '', lts: '' },
    _exiting: false,
    reallyExit: (code) => kProcess.exit(code || 0),
    binding: () => { throw new Error('process.binding is not supported'); },
    dlopen: () => { throw new Error('process.dlopen is not supported'); },
    umask: () => 0o022,
    getuid: () => 1000,
    getgid: () => 1000,
    geteuid: () => 1000,
    getegid: () => 1000,
    setuid: () => {},
    setgid: () => {},
    setegid: () => {},
    seteuid: () => {},
    initgroups: () => {},
    uptime: () => 0,
    emitWarning: (warning, type, code) => {
      console.warn(`[${type || 'Warning'}]${code ? '(' + code + ') ' : ' '}${warning}`);
    },
    emit: (event, ...args) => {
      if (event === 'warning') console.warn(...args);
      return true;
    },
    on: (event, listener) => {},
    once: (event, listener) => {},
    listeners: (event) => [],
    removeAllListeners: (event) => {},
    hasUncaughtExceptionCaptureCallback: () => false,
    setUncaughtExceptionCaptureCallback: () => {},
    domain: null,
    EventEmitter: EventTarget,
  };
  globalThis.process = process;
}

// ── __dirname, __filename ────────────────────────────────────────────────────

if (typeof globalThis.__dirname === 'undefined') {
  globalThis.__dirname = '/';
}

if (typeof globalThis.__filename === 'undefined') {
  globalThis.__filename = '/index.js';
}

// ── require (CommonJS) ────────────────────────────────────────────────────────

if (typeof globalThis.require !== 'function') {
  const moduleCache = new Map();

  function resolveModule(specifier, parentPath) {
    if (specifier.startsWith('node:')) {
      return 'node:' + specifier.slice(5);
    }
    if (specifier.startsWith('./') || specifier.startsWith('../')) {
      const parentDir = parentPath ? parentPath.substring(0, parentPath.lastIndexOf('/')) : __dirname;
      const resolved = resolvePath(parentDir, specifier);
      return resolved;
    }
    if (specifier.startsWith('/')) {
      return specifier;
    }
    return 'node_modules/' + specifier;
  }

  function resolvePath(base, relative) {
    const parts = relative.split('/');
    const baseParts = base.split('/');
    for (const part of parts) {
      if (part === '.') continue;
      if (part === '..') { if (baseParts.length > 0) baseParts.pop(); }
      else baseParts.push(part);
    }
    return baseParts.join('/');
  }

  const nodeBuiltins = {
    'fs': './node_fs.js',
    'path': './node_path.js',
    'buffer': './node_buffer.js',
    'events': './node_events.js',
    'stream': './node_stream.js',
    'http': './node_http.js',
    'https': './node_https.js',
    'crypto': './node_crypto.js',
    'os': './node_os.js',
    'child_process': './node_child_process.js',
    'timers': './timers.js',
    'console': './console.js',
    'url': './url.js',
    'querystring': './querystring.js',
    'assert': './assert.js',
    'util': './util.js',
    'string_decoder': './string_decoder.js',
  };

  function require(specifier) {
    const cacheKey = String(specifier);
    if (moduleCache.has(cacheKey)) {
      return moduleCache.get(cacheKey);
    }

    if (nodeBuiltins[cacheKey]) {
      try {
        const mod = require('./' + nodeBuiltins[cacheKey]);
        moduleCache.set(cacheKey, mod);
        return mod;
      } catch (e) {
        moduleCache.set(cacheKey, {});
        return {};
      }
    }

    const path = specifier.endsWith('.js') || specifier.endsWith('.json') ? specifier : specifier + '.js';

    try {
      let loaded = false;
      let exports = {};

      if (typeof Klyron_core !== 'undefined' && typeof Klyron_core.readFileSync === 'function') {
        try {
          const content = Klyron_core.readFileSync(path);
          const code = new TextDecoder().decode(content);
          const wrapped = new Function('exports', 'require', 'module', '__filename', '__dirname', code);
          const mod = { exports };
          wrapped(exports, require, mod, path, path.substring(0, path.lastIndexOf('/')) || '/');
          exports = mod.exports;
          loaded = true;
        } catch (e) {
          if (!e.message.includes('ENOENT')) throw e;
        }
      }

      if (!loaded) {
        const kFs = Klyron && Klyron.fs;
        if (kFs && typeof kFs.readFileSync === 'function') {
          const content = kFs.readFileSync(path);
          const code = new TextDecoder().decode(content);
          const wrapped = new Function('exports', 'require', 'module', '__filename', '__dirname', code);
          const mod = { exports };
          wrapped(exports, require, mod, path, path.substring(0, path.lastIndexOf('/')) || '/');
          exports = mod.exports;
        } else {
          exports = {};
        }
      }

      moduleCache.set(cacheKey, exports);
      return exports;
    } catch (e) {
      moduleCache.set(cacheKey, {});
      return {};
    }
  }

  require.resolve = (specifier) => specifier;
  require.cache = moduleCache;
  require.extensions = { '.js': () => {}, '.json': () => {}, '.node': () => {} };
  require.main = { filename: __filename, paths: [] };

  globalThis.require = require;
}

// ── crypto (Web) ──────────────────────────────────────────────────────────────

if (typeof globalThis.crypto !== 'object') {
  class SubtleCrypto {
    async digest(algorithm, data) {
      const algo = typeof algorithm === 'string' ? algorithm.toUpperCase() : algorithm.name.toUpperCase();
      const buf = data instanceof ArrayBuffer ? new Uint8Array(data) : new Uint8Array(data);
      return Klyron.crypto.digest(algo, buf).buffer;
    }
    async randomUUID() { return Klyron.crypto.randomUUID(); }
  }
  class Crypto {
    constructor() { this.subtle = new SubtleCrypto(); }
    getRandomValues(array) {
      if (!(array instanceof Uint8Array || array instanceof Int8Array ||
            array instanceof Uint16Array || array instanceof Int16Array ||
            array instanceof Uint32Array || array instanceof Int32Array)) {
        throw new TypeError('Expected integer TypedArray');
      }
      const bytes = Klyron.crypto.randomBytes(array.byteLength);
      array.set(new Uint8Array(bytes));
      return array;
    }
    randomUUID() { return Klyron.crypto.randomUUID(); }
  }
  globalThis.crypto = new Crypto();
}

// ── WebSocket ─────────────────────────────────────────────────────────────────

const CONNECTING = 0;
const OPEN = 1;
const CLOSING = 2;
const CLOSED = 3;

if (typeof globalThis.WebSocket !== 'function') {
  class WebSocket extends EventTarget {
    #url;
    #readyState = CONNECTING;
    #onopen = null;
    #onmessage = null;
    #onerror = null;
    #onclose = null;
    #closed = false;
    constructor(url) {
      super();
      this.#url = url;
      queueMicrotask(() => {
        this.#readyState = OPEN;
        this.#dispatchEvent(new Event('open'));
      });
    }
    get url() { return this.#url; }
    get readyState() { return this.#readyState; }
    get protocol() { return ''; }
    get extensions() { return ''; }
    get bufferedAmount() { return 0; }
    get binaryType() { return 'blob'; }
    set binaryType(v) {}
    get onopen() { return this.#onopen; }
    set onopen(fn) { this.#onopen = fn; if (fn) this.addEventListener('open', fn); }
    get onmessage() { return this.#onmessage; }
    set onmessage(fn) { this.#onmessage = fn; if (fn) this.addEventListener('message', fn); }
    get onerror() { return this.#onerror; }
    set onerror(fn) { this.#onerror = fn; if (fn) this.addEventListener('error', fn); }
    get onclose() { return this.#onclose; }
    set onclose(fn) { this.#onclose = fn; if (fn) this.addEventListener('close', fn); }
    send(data) {
      if (this.#readyState !== OPEN) throw new Error('WebSocket is not open');
    }
    close(code, reason) {
      if (this.#closed) return;
      this.#closed = true;
      this.#readyState = CLOSED;
      this.#dispatchEvent(new CloseEvent('close', { code: code || 1000, reason: reason || '', wasClean: true }));
    }
  }
  class CloseEvent extends Event {
    constructor(type, opts = {}) {
      super(type, opts);
      this.code = opts.code ?? 1005;
      this.reason = opts.reason ?? '';
      this.wasClean = opts.wasClean ?? true;
    }
  }
  class MessageEvent extends Event {
    constructor(type, opts = {}) {
      super(type, opts);
      this.data = opts.data ?? null;
      this.origin = opts.origin ?? '';
      this.lastEventId = opts.lastEventId ?? '';
      this.source = opts.source ?? null;
      this.ports = opts.ports ?? [];
    }
  }
  class ErrorEvent extends Event {
    constructor(type, opts = {}) {
      super(type, opts);
      this.message = opts.message ?? '';
      this.error = opts.error ?? null;
    }
  }
  globalThis.WebSocket = WebSocket;
  globalThis.CloseEvent = CloseEvent;
  globalThis.MessageEvent = MessageEvent;
  globalThis.ErrorEvent = ErrorEvent;
}

// ── ReadableStream, WritableStream, TransformStream ──────────────────────────

if (typeof globalThis.ReadableStream !== 'function') {
  class ReadableStreamDefaultController {
    constructor(stream) {
      this._stream = stream;
      this._queue = [];
      this._requests = [];
    }
    enqueue(chunk) {
      if (this._requests.length > 0) {
        this._requests.shift()({ value: chunk, done: false });
      } else {
        this._queue.push(chunk);
      }
    }
    close() {
      for (const r of this._requests) r({ value: undefined, done: true });
      this._requests = [];
      this._stream._state = 'closed';
    }
    error(e) {
      for (const r of this._requests) r(Promise.reject(e));
      this._requests = [];
      this._stream._state = 'errored';
    }
  }
  class ReadableStreamDefaultReader {
    constructor(stream) {
      this._stream = stream;
      stream._reader = this;
    }
    async read() {
      if (this._stream._controller._queue.length > 0) {
        return { value: this._stream._controller._queue.shift(), done: false };
      }
      if (this._stream._state === 'closed') return { value: undefined, done: true };
      return new Promise(resolve => this._stream._controller._requests.push(resolve));
    }
    releaseLock() { this._stream._reader = null; }
    async next() { return this.read(); }
  }
  class ReadableStream {
    constructor(underlyingSource = {}) {
      this._state = 'readable';
      this._controller = new ReadableStreamDefaultController(this);
      this._reader = null;
      if (underlyingSource.start) underlyingSource.start(this._controller);
    }
    get locked() { return this._reader !== null; }
    getReader() { return new ReadableStreamDefaultReader(this); }
    cancel(reason) { this._state = 'closed'; return Promise.resolve(); }
    [Symbol.asyncIterator]() { return this.getReader(); }
  }
  class WritableStreamDefaultWriter {
    constructor(stream) { this._stream = stream; }
    async write(chunk) { if (this._stream._sink.write) await this._stream._sink.write(chunk); }
    async close() { if (this._stream._sink.close) await this._stream._sink.close(); this._stream._state = 'closed'; }
    async abort(reason) { if (this._stream._sink.abort) await this._stream._sink.abort(reason); }
  }
  class WritableStream {
    constructor(underlyingSink = {}) {
      this._state = 'writable';
      this._sink = underlyingSink;
      if (underlyingSink.start) underlyingSink.start(this);
    }
    get locked() { return false; }
    getWriter() { return new WritableStreamDefaultWriter(this); }
  }
  class TransformStream {
    constructor(transformer = {}) {
      this._readable = new ReadableStream({ start: (c) => { this._readableController = c; } });
      this._writable = new WritableStream({
        write: async (chunk) => { if (transformer.transform) await transformer.transform(chunk, this._readableController); },
        close: () => { if (transformer.flush) transformer.flush(this._readableController); this._readableController.close(); },
      });
    }
    get readable() { return this._readable; }
    get writable() { return this._writable; }
  }
  globalThis.ReadableStream = ReadableStream;
  globalThis.WritableStream = WritableStream;
  globalThis.TransformStream = TransformStream;
}

// ── BroadcastChannel ──────────────────────────────────────────────────────────

if (typeof globalThis.BroadcastChannel === 'undefined') {
  class BroadcastChannel extends EventTarget {
    constructor(name) {
      super();
      this._name = name;
    }
    get name() { return this._name; }
    postMessage(msg) {
      this.dispatchEvent(new MessageEvent('message', { data: msg }));
    }
    close() { this._listeners.clear(); }
  }
  globalThis.BroadcastChannel = BroadcastChannel;
}

// ── MessageChannel, MessagePort ──────────────────────────────────────────────

if (typeof globalThis.MessageChannel === 'undefined') {
  class MessagePort extends EventTarget {
    constructor() {
      super();
      this._other = null;
    }
    postMessage(msg) {
      if (this._other) {
        queueMicrotask(() => {
          this._other.dispatchEvent(new MessageEvent('message', { data: msg }));
        });
      }
    }
    start() {}
    close() {}
  }
  class MessageChannel {
    constructor() {
      this.port1 = new MessagePort();
      this.port2 = new MessagePort();
      this.port1._other = this.port2;
      this.port2._other = this.port1;
    }
  }
  globalThis.MessageChannel = MessageChannel;
  globalThis.MessagePort = MessagePort;
}

// ── Blob ──────────────────────────────────────────────────────────────────────

if (typeof globalThis.Blob === 'undefined') {
  class Blob {
    constructor(parts = [], opts = {}) {
      this._parts = parts;
      this._type = opts.type || '';
    }
    get size() {
      return this._parts.reduce((s, p) => {
        if (typeof p === 'string') return s + new TextEncoder().encode(p).length;
        if (p instanceof Uint8Array) return s + p.length;
        if (p instanceof ArrayBuffer) return s + p.byteLength;
        return s;
      }, 0);
    }
    get type() { return this._type; }
    async text() {
      let result = '';
      for (const p of this._parts) {
        if (typeof p === 'string') result += p;
        else if (p instanceof Uint8Array) result += new TextDecoder().decode(p);
        else if (p instanceof ArrayBuffer) result += new TextDecoder().decode(new Uint8Array(p));
      }
      return result;
    }
    async arrayBuffer() {
      const bufs = [];
      for (const p of this._parts) {
        if (typeof p === 'string') bufs.push(new TextEncoder().encode(p));
        else if (p instanceof Uint8Array) bufs.push(p);
        else if (p instanceof ArrayBuffer) bufs.push(new Uint8Array(p));
      }
      const total = bufs.reduce((s, b) => s + b.length, 0);
      const result = new Uint8Array(total);
      let offset = 0;
      for (const b of bufs) { result.set(b, offset); offset += b.length; }
      return result.buffer;
    }
  }
  globalThis.Blob = Blob;
}
