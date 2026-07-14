// Klyron Runtime — Fetch API (Web compatible)
// fetch, Request, Response, Headers

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
