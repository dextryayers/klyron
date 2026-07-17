import {
  op_web_fetch_ex,
  op_web_text_encode,
  op_web_text_decode,
  op_web_base64_encode,
  op_web_base64_decode,
} from "ext:core/ops";

// -- Headers ------------------------------------------------

export class Headers {
  constructor(init) {
    this._map = new Map();
    if (init) {
      if (init instanceof Headers) {
        for (const [k, v] of init._map) this._map.set(k.toLowerCase(), v);
      } else if (Array.isArray(init)) {
        for (const [k, v] of init) this.append(k, v);
      } else if (typeof init === "object") {
        for (const k in init) this.append(k, init[k]);
      }
    }
  }

  append(name, value) {
    const key = String(name).toLowerCase();
    const existing = this._map.get(key);
    this._map.set(key, existing ? `${existing}, ${value}` : String(value));
  }

  set(name, value) {
    this._map.set(String(name).toLowerCase(), String(value));
  }

  get(name) {
    return this._map.get(String(name).toLowerCase()) ?? null;
  }

  has(name) {
    return this._map.has(String(name).toLowerCase());
  }

  delete(name) {
    this._map.delete(String(name).toLowerCase());
  }

  forEach(cb, thisArg) {
    for (const [k, v] of this._map) cb.call(thisArg, v, k, this);
  }

  entries() {
    const items = [...this._map.entries()];
    return items[Symbol.iterator]();
  }

  keys() {
    return [...this._map.keys()][Symbol.iterator]();
  }

  values() {
    return [...this._map.values()][Symbol.iterator]();
  }

  [Symbol.iterator]() {
    return this.entries();
  }

  toJSON() {
    const obj = {};
    for (const [k, v] of this._map) obj[k] = v;
    return obj;
  }

  // Internal: array of [name, value] pairs for the Rust op.
  _toPairs() {
    return [...this._map.entries()].map(([k, v]) => [k, v]);
  }
}

// -- Body mixin (shared by Request & Response) --------------

function bodyToBytes(body) {
  if (body == null) return null;
  if (typeof body === "string") return op_web_text_encode(body);
  if (body instanceof Uint8Array) return Array.from(body);
  if (body instanceof ArrayBuffer) return Array.from(new Uint8Array(body));
  if (Array.isArray(body)) return body;
  if (typeof body === "object" && body._bytes) return body._bytes;
  // Default: stringify objects as JSON.
  return op_web_text_encode(JSON.stringify(body));
}

class Body {
  constructor(body, opts = {}) {
    this._bytes = bodyToBytes(body);
    this._headers = opts.headers instanceof Headers ? opts.headers : new Headers(opts.headers);
    this._consumed = false;
  }

  get bodyUsed() { return this._consumed; }

  _decode() {
    this._consumed = true;
    return this._bytes ? Uint8Array.from(this._bytes) : new Uint8Array(0);
  }

  async arrayBuffer() {
    return this._decode().buffer;
  }

  async blob() {
    return { type: this._headers.get("content-type") || "", bytes: this._decode() };
  }

  async bytes() {
    return this._decode();
  }

  async text() {
    const b = this._decode();
    return op_web_text_decode(Array.from(b));
  }

  async json() {
    return JSON.parse(await this.text());
  }

  async formData() {
    const text = await this.text();
    const fd = new FormData();
    const params = new URLSearchParams(text);
    for (const [k, v] of params) fd.append(k, v);
    return fd;
  }
}

// -- Request -------------------------------------------------

export class Request extends Body {
  constructor(input, init = {}) {
    let url;
    let method = init.method || "GET";
    let headers;
    let body;

    if (typeof input === "string") {
      url = input;
      headers = init.headers;
      body = init.body;
    } else if (input instanceof Request) {
      url = input.url;
      method = init.method || input.method;
      headers = init.headers || input.headers;
      body = init.body !== undefined ? init.body : input._bytes;
    } else {
      url = String(input);
      headers = init.headers;
      body = init.body;
    }

    super(body, { headers });
    this.method = method.toUpperCase();
    this.url = url;
    this._referrer = init.referrer || "";
    this.mode = init.mode || "cors";
    this.credentials = init.credentials || "same-origin";
    this.cache = init.cache || "default";
    this.redirect = init.redirect || "follow";
    this.integrity = init.integrity || "";
    this.keepalive = init.keepalive || false;
    this.destination = init.destination || "";
  }

  clone() {
    return new Request(this);
  }
}

// -- Response ------------------------------------------------

