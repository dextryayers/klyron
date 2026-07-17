import EventEmitter from "./events.js";
import { Buffer } from "./buffer.js";
import net from "./net.js";

// -- HTTP status codes -----------------------------------------

const STATUS_CODES = {
  100: "Continue",
  101: "Switching Protocols",
  200: "OK",
  201: "Created",
  202: "Accepted",
  204: "No Content",
  206: "Partial Content",
  301: "Moved Permanently",
  302: "Found",
  303: "See Other",
  304: "Not Modified",
  307: "Temporary Redirect",
  308: "Permanent Redirect",
  400: "Bad Request",
  401: "Unauthorized",
  403: "Forbidden",
  404: "Not Found",
  405: "Method Not Allowed",
  409: "Conflict",
  410: "Gone",
  411: "Length Required",
  413: "Payload Too Large",
  415: "Unsupported Media Type",
  418: "I'm a Teapot",
  422: "Unprocessable Entity",
  429: "Too Many Requests",
  500: "Internal Server Error",
  501: "Not Implemented",
  502: "Bad Gateway",
  503: "Service Unavailable",
  504: "Gateway Timeout",
};

// -- IncomingMessage -------------------------------------------

export class IncomingMessage extends EventEmitter {
  constructor(socket) {
    super();
    this.socket = socket;
    this.method = "GET";
    this.url = "/";
    this.version = "1.1";
    this.headers = {};
    this.rawHeaders = [];
    this.trailers = {};
    this.rawTrailers = [];
    this.complete = false;
    this.aborted = false;
    this.destroyed = false;
    this.upgrade = false;
    this._chunks = [];
    this._bodyLength = 0;
    this._readableState = { flowing: false };
  }

  get httpVersion() { return `HTTP/${this.version}`; }
  get httpVersionMajor() { return parseInt(this.version.split(".")[0] || "1"); }
  get httpVersionMinor() { return parseInt(this.version.split(".")[1] || "1"); }
  get connection() { return this.socket; }

  on(event, listener) {
    if (event === "data" && this._bodyBuffer) {
      // Body already buffered; emit it.
      listener(this._bodyBuffer);
      return this;
    }
    return super.on(event, listener);
  }

  push(data) {
    if (data) this.emit("data", data);
  }

  destroy(err) {
    this.destroyed = true;
    if (err) this.emit("error", err);
    if (this.socket) this.socket.destroy();
  }
}

// -- ServerResponse -------------------------------------------

export class ServerResponse extends EventEmitter {
  constructor(req) {
    super();
    this.req = req;
    this.socket = req ? req.socket : null;
    this.statusCode = 200;
    this.statusMessage = "";
    this.headersSent = false;
    this._headers = {};
    this._wroteData = false;
    this.finished = false;
    this.sendDate = true;
    this.chunkedEncoding = false;
    this._implicitHeader = false;
  }

  setHeader(name, value) {
    this._headers[String(name).toLowerCase()] = value;
    return this;
  }

  getHeader(name) {
    return this._headers[String(name).toLowerCase()];
  }

  getHeaderNames() {
    return Object.keys(this._headers);
  }

  hasHeader(name) {
    return Object.prototype.hasOwnProperty.call(this._headers, String(name).toLowerCase());
  }

  removeHeader(name) {
    delete this._headers[String(name).toLowerCase()];
    return this;
  }

  getHeaders() {
    return { ...this._headers };
  }

  writeHead(statusCode, statusMessageOrHeaders, headers) {
    this.statusCode = statusCode;
    if (typeof statusMessageOrHeaders === "string") {
      this.statusMessage = statusMessageOrHeaders;
      if (headers) {
        for (const k in headers) this.setHeader(k, headers[k]);
      }
    } else if (statusMessageOrHeaders && typeof statusMessageOrHeaders === "object") {
      for (const k in statusMessageOrHeaders) this.setHeader(k, statusMessageOrHeaders[k]);
    }
    return this;
  }

  write(chunk, encoding, cb) {
    if (typeof encoding === "function") { cb = encoding; encoding = "utf8"; }
    const buf = Buffer.isBuffer(chunk) ? chunk : Buffer.from(String(chunk), encoding || "utf8");
    if (!this.headersSent) this._implicitHead();
    this._writeToSocket(buf);
    this._wroteData = true;
    if (cb) cb();
    return true;
  }

