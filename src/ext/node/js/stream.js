import { EventEmitter } from "./events.js";

function hwm(opts, def) { return (opts && opts.highWaterMark) || def || 16384; }

export class Readable extends EventEmitter {
  constructor(opts = {}) {
    super();
    this._readableState = {
      highWaterMark: hwm(opts, 16384),
      flowing: null,
      ended: false,
      destroyed: false,
      buffer: [],
      reading: false,
      readableListening: false,
    };
    this._read = opts.read || (() => this.push(null));
  }

  push(chunk) {
    if (chunk === null) {
      this._readableState.ended = true;
      if (this._readableState.buffer.length === 0) this.emit("end");
      return false;
    }
    if (this._readableState.flowing) {
      this.emit("data", chunk);
      return true;
    }
    this._readableState.buffer.push(chunk);
    this.emit("readable");
    return this._readableState.buffer.length < this._readableState.highWaterMark;
  }

  read(size) {
    if (this._readableState.ended && this._readableState.buffer.length === 0) {
      this.emit("end");
      return null;
    }
    if (this._readableState.buffer.length > 0) {
      if (size && size > 0) {
        const chunks = [];
        let len = 0;
        while (this._readableState.buffer.length > 0 && len < size) {
          const c = this._readableState.buffer.shift();
          chunks.push(c);
          len += c.length || c.toString().length || 1;
        }
        const data = chunks.length === 1 ? chunks[0] : Buffer.concat(chunks);
        this.emit("data", data);
        return data;
      }
      const data = this._readableState.buffer.shift();
      this.emit("data", data);
      return data;
    }
    this._readableState.reading = true;
    this._read(size);
    return null;
  }

  pipe(dest, opts = {}) {
    if (typeof dest === "function") return this;
    let ended = false;

    const ondata = (chunk) => {
      if (dest._writableState && dest._writableState.ended) return;
      const ok = dest.write(chunk);
      if (!ok && this.pause) this.pause();
    };

    const ondrain = () => {
      if (this.resume) this.resume();
    };

    const onend = () => {
      ended = true;
      dest.end();
    };

    const onerror = (err) => {
      dest.destroy(err);
    };

    const onclose = () => {
      if (!ended) this.emit("end");
    };

    this.on("data", ondata);
    dest.on("drain", ondrain);
    dest.on("error", onerror);
    this.on("end", onend);
    this.on("close", onclose);

    if (opts.end !== false) {
      this.on("end", () => { if (!dest._writableState?.ended) dest.end(); });
    }

    this.resume();
    return dest;
  }

  resume() {
    this._readableState.flowing = true;
    while (this._readableState.buffer.length > 0) {
      const chunk = this._readableState.buffer.shift();
      this.emit("data", chunk);
    }
    if (this._readableState.ended) {
      this.emit("end");
      return;
    }
    this._read();
    return this;
  }

  pause() {
    this._readableState.flowing = false;
    return this;
  }

  destroy(err) {
    if (this._readableState.destroyed) return;
    this._readableState.destroyed = true;
    if (err) this.emit("error", err);
    this.emit("close");
  }

  setEncoding() { return this; }

  [Symbol.asyncIterator]() {
    let done = false;
    const buf = [];
    let resolve = null;

    this.on("data", (chunk) => {
      if (resolve) { resolve(chunk); resolve = null; }
      else buf.push(chunk);
    });
    this.on("end", () => { done = true; if (resolve) { resolve(null); resolve = null; } });

    return {
      next: () => new Promise((r) => {
        if (buf.length > 0) r({ value: buf.shift(), done: false });
        else if (done) r({ value: undefined, done: true });
        else resolve = (value) => r(value === null ? { value: undefined, done: true } : { value, done: false });
      }),
      [Symbol.asyncIterator]() { return this; },
    };
  }
}

export class Writable extends EventEmitter {
  constructor(opts = {}) {
    super();
    this._writableState = {
      highWaterMark: hwm(opts, 16384),
      ended: false,
      destroyed: false,
      buffer: [],
      writing: false,
      pendingcb: null,
    };
    this._write = opts.write || ((chunk, enc, cb) => cb());
    this._final = opts.final || (cb => cb());
    this._writev = opts.writev || null;
  }