export class Response extends Body {
  constructor(body, init = {}) {
    super(body, { headers: init.headers });
    this.status = init.status ?? 200;
    this.statusText = init.statusText ?? "";
    this.ok = this.status >= 200 && this.status < 300;
    this.redirected = false;
    this.type = "default";
    this.url = init.url || "";
  }

  clone() {
    return new Response(this._bytes ? Uint8Array.from(this._bytes) : null, {
      status: this.status,
      statusText: this.statusText,
      headers: new Headers(this._headers),
      url: this.url,
    });
  }

  static error() {
    const r = new Response(null, { status: 0 });
    r.type = "error";
    return r;
  }

  static redirect(url, status = 302) {
    return new Response(null, { status, headers: { location: url } });
  }

  static json(data, init = {}) {
    const headers = new Headers(init.headers);
    if (!headers.has("content-type")) headers.set("content-type", "application/json");
    return new Response(JSON.stringify(data), { ...init, headers });
  }
}

// -- FormData ------------------------------------------------

export class FormData {
  constructor() {
    this._entries = [];
  }

  append(name, value, filename) {
    this._entries.push({ name: String(name), value, filename });
  }

  set(name, value, filename) {
    const n = String(name);
    this._entries = this._entries.filter((e) => e.name !== n);
    this.append(n, value, filename);
  }

  get(name) {
    const e = this._entries.find((x) => x.name === String(name));
    return e ? e.value : null;
  }

  getAll(name) {
    return this._entries.filter((x) => x.name === String(name)).map((e) => e.value);
  }

  has(name) {
    return this._entries.some((x) => x.name === String(name));
  }

  delete(name) {
    const n = String(name);
    this._entries = this._entries.filter((x) => x.name !== n);
  }

  forEach(cb, thisArg) {
    for (const e of this._entries) cb.call(thisArg, e.value, e.name, this);
  }

  entries() {
    return this._entries.map((e) => [e.name, e.value])[Symbol.iterator]();
  }

  keys() {
    return this._entries.map((e) => e.name)[Symbol.iterator]();
  }

  values() {
    return this._entries.map((e) => e.value)[Symbol.iterator]();
  }

  [Symbol.iterator]() {
    return this.entries();
  }

  // Serialize as multipart/form-data body. Returns { body: Uint8Array, contentType }.
  _toMultipart() {
    const boundary = "----klyronFormData" + Date.now().toString(36);
    const parts = [];
    const enc = (s) => op_web_text_encode(s);
    for (const e of this._entries) {
      let head = `--${boundary}\r\n`;
      head += `Content-Disposition: form-data; name="${e.name}"`;
      if (e.filename) head += `; filename="${e.filename}"`;
      head += `\r\n`;
      if (e.filename && e.value instanceof Uint8Array) {
        head += `Content-Type: application/octet-stream\r\n\r\n`;
        parts.push(enc(head));
        parts.push(Array.from(e.value));
        parts.push(enc("\r\n"));
      } else {
        head += `\r\n\r\n`;
        parts.push(enc(head));
        parts.push(enc(String(e.value)));
        parts.push(enc("\r\n"));
      }
    }
    parts.push(enc(`--${boundary}--\r\n`));
    return {
      body: new Uint8Array(parts.flat()),
      contentType: `multipart/form-data; boundary=${boundary}`,
    };
  }
}

// -- fetch ---------------------------------------------------

export async function fetch(input, init = {}) {
  const request = input instanceof Request ? input : new Request(input, init);

  const headersJson = JSON.stringify(request.headers._toPairs());
  const bodyBytes = request._bytes;

  const resultJson = op_web_fetch_ex(
    request.url,
    request.method,
    headersJson,
    bodyBytes,
  );
  const result = JSON.parse(resultJson);

  const bodyBytesDecoded = result.body ? op_web_base64_decode(result.body) : [];
  const headers = new Headers(result.headers);
  const response = new Response(Uint8Array.from(bodyBytesDecoded), {
    status: result.status,
    statusText: result.status_text,
    headers,
    url: request.url,
  });
  return response;
}

// -- Globals ------------------------------------------------

if (typeof globalThis !== "undefined") {
  globalThis.fetch = fetch;
  globalThis.Request = Request;
  globalThis.Response = Response;
  globalThis.Headers = Headers;
  globalThis.FormData = FormData;
}

export default {
  fetch,
  Request,
  Response,
  Headers,
  FormData,
  textEncode: (s) => op_web_text_encode(s),
  textDecode: (b) => op_web_text_decode(b),
  base64Encode: (d) => op_web_base64_encode(d),
  base64Decode: (s) => op_web_base64_decode(s),
};