  end(chunk, encoding, cb) {
    if (typeof chunk === "function") { cb = chunk; chunk = undefined; }
    if (typeof encoding === "function") { cb = encoding; encoding = "utf8"; }
    const body = chunk === undefined
      ? Buffer.alloc(0)
      : (Buffer.isBuffer(chunk) ? chunk : Buffer.from(String(chunk), encoding || "utf8"));
    if (!this.headersSent) {
      if (!this.hasHeader("Content-Length") && !this.chunkedEncoding) {
        this.setHeader("Content-Length", String(body.length));
      }
      this._implicitHead();
    }
    this._writeToSocket(body);
    this.finished = true;
    this.emit("finish");
    if (this.socket) {
      // Keep-alive: leave socket open for next request; caller decides.
      this.socket._endResponse && this.socket._endResponse();
    }
    if (cb) cb();
    return this;
  }

  _implicitHead() {
    if (this.statusMessage === "") {
      this.statusMessage = STATUS_CODES[this.statusCode] || "OK";
    }
    this.headersSent = true;
  }

  _writeToSocket(buf) {
    if (this.socket) {
      this.socket.write(buf);
    }
  }

  writeContinue() {
    this._writeToSocket(Buffer.from("HTTP/1.1 100 Continue\r\n\r\n"));
  }

  flushHeaders() {
    if (!this.headersSent) this._implicitHead();
  }

  get writableEnded() { return this.finished; }
  get writableFinished() { return this.finished; }
}

// -- HTTP request parser --------------------------------------

function parseHeaders(lines) {
  const headers = {};
  const rawHeaders = [];
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const idx = line.indexOf(":");
    if (idx === -1) continue;
    const key = line.slice(0, idx).trim().toLowerCase();
    const value = line.slice(idx + 1).trim();
    rawHeaders.push(key, value);
    if (headers[key] === undefined) {
      headers[key] = value;
    } else if (Array.isArray(headers[key])) {
      headers[key].push(value);
    } else {
      headers[key] = [headers[key], value];
    }
  }
  return { headers, rawHeaders };
}

function buildResponseHead(res) {
  const status = res.statusCode || 200;
  const message = res.statusMessage || STATUS_CODES[status] || "OK";
  let head = `HTTP/1.1 ${status} ${message}\r\n`;
  if (res.sendDate && !res.hasHeader("date")) {
    res.setHeader("Date", new Date().toUTCString());
  }
  for (const name in res._headers) {
    const v = res._headers[name];
    if (Array.isArray(v)) {
      for (const item of v) head += `${name}: ${item}\r\n`;
    } else {
      head += `${name}: ${v}\r\n`;
    }
  }
  head += "\r\n";
  return head;
}

// Serialize the full response (head + body) for non-streaming use.
function serializeResponse(res, body) {
  const head = buildResponseHead(res);
  return Buffer.concat([Buffer.from(head), body]);
}

// -- Server ---------------------------------------------------

export class Server extends net.Server {
  constructor(options, requestListener) {
    super(options);
    this._httpListener = requestListener;
    if (typeof options === "function") this._httpListener = options;
    this.on("connection", (socket) => this._onConnection(socket));
  }

  _onConnection(socket) {
    const self = this;
    let buffer = Buffer.alloc(0);

    socket.on("data", (chunk) => {
      buffer = Buffer.concat([buffer, chunk]);
      // Parse as many complete requests as we can from the buffer.
      let consumed = 0;
      while (true) {
        const headerEnd = buffer.indexOf("\r\n\r\n", consumed);
        if (headerEnd === -1) break;

        const headerPart = buffer.slice(consumed, headerEnd).toString("latin1");
        const headerLines = headerPart.split("\r\n");
        const requestLine = headerLines[0].split(" ");
        const method = requestLine[0];
        const url = requestLine[1] || "/";
        const version = (requestLine[2] || "HTTP/1.1").split("/")[1] || "1.1";

        const { headers } = parseHeaders(headerLines.slice(1));
        const contentLength = parseInt(headers["content-length"] || "0", 10) || 0;

        const bodyStart = headerEnd + 4;
        const totalNeeded = bodyStart + contentLength;
        if (buffer.length < totalNeeded) break; // wait for full body

        let bodyBuf = Buffer.alloc(0);
        if (contentLength > 0) {
          bodyBuf = buffer.slice(bodyStart, totalNeeded);
        }
        consumed = totalNeeded;

        const req = new IncomingMessage(socket);
        req.method = method;
        req.url = url;
        req.version = version;
        req.headers = headers;
        req.rawHeaders = [];
        req.complete = true;
        req._bodyBuffer = bodyBuf;
        req.push(bodyBuf);
        req.emit("end");

        const res = new ServerResponse(req);
        // Override serialize-on-end so each request writes a complete response.
        res._writeToSocket = (buf) => {
          let out;
          if (!res.headersSent) {
            // For write() streaming path, send head then body once.
            out = serializeResponse(res, buf);
            res.headersSent = true;
          } else {
            out = buf;
          }
          socket.write(out);
          socket._responseWritten = true;
        };

        socket._endResponse = () => {
          // After response finished, trim consumed bytes for keep-alive.
          buffer = buffer.slice(consumed);
          consumed = 0;
        };

        self.emit("request", req, res);
        if (self._httpListener) {
          try {
            self._httpListener(req, res);
          } catch (err) {
            if (!res.headersSent) {
              res.statusCode = 500;
              res.end("Internal Server Error");
            }
            self.emit("clientError", err, socket);
          }
        }

        // Non keep-alive or upgrade: close after response.
        const connHeader = (headers["connection"] || "").toLowerCase();
        const keepAlive = version === "1.1" && connHeader !== "close";
        if (!keepAlive) {
          res.on("finish", () => socket.end());
        }
      }

      // If no more complete requests, keep buffer for next data event.
      if (consumed === 0 && buffer.length > 0 && buffer.indexOf("\r\n\r\n") === -1) {
        // incomplete; keep buffer
      }
    });

    socket.on("error", (err) => self.emit("clientError", err, socket));
    socket.on("close", () => {});
  }

