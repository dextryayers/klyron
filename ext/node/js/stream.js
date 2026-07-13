import { EventEmitter } from "./events.js";

export class Readable extends EventEmitter {
  constructor(opts = {}) {
    super();
    this._readableState = { highWaterMark: opts.highWaterMark || 16384, flowing: null, ended: false };
    this._read = opts.read || (() => this.push(null));
  }
  push(chunk) {
    if (chunk === null) { this._readableState.ended = true; this.emit("end"); return; }
    this.emit("data", chunk);
  }
  pipe(dest) {
    this.on("data", d => dest.write(d));
    this.on("end", () => dest.end());
    return dest;
  }
  read(size) { this._read(size); }
  resume() { this._readableState.flowing = true; this.read(); }
  pause() { this._readableState.flowing = false; }
  destroy(err) { if (err) this.emit("error", err); this.emit("close"); }
  setEncoding() { return this; }
}

export class Writable extends EventEmitter {
  constructor(opts = {}) {
    super();
    this._writableState = { ended: false };
    this._write = opts.write || ((chunk, enc, cb) => cb());
    this._final = opts.final || (cb => cb());
  }
  write(chunk, encoding, callback) {
    if (typeof encoding === "function") { callback = encoding; encoding = null; }
    this._write(chunk, encoding || "utf8", callback || (() => {}));
    return true;
  }
  end(chunk, encoding, callback) {
    if (typeof chunk === "function") { callback = chunk; chunk = null; }
    if (typeof encoding === "function") { callback = encoding; encoding = null; }
    if (chunk) this.write(chunk, encoding);
    this._writableState.ended = true;
    this._final(callback || (() => {}));
    this.emit("finish");
  }
  destroy(err) { if (err) this.emit("error", err); this.emit("close"); }
}

export class Transform extends Readable {
  constructor(opts = {}) {
    super(opts);
    this._transform = opts.transform || ((chunk, enc, cb) => { cb(null, chunk); });
    this._flush = opts.flush || (cb => cb());
    this._writableState = { ended: false };
  }
  write(chunk, encoding, callback) {
    if (typeof encoding === "function") { callback = encoding; encoding = null; }
    this._transform(chunk, encoding || "utf8", (err, data) => {
      if (err) { this.destroy(err); return callback?.(err); }
      if (data) this.push(data);
      callback?.();
    });
    return true;
  }
  end(chunk, encoding, callback) {
    if (typeof chunk === "function") { callback = chunk; chunk = null; }
    if (chunk) this.write(chunk, encoding);
    this._writableState.ended = true;
    this._flush(err => { if (err) this.destroy(err); this.push(null); callback?.(); });
  }
}

export class PassThrough extends Transform {
  constructor(opts) {
    super({ ...opts, transform: (chunk, enc, cb) => cb(null, chunk) });
  }
}

export function pipeline(...streams) {
  const cb = typeof streams[streams.length - 1] === "function" ? streams.pop() : (() => {});
  for (let i = 0; i < streams.length - 1; i++)
    streams[i].pipe(streams[i + 1]);
  streams[streams.length - 1].on("finish", cb);
  streams[streams.length - 1].on("error", cb);
}

export function finished(stream, cb) {
  stream.on("end", () => cb?.());
  stream.on("finish", () => cb?.());
  stream.on("error", cb);
}

export default { Readable, Writable, Transform, PassThrough, pipeline, finished };
