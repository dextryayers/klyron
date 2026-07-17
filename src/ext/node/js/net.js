import EventEmitter from "./events.js";
import { op_net_connect, op_net_send_bin, op_net_recv_bin, op_net_close, op_net_listen, op_net_accept, op_net_listen_close } from "ext:core/ops";

// -- isIP / isIPv4 / isIPv6 -------------------------------

export function isIP(input) {
  if (!input || typeof input !== "string") return 0;
  if (isIPv4(input)) return 4;
  if (isIPv6(input)) return 6;
  return 0;
}

export function isIPv4(input) {
  if (!input || typeof input !== "string") return false;
  const parts = input.split(".");
  if (parts.length !== 4) return false;
  return parts.every(p => {
    const n = parseInt(p, 10);
    return n >= 0 && n <= 255 && String(n) === p;
  });
}

export function isIPv6(input) {
  if (!input || typeof input !== "string") return false;
  // Very basic check - must contain at least one colon
  return input.includes(":");
}

// -- Address helpers --------------------------------------

function addrToStr(addr) {
  if (typeof addr === "string") return addr;
  if (addr?.address && addr?.port) {
    const host = addr.family === "IPv6" ? `[${addr.address}]` : addr.address;
    return `${host}:${addr.port}`;
  }
  return addr?.address || "127.0.0.1";
}

function normalizeFamily(family) {
  if (family === 6 || family === "IPv6") return 6;
  return 4;
}

// -- Socket -----------------------------------------------

let nextRid = 1;
const connState = new Map(); // rid -> { readable, writable }

export class Socket extends EventEmitter {
  constructor(options) {
    super();
    this._options = options || {};
    this._rid = -1;
    this._readable = true;
    this._writable = true;
    this._connecting = false;
    this._destroyed = false;
    this._bytesRead = 0;
    this._bytesWritten = 0;
    this._localAddress = "";
    this._localPort = 0;
    this._remoteAddress = "";
    this._remotePort = 0;
    this._remoteFamily = "";
    this._readBuffer = [];
    this._paused = false;
    this._pollTimer = null;
    this._allowHalfOpen = options?.allowHalfOpen || false;
    this._readableHighWaterMark = 65536;
    this._writableHighWaterMark = 65536;
  }

  get connecting() { return this._connecting; }
  get destroyed() { return this._destroyed; }
  get readable() { return this._readable && !this._destroyed; }
  get writable() { return this._writable && !this._destroyed; }
  get bytesRead() { return this._bytesRead; }
  get bytesWritten() { return this._bytesWritten; }
  get localAddress() { return this._localAddress; }
  get localPort() { return this._localPort; }
  get remoteAddress() { return this._remoteAddress; }
  get remotePort() { return this._remotePort; }
  get remoteFamily() { return this._remoteFamily; }
  get bufferSize() { return this._readBuffer.length; }

  connect(port, host, connectListener) {
    if (typeof host === "function") { connectListener = host; host = "127.0.0.1"; }
    if (typeof port === "object" && port?.port) {
      host = port.host || "127.0.0.1";
      port = port.port;
    }
    const addr = `${host}:${port}`;
    this._connecting = true;
    if (connectListener) this.once("connect", connectListener);

    // Use Deno ops via imported net extension
    op_net_connect(addr).then(result => {
      this._rid = result.rid;
      this._localAddress = result.local_addr.split(":")[0] || "";
      this._localPort = parseInt(result.local_addr.split(":")[1] || "0");
      this._remoteAddress = result.peer_addr.split(":")[0] || "";
      this._remotePort = parseInt(result.peer_addr.split(":")[1] || "0");
      this._remoteFamily = isIPv4(this._remoteAddress) ? "IPv4" : "IPv6";
      this._connecting = false;
      connState.set(this._rid, { socket: this });
      this.emit("connect");
      this._startPolling();
    }).catch(err => {
      this._connecting = false;
      this.emit("error", err);
    });
    return this;
  }

  _startPolling() {
    if (this._destroyed || this._pollTimer) return;
    this._pollTimer = setInterval(() => this._poll(), 10);
  }

  _stopPolling() {
    if (this._pollTimer) {
      clearInterval(this._pollTimer);
      this._pollTimer = null;
    }
  }

  _poll() {
    if (this._destroyed) { this._stopPolling(); return; }
    if (this._paused) return;

    op_net_recv_bin(this._rid).then(data => {
      if (this._destroyed) return;
      if (data && data.length > 0) {
        this._bytesRead += data.length;
        const buf = Buffer.from(data);
        if (this._paused) {
          this._readBuffer.push(buf);
        } else {
          this.emit("data", buf);
        }
      } else {
        // Connection closed - data is empty
        this._readable = false;
        this._stopPolling();
        if (!this._allowHalfOpen && !this._writable) {
          this.destroy();
        }
        this.emit("end");
        if (!this._allowHalfOpen && this._writable) {
          this._writable = false;
          this.destroy();
        }
      }
    }).catch(err => {
      if (!this._destroyed) {
        this.destroy(err);
      }
    });
  }

  write(data, encoding, cb) {
    if (typeof encoding === "function") { cb = encoding; encoding = "utf8"; }
    if (this._destroyed || !this._writable) {
      if (cb) cb(new Error("Socket is not writable"));
      return false;
    }

    const buf = typeof data === "string" ? Buffer.from(data, encoding) : Buffer.from(data);
    this._bytesWritten += buf.length;

    op_net_send_bin(this._rid, buf).then(() => {
      if (cb) cb();
    }).catch(err => {
      if (cb) cb(err);
      this.destroy(err);
    });
    return true;
  }

