import * as querystring from "./querystring.js";

export class URL {
  constructor(url, base) {
    if (base) url = new URL(base).href.replace(/\/$/, "") + "/" + url.replace(/^\//, "");
    const match = url.match(/^([a-z][a-z0-9+.-]*:)?(\/\/)?([^\/?#]*)([^?#]*)(\?[^#]*)?(#.*)?$/i);
    if (!match) throw new TypeError(`Invalid URL: ${url}`);
    const [, protocol, , host, pathname, search, hash] = match;
    this._protocol = (protocol || "http:").toLowerCase();
    this._host = host || "";
    this._pathname = pathname || "/";
    this._search = search || "";
    this._hash = hash || "";
    const [hostname, port] = this._host.split(":");
    this._hostname = hostname || "";
    this._port = port || "";
  }
  get href() { return `${this._protocol}//${this._host}${this._pathname}${this._search}${this._hash}`; }
  set href(v) { Object.assign(this, new URL(v)); }
  get protocol() { return this._protocol; }
  set protocol(v) { this._protocol = v; }
  get host() { return this._host; }
  set host(v) { this._host = v; const [hn, p] = v.split(":"); this._hostname = hn || ""; this._port = p || ""; }
  get hostname() { return this._hostname; }
  set hostname(v) { this._hostname = v; this._host = v + (this._port ? ":" + this._port : ""); }
  get port() { return this._port; }
  set port(v) { this._port = v; this._host = this._hostname + (v ? ":" + v : ""); }
  get pathname() { return this._pathname; }
  set pathname(v) { this._pathname = v; }
  get search() { return this._search; }
  set search(v) { this._search = v.startsWith("?") ? v : "?" + v; }
  get hash() { return this._hash; }
  set hash(v) { this._hash = v.startsWith("#") ? v : "#" + v; }
  get origin() { return `${this._protocol}//${this._host}`; }
  get searchParams() { return new URLSearchParams(this._search.slice(1)); }
  toString() { return this.href; }
  toJSON() { return this.href; }
}

export class URLSearchParams {
  constructor(init) {
    this._params = [];
    if (typeof init === "string") {
      init.replace(/^\?/, "").split("&").filter(Boolean).forEach(p => {
        const [k, v] = p.split("=").map(d => decodeURIComponent(d.replace(/\+/g, " ")));
        this._params.push([k, v || ""]);
      });
    } else if (Array.isArray(init)) {
      for (const [k, v] of init) this._params.push([k, v]);
    } else if (typeof init === "object" && init !== null) {
      for (const k of Object.keys(init)) this._params.push([k, String(init[k])]);
    }
  }
  append(k, v) { this._params.push([k, v]); }
  delete(k) { this._params = this._params.filter(([key]) => key !== k); }
  get(k) { const e = this._params.find(([key]) => key === k); return e ? e[1] : null; }
  getAll(k) { return this._params.filter(([key]) => key === k).map(([, v]) => v); }
  has(k) { return this._params.some(([key]) => key === k); }
  set(k, v) { this.delete(k); this.append(k, v); }
  sort() { this._params.sort(([a], [b]) => a.localeCompare(b)); }
  entries() { return this._params.values(); }
  keys() { return this._params.map(([k]) => k).values(); }
  values() { return this._params.map(([, v]) => v).values(); }
  forEach(fn) { this._params.forEach(([k, v]) => fn(v, k, this)); }
  toString() { return this._params.map(([k, v]) => `${encodeURIComponent(k)}=${encodeURIComponent(v)}`).join("&"); }
  [Symbol.iterator]() { return this._params.values(); }
}

export function parse(urlStr, parseQueryString, slashesDenoteHost) {
  return { protocol: "", hostname: "", pathname: urlStr, search: "", query: {}, hash: "", href: urlStr };
}

export function format(urlObj) { return urlObj.href || (urlObj.protocol || "http:") + "//" + (urlObj.host || urlObj.hostname || "") + (urlObj.pathname || "/"); }

export function resolve(from, to) { return new URL(to, from).href; }

export const pathToFileURL = (p) => new URL("file://" + p);
export const fileURLToPath = (url) => new URL(url).pathname;

export default { URL, URLSearchParams, parse, format, resolve, pathToFileURL, fileURLToPath };