  // Node API: server.requestListener via callback
  setTimeout(msecs, cb) {
    this._timeout = msecs;
    if (cb) this.on("timeout", cb);
    return this;
  }

  listen(...args) {
    const cb = args.find((a) => typeof a === "function");
    if (cb) this.once("listening", cb);
    return super.listen(...args);
  }
}

// -- Factory functions ----------------------------------------

export function createServer(options, requestListener) {
  return new Server(options, requestListener);
}

export function request(options, cb) {
  // Minimal client-side request (used by https too). Returns a ClientRequest.
  const req = new ClientRequest(options, cb);
  return req;
}

// -- ClientRequest (basic) ------------------------------------

export class ClientRequest extends EventEmitter {
  constructor(options, cb) {
    super();
    this._options = typeof options === "string" ? { url: options } : (options || {});
    this._cb = cb;
    this._headers = {};
    this._body = null;
    this.method = this._options.method || "GET";
  }

  setHeader(name, value) { this._headers[name.toLowerCase()] = value; }
  getHeader(name) { return this._headers[name.toLowerCase()]; }
  removeHeader(name) { delete this._headers[name.toLowerCase()]; }

  write(chunk) { this._body = Buffer.isBuffer(chunk) ? chunk : Buffer.from(String(chunk)); }

  end(chunk) {
    if (chunk !== undefined) this.write(chunk);
    const opts = this._options;
    const net2 = net;
    const port = opts.port || (this.agent && false ? 443 : 80);
    const host = opts.host || (typeof opts === "object" && opts.hostname) || "127.0.0.1";
    const path = opts.path || "/";

    const socket = net2.connect(port, host, () => {
      let head = `${this.method} ${path} HTTP/1.1\r\n`;
      head += `Host: ${host}\r\n`;
      for (const k in this._headers) head += `${k}: ${this._headers[k]}\r\n`;
      if (this._body && !this._headers["content-length"]) {
        head += `Content-Length: ${this._body.length}\r\n`;
      }
      head += "Connection: close\r\n\r\n";
      socket.write(Buffer.from(head));
      if (this._body) socket.write(this._body);
    });

    let respBuf = Buffer.alloc(0);
    socket.on("data", (d) => { respBuf = Buffer.concat([respBuf, d]); });
    socket.on("end", () => {
      const text = respBuf.toString("latin1");
      const he = text.indexOf("\r\n\r\n");
      const headerLines = text.slice(0, he).split("\r\n");
      const statusParts = headerLines[0].split(" ");
      const statusCode = parseInt(statusParts[1] || "200", 10);
      const { headers } = parseHeaders(headerLines.slice(1));
      const body = respBuf.slice(he + 4);
      const res = new IncomingMessage(socket);
      res.statusCode = statusCode;
      res.headers = headers;
      res._bodyBuffer = body;
      if (this._cb) this._cb(res);
      this.emit("response", res);
      res.emit("data", body);
      res.emit("end");
    });
    socket.on("error", (e) => this.emit("error", e));
  }
}

export const METHODS = ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS", "TRACE", "CONNECT"];
export const STATUS = STATUS_CODES;

export default {
  Server,
  createServer,
  request,
  IncomingMessage,
  ServerResponse,
  ClientRequest,
  METHODS,
  STATUS_CODES,
};
