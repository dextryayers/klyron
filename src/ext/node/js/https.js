import http from "./http.js";
import { EventEmitter } from "./events.js";
import { Buffer } from "./buffer.js";
import { op_https_request } from "ext:core/ops";

export class Agent {
  constructor(options) { this.options = options || {}; }
}
export const globalAgent = new Agent();

function parseUrl(url) {
  try { return new URL(url); } catch {
    const m = url.match(/^https?:\/\/([^:/]+)(?::(\d+))?(\/.*)?$/);
    return m ? {
      hostname: m[1], port: parseInt(m[2] || "443"),
      pathname: m[3] || "/", protocol: "https:",
    } : { hostname: "localhost", port: 443, pathname: "/", protocol: "https:" };
  }
}

function httpsRequestSync(method, url, headers, body) {
  const parsed = typeof url === "string" ? parseUrl(url) : url;
  const host = parsed.hostname || "localhost";
  const port = parsed.port || 443;
  const path = parsed.pathname || "/";
  const result = JSON.parse(op_https_request(method, host, port, path,
    JSON.stringify(headers || {}), body || ""));
  return result;
}

export class ClientRequest extends EventEmitter {
  constructor(url, cb) {
    super();
    this._url = url;
    this._headers = {};
    this._body = null;
    this._method = "GET";
    if (cb) this.on("response", cb);
  }

  setHeader(name, value) { this._headers[name.toLowerCase()] = value; }
  getHeader(name) { return this._headers[name.toLowerCase()]; }
  removeHeader(name) { delete this._headers[name.toLowerCase()]; }
  write(chunk) { this._body = (this._body || "") + String(chunk); }

  end(chunk) {
    if (chunk !== undefined) this.write(chunk);
    const parsed = typeof this._url === "string" ? parseUrl(this._url) : this._url;
    const host = parsed.hostname || "localhost";
    const port = parsed.port || 443;
    const path = parsed.pathname || "/";

    const result = JSON.parse(op_https_request(this._method, host, port, path,
      JSON.stringify(this._headers), this._body || ""));

    const { IncomingMessage } = http;
    const socket = new EventEmitter();
    const res = new IncomingMessage(socket);
    res.statusCode = result.statusCode;
    res.statusMessage = result.statusMessage;
    res.headers = result.headers || {};
    const bodyBuf = Buffer.from(result.body || "", "utf8");
    res._bodyBuffer = bodyBuf;
    this.emit("response", res);
    res.emit("data", bodyBuf);
    res.emit("end");
  }
}

export function createServer(options, requestListener) {
  return http.createServer(options, requestListener);
}

export function request(options, cb) {
  return new ClientRequest(options, cb);
}

export function get(options, cb) {
  const req = request(options, cb);
  req._method = "GET";
  return req;
}

export const Server = http.Server;
export const STATUS_CODES = http.STATUS_CODES;
export const METHODS = http.METHODS;

export default {
  Agent, globalAgent, createServer, request, get,
  Server, STATUS_CODES, METHODS,
};