  end(data, encoding, cb) {
    if (typeof data === "function") { cb = data; data = undefined; }
    if (typeof encoding === "function") { cb = encoding; encoding = "utf8"; }
    if (data) this.write(data, encoding);
    this._writable = false;
    if (cb) this.once("finish", cb);
    // Send a graceful close signal
    if (this._rid >= 0) {
      op_net_close(this._rid).catch(() => {});
    }
    this.emit("finish");
    if (!this._allowHalfOpen || !this._readable) {
      this.destroy();
    }
  }

  destroy(err) {
    if (this._destroyed) return;
    this._destroyed = true;
    this._readable = false;
    this._writable = false;
    this._stopPolling();
    if (err) this.emit("error", err);
    if (this._rid >= 0) {
      op_net_close(this._rid).catch(() => {});
      connState.delete(this._rid);
    }
    this.emit("close", !!err);
  }

  pause() { this._paused = true; }
  resume() {
    this._paused = false;
    // Flush buffered data
    if (this._readBuffer.length > 0) {
      const buf = Buffer.concat(this._readBuffer);
      this._readBuffer = [];
      this.emit("data", buf);
    }
  }
  setEncoding(enc) { this._encoding = enc; return this; }
  setKeepAlive(enable, delay) { return this; }
  setNoDelay(noDelay) { return this; }
  setTimeout(ms, cb) {
    if (cb) this.once("timeout", cb);
    if (ms > 0) {
      setTimeout(() => {
        if (!this._destroyed) this.emit("timeout");
      }, ms);
    }
    return this;
  }
  address() {
    return { address: this._localAddress, family: this._remoteFamily, port: this._localPort };
  }
}

// -- Server -----------------------------------------------

export class Server extends EventEmitter {
  constructor(options, connectionListener) {
    super();
    if (typeof options === "function") { connectionListener = options; options = {}; }
    this._options = options || {};
    this._listening = false;
    this._listenRid = -1;
    this._acceptTimer = null;
    this._maxConnections = options?.maxConnections || 0;
    this._connections = new Set();
    this._allowHalfOpen = options?.allowHalfOpen || false;
    this._paused = false;
    if (connectionListener) this.on("connection", connectionListener);
  }

  get listening() { return this._listening; }
  get maxConnections() { return this._maxConnections; }
  set maxConnections(val) { this._maxConnections = val; }

  listen(port, host, backlog, cb) {
    if (typeof port === "object" && port?.port) {
      host = port.host || "0.0.0.0";
      port = port.port;
    }
    if (typeof host === "function") { cb = host; host = "0.0.0.0"; }
    if (typeof backlog === "function") { cb = backlog; backlog = 511; }
    if (typeof cb === "function") this.once("listening", cb);

    const addr = `${host || "0.0.0.0"}:${port || 0}`;
    op_net_listen(addr).then(lrid => {
      this._listenRid = lrid;
      this._listening = true;
      this.emit("listening");
      this._startAcceptLoop();
    }).catch(err => {
      this.emit("error", err);
    });
    return this;
  }

  _startAcceptLoop() {
    if (this._acceptTimer) return;
    const self = this;
    this._acceptTimer = setInterval(() => {
      if (self._paused) return;
      op_net_accept(self._listenRid).then(result => {
        if (!self._listening) return;
        const sock = new Socket();
        sock._rid = result.rid;
        sock._localAddress = result.local_addr.split(":")[0] || "";
        sock._localPort = parseInt(result.local_addr.split(":")[1] || "0");
        sock._remoteAddress = result.peer_addr.split(":")[0] || "";
        sock._remotePort = parseInt(result.peer_addr.split(":")[1] || "0");
        sock._remoteFamily = isIPv4(sock._remoteAddress) ? "IPv4" : "IPv6";
        sock._allowHalfOpen = self._allowHalfOpen;
        connState.set(sock._rid, { socket: sock });
        sock._startPolling();
        self._connections.add(sock);
        sock.on("close", () => self._connections.delete(sock));
        self.emit("connection", sock);
      }).catch(err => {
        // Accept can fail silently (no pending connections)
      });
    }, 10);
  }

  close(cb) {
    if (typeof cb === "function") this.once("close", cb);
    if (this._acceptTimer) {
      clearInterval(this._acceptTimer);
      this._acceptTimer = null;
    }
    if (this._listenRid >= 0) {
      op_net_listen_close(this._listenRid).catch(() => {});
      this._listenRid = -1;
    }
    this._listening = false;
    // Close all connections
    for (const sock of this._connections) sock.destroy();
    this._connections.clear();
    this.emit("close");
  }

  address() {
    return { address: "0.0.0.0", family: "IPv4", port: 0 };
  }

  getConnections(cb) {
    if (cb) cb(null, this._connections.size);
  }

  ref() { return this; }
  unref() { return this; }
}

// -- Factory functions ------------------------------------

export function createServer(opts, cb) {
  return new Server(opts, cb);
}

export function connect(opts, host, connectListener) {
  if (typeof opts === "number") { opts = { port: opts, host: host }; }
  if (typeof opts === "string") { opts = { path: opts }; }
  if (typeof host === "function") { connectListener = host; host = undefined; }
  const sock = new Socket();
  if (opts?.path) {
    // Unix socket - not yet supported
    process.nextTick(() => sock.emit("error", new Error("Unix sockets not supported")));
  } else {
    sock.connect(opts.port || opts, opts.host || host, connectListener);
  }
  return sock;
}

export function createConnection(opts, host, connectListener) {
  return connect(opts, host, connectListener);
}

// -- Default Export ---------------------------------------

export default {
  Socket,
  Server,
  createServer,
  connect,
  createConnection,
  isIP,
  isIPv4,
  isIPv6,
};