  write(chunk, encoding, callback) {
    if (typeof encoding === "function") { callback = encoding; encoding = null; }
    if (!encoding) encoding = "utf8";

    const cb = callback || (() => {});
    const state = this._writableState;

    if (state.ended) {
      cb(new Error("write after end"));
      return false;
    }

    if (state.writing) {
      state.buffer.push({ chunk, encoding, cb });
      return state.buffer.length < state.highWaterMark;
    }

    state.writing = true;
    state.pendingcb = cb;

    this._write(chunk, encoding, (err) => {
      state.writing = false;
      const cb2 = state.pendingcb;
      state.pendingcb = null;
      cb2?.(err);

      if (state.buffer.length > 0) {
        const next = state.buffer.shift();
        this.write(next.chunk, next.encoding, next.cb);
      } else if (this._writableState.buffer.length < this._writableState.highWaterMark) {
        this.emit("drain");
      }

      if (state.ended && state.buffer.length === 0) {
        this._final((err2) => {
          if (!err2) this.emit("finish");
        });
      }
    });

    return state.buffer.length < state.highWaterMark;
  }

  end(chunk, encoding, callback) {
    if (typeof chunk === "function") { callback = chunk; chunk = null; }
    if (typeof encoding === "function") { callback = encoding; encoding = null; }
    if (chunk) this.write(chunk, encoding);
    this._writableState.ended = true;
    if (this._writableState.buffer.length === 0) {
      this._final(callback || (() => {}));
      this.emit("finish");
    } else {
      const origCb = this._writableState.pendingcb;
      this._writableState.pendingcb = (err) => {
        origCb?.(err);
        this._final(callback || (() => {}));
        this.emit("finish");
      };
    }
  }

  destroy(err) {
    if (this._writableState.destroyed) return;
    this._writableState.destroyed = true;
    if (err) this.emit("error", err);
    this.emit("close");
  }

  get writableLength() { return this._writableState.buffer.length; }
  get writableHighWaterMark() { return this._writableState.highWaterMark; }
  get writableFinished() { return this._writableState.ended && this._writableState.buffer.length === 0; }
}

export class Transform extends Readable {
  constructor(opts = {}) {
    super(opts);
    this._transform = opts.transform || ((chunk, enc, cb) => { cb(null, chunk); });
    this._flush = opts.flush || (cb => cb());
    this._writableState = {
      highWaterMark: hwm(opts, 16384),
      ended: false,
      destroyed: false,
      buffer: [],
      writing: false,
      pendingcb: null,
    };
  }

  write(chunk, encoding, callback) {
    if (typeof encoding === "function") { callback = encoding; encoding = null; }
    if (!encoding) encoding = "utf8";
    const cb = callback || (() => {});
    const state = this._writableState;

    if (state.writing) {
      state.buffer.push({ chunk, encoding, cb });
      return state.buffer.length < state.highWaterMark;
    }

    state.writing = true;
    state.pendingcb = cb;

    this._transform(chunk, encoding, (err, data) => {
      state.writing = false;
      const cb2 = state.pendingcb;
      state.pendingcb = null;
      if (err) { this.destroy(err); cb2?.(err); return; }
      if (data !== null && data !== undefined) this.push(data);
      cb2?.(null);

      if (state.buffer.length > 0) {
        const next = state.buffer.shift();
        this.write(next.chunk, next.encoding, next.cb);
      } else if (state.buffer.length < state.highWaterMark) {
        this.emit("drain");
      }

      if (state.ended && state.buffer.length === 0) {
        this._flush((err2) => {
          if (err2) { this.destroy(err2); return; }
          this.push(null);
        });
      }
    });

    return state.buffer.length < state.highWaterMark;
  }

  end(chunk, encoding, callback) {
    if (typeof chunk === "function") { callback = chunk; chunk = null; }
    if (typeof encoding === "function") { callback = encoding; encoding = null; }
    if (chunk) this.write(chunk, encoding);
    this._writableState.ended = true;
  }
}

export class PassThrough extends Transform {
  constructor(opts) {
    super({ ...opts, transform: (chunk, enc, cb) => cb(null, chunk) });
  }
}

export function pipeline(...streams) {
  const cb = typeof streams[streams.length - 1] === "function" ? streams.pop() : (() => {});
  let error;

  for (let i = 0; i < streams.length - 1; i++) {
    streams[i].pipe(streams[i + 1]);
    streams[i].on("error", (err) => { error = err; cleanup(); });
  }

  function cleanup() {
    for (const s of streams) {
      s.removeAllListeners?.("data");
      s.removeAllListeners?.("drain");
      s.removeAllListeners?.("error");
      s.removeAllListeners?.("end");
    }
    cb(error);
  }

  const last = streams[streams.length - 1];
  last.on("finish", cleanup);
  last.on("error", cleanup);
  last.on("close", cleanup);
}

export function finished(stream, cb) {
  let done = false;
  function onFinish() { if (!done) { done = true; cb?.(); } }
  function onError(err) { if (!done) { done = true; cb?.(err); } }
  stream.on("end", onFinish);
  stream.on("finish", onFinish);
  stream.on("error", onError);
  stream.on("close", () => { if (!done) onFinish(); });
  return () => { done = true; };
}

export default { Readable, Writable, Transform, PassThrough, pipeline, finished };
