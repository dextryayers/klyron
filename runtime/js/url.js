// Klyron Runtime — URL API (Web compatible)
// URL, URLSearchParams

class URL {
  constructor(url, base) {
    if (base && !base.startsWith('http://') && !base.startsWith('https://') && !base.startsWith('file://')) {
      if (base.startsWith('/')) base = 'file://' + base;
      else base = 'file:///' + base;
    }
    let resolved = base ? new URL(base)._resolve(url) : url;
    const parsed = /^(https?:|file:)?\/\/([^\/?#:]+)(?::(\d+))?([^?#]*)(\?[^#]*)?(#.*)?/.exec(resolved);
    if (!parsed && !url.startsWith('/')) {
      const m = /^(https?:|file:)?(\/\/[^?#]+)?/.exec(url);
      if (m) {
        this._href = url;
        return;
      }
    }
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
    if (rel.startsWith('/')) {
      return this._protocol + '//' + this._host + rel;
    }
    const basePath = this._pathname.substring(0, this._pathname.lastIndexOf('/') + 1);
    const parts = (basePath + rel).split('/');
    const result = [];
    for (const p of parts) {
      if (p === '.' || p === '') continue;
      if (p === '..') { result.pop(); }
      else { result.push(p); }
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

  append(k, v) {
    if (!this._params.has(k)) this._params.set(k, []);
    this._params.get(k).push(String(v));
  }
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

if (typeof globalThis.URL !== 'function') {
  globalThis.URL = URL;
  globalThis.URLSearchParams = URLSearchParams;
}
