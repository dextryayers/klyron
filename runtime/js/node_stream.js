// Klyron Runtime — node:stream polyfill
// Readable, Writable, Transform, Duplex, PassThrough, pipeline, finished

const EE = (typeof require === 'function' ? (function() { try { return require('./node_events'); } catch {} })() : null) || { EventEmitter: globalThis.EventEmitter || class {} };
const EventEmitter = EE.EventEmitter;

class Readable extends EventEmitter {
  constructor(options = {}) {
    super();
    this._readableState = {
      objectMode: !!options.objectMode,
      highWaterMark: options.highWaterMark || 16384,
      buffer: [],
      flowing: null,
      ended: false,
      endEmitted: false,
      reading: false,
      destroyed: false,
      errorEmitted: false,
      encoding: options.encoding || null,
    };
    this._read = options.read || null;
    if (this._read) this._read = this._read.bind(this);
  }

  get readable() { return !this._readableState.destroyed; }
  get readableLength() { return this._readableState.buffer.reduce((s, c) => s + c.length, 0); }
  get readableEnded() { return this._readableState.ended; }
  get readableFlowing() { return this._readableState.flowing; }
  get destroyed() { return this._readableState.destroyed; }

  _read(size) {
    if (this._read) this._read(size);
  }

  read(size) {
    if (size === undefined) size = this._readableState.highWaterMark;
    if (this._readableState.buffer.length > 0) {
      const chunk = this._readableState.buffer.shift();
      if (this._readableState.encoding === 'utf8' || this._readableState.encoding === 'utf-8') {
        return typeof chunk === 'string' ? chunk : new TextDecoder().decode(chunk);
      }
      return chunk;
    }
    if (this._readableState.ended) return null;
    this._read(size);
    return null;
  }

  push(chunk, encoding) {
    if (chunk === null) {
      this._readableState.ended = true;
      if (!this._readableState.endEmitted) {
        this._readableState.endEmitted = true;
        this.emit('end');
      }
      return false;
    }
    if (encoding === 'utf8' || encoding === 'utf-8') {
      chunk = new TextEncoder().encode(chunk);
    }
    if (this._readableState.flowing) {
      this.emit('data', chunk);
    } else {
      this._readableState.buffer.push(chunk);
    }
    return true;
  }

  pipe(dest, options = {}) {
    this.on('data', chunk => {
      const canContinue = dest.write(chunk);
      if (!canContinue && !options.end !== false) this.pause();
    });
    this.on('end', () => {
      if (options.end !== false) dest.end();
    });
    this.on('error', err => dest.destroy(err));
    dest.on('drain', () => this.resume());
    return dest;
  }

  unpipe(dest) {
    this.removeAllListeners('data');
    this.removeAllListeners('end');
    this.removeAllListeners('error');
    return this;
  }

  pause() {
    this._readableState.flowing = false;
    return this;
  }

  resume() {
    this._readableState.flowing = true;
    if (this._readableState.buffer.length > 0) {
      for (const chunk of this._readableState.buffer.splice(0)) {
        this.emit('data', chunk);
      }
    }
    return this;
  }

  isPaused() {
    return this._readableState.flowing !== true;
  }

  setEncoding(encoding) {
    this._readableState.encoding = encoding;
    return this;
  }

  unshift(chunk) {
    this._readableState.buffer.unshift(chunk);
  }

  wrap(stream) {
    stream.on('data', chunk => this.push(chunk));
    stream.on('end', () => this.push(null));
    stream.on('error', err => this.emit('error', err));
    return this;
  }

  destroy(error) {
    if (this._readableState.destroyed) return;
    this._readableState.destroyed = true;
    if (error) {
      this._readableState.errorEmitted = true;
      this.emit('error', error);
    }
    this.emit('close');
  }

  [Symbol.asyncIterator]() {
    const self = this;
    let done = false;
    this.on('end', () => done = true);
    return {
      next() {
        if (done) return Promise.resolve({ value: undefined, done: true });
        return new Promise(resolve => {
          self.once('data', value => resolve({ value, done: false }));
          if (done) resolve({ value: undefined, done: true });
        });
      },
      [Symbol.asyncIterator]() { return this; },
    };
  }
}

class Writable extends EventEmitter {
  constructor(options = {}) {
    super();
    this._writableState = {
      objectMode: !!options.objectMode,
      highWaterMark: options.highWaterMark || 16384,
      buffer: [],
      writing: false,
      ended: false,
      ending: false,
      destroyed: false,
      errorEmitted: false,
      decodeStrings: options.decodeStrings !== false,
    };
    this._write = options.write || null;
    if (this._write) this._write = this._write.bind(this);
    this._final = options.final || null;
    if (this._final) this._final = this._final.bind(this);
  }

  get writable() { return !this._writableState.destroyed; }
  get writableLength() { return this._writableState.buffer.length; }
  get writableEnded() { return this._writableState.ended; }
  get destroyed() { return this._writableState.destroyed; }

  _write(chunk, encoding, callback) {
    if (this._write) {
      this._write(chunk, encoding, callback);
    } else {
      callback(null);
    }
  }

  _writev(chunks, callback) {
    callback(null);
  }

  _final(callback) {
    if (this._final) {
      this._final(callback);
    } else {
      callback(null);
    }
  }

  write(chunk, encoding, callback) {
    if (typeof encoding === 'function') { callback = encoding; encoding = null; }
    if (!callback) callback = () => {};
    if (this._writableState.ended || this._writableState.ending) {
      callback(new Error('write after end'));
      return false;
    }
    if (this._writableState.decodeStrings && typeof chunk === 'string') {
      chunk = new TextEncoder().encode(chunk);
    }
    if (this._writableState.writing) {
      this._writableState.buffer.push({ chunk, encoding, callback });
      return this._writableState.buffer.length < this._writableState.highWaterMark;
    }
    this._writableState.writing = true;
    try {
      this._write(chunk, encoding, err => {
        this._writableState.writing = false;
        if (err) {
          callback(err);
          this.destroy(err);
          return;
        }
        callback(null);
        this.emit('drain');
        if (this._writableState.buffer.length > 0) {
          const next = this._writableState.buffer.shift();
          this.write(next.chunk, next.encoding, next.callback);
        }
        if (this._writableState.ending) {
          this._final(err => {
            this._writableState.ended = true;
            this.emit('finish');
          });
        }
      });
    } catch (e) {
      this._writableState.writing = false;
      callback(e);
      this.destroy(e);
    }
    return this._writableState.buffer.length < this._writableState.highWaterMark;
  }

  end(chunk, encoding, callback) {
    if (typeof chunk === 'function') { callback = chunk; chunk = null; encoding = null; }
    else if (typeof encoding === 'function') { callback = encoding; encoding = null; }
    if (chunk) this.write(chunk, encoding);
    this._writableState.ending = true;
    if (!this._writableState.writing) {
      this._final(err => {
        this._writableState.ended = true;
        if (callback) callback(err);
        this.emit('finish');
      });
    } else if (callback) {
      this.once('finish', () => callback(null));
    }
    return this;
  }

  cork() { this._writableState.corked = true; }
  uncork() {
    this._writableState.corked = false;
    while (this._writableState.buffer.length > 0) {
      const next = this._writableState.buffer.shift();
      this.write(next.chunk, next.encoding, next.callback);
    }
  }

  setDefaultEncoding(encoding) { return this; }

  destroy(error) {
    if (this._writableState.destroyed) return;
    this._writableState.destroyed = true;
    if (error) {
      this._writableState.errorEmitted = true;
      this.emit('error', error);
    }
    this.emit('close');
  }
}

class Duplex extends Readable {
  constructor(options = {}) {
    super(options);
    this._writableState = {
      objectMode: !!options.objectMode,
      highWaterMark: options.highWaterMark || 16384,
      buffer: [],
      writing: false,
      ended: false,
      ending: false,
      destroyed: false,
      errorEmitted: false,
      decodeStrings: options.decodeStrings !== false,
    };
    this._write = options.write || null;
    if (this._write) this._write = this._write.bind(this);
    this._final = options.final || null;
    if (this._final) this._final = this._final.bind(this);
  }

  get writable() { return !this._writableState.destroyed; }
  get writableLength() { return this._writableState.buffer.length; }
  get writableEnded() { return this._writableState.ended; }

  _write(chunk, encoding, callback) {
    if (this._write) this._write(chunk, encoding, callback);
    else callback(null);
  }

  _final(callback) {
    if (this._final) this._final(callback);
    else callback(null);
  }

  write(chunk, encoding, callback) {
    if (typeof encoding === 'function') { callback = encoding; encoding = null; }
    if (!callback) callback = () => {};
    if (this._writableState.ended || this._writableState.ending) {
      callback(new Error('write after end'));
      return false;
    }
    if (this._writableState.decodeStrings && typeof chunk === 'string') {
      chunk = new TextEncoder().encode(chunk);
    }
    if (this._writableState.writing) {
      this._writableState.buffer.push({ chunk, encoding, callback });
      return this._writableState.buffer.length < this._writableState.highWaterMark;
    }
    this._writableState.writing = true;
    try {
      this._write(chunk, encoding, err => {
        this._writableState.writing = false;
        if (err) { callback(err); this.destroy(err); return; }
        callback(null);
        this.emit('drain');
        if (this._writableState.buffer.length > 0) {
          const next = this._writableState.buffer.shift();
          this.write(next.chunk, next.encoding, next.callback);
        }
        if (this._writableState.ending) {
          this._final(err => {
            this._writableState.ended = true;
            this.emit('finish');
          });
        }
      });
    } catch (e) {
      this._writableState.writing = false;
      callback(e);
      this.destroy(e);
    }
    return this._writableState.buffer.length < this._writableState.highWaterMark;
  }

  end(chunk, encoding, callback) {
    if (typeof chunk === 'function') { callback = chunk; chunk = null; encoding = null; }
    else if (typeof encoding === 'function') { callback = encoding; encoding = null; }
    if (chunk) this.write(chunk, encoding);
    this._writableState.ending = true;
    if (!this._writableState.writing) {
      this._final(err => {
        this._writableState.ended = true;
        if (callback) callback(err);
        this.emit('finish');
      });
    } else if (callback) {
      this.once('finish', () => callback(null));
    }
    return this;
  }

  destroy(error) {
    if (this._writableState.destroyed) return;
    this._writableState.destroyed = true;
    if (error) {
      this._writableState.errorEmitted = true;
      this.emit('error', error);
    }
    this.emit('close');
  }
}

class Transform extends Duplex {
  constructor(options = {}) {
    super(options);
    this._transform = options.transform || null;
    if (this._transform) this._transform = this._transform.bind(this);
    this._flush = options.flush || null;
    if (this._flush) this._flush = this._flush.bind(this);
  }

  _transform(chunk, encoding, callback) {
    if (this._transform) {
      this._transform(chunk, encoding, callback);
    } else {
      this.push(chunk);
      callback(null);
    }
  }

  _flush(callback) {
    if (this._flush) this._flush(callback);
    else callback(null);
  }

  _write(chunk, encoding, callback) {
    this._transform(chunk, encoding, err => {
      if (err) { callback(err); return; }
      callback(null);
    });
  }

  _final(callback) {
    this._flush(callback);
  }

  push(chunk, encoding) {
    return super.push(chunk, encoding);
  }
}

class PassThrough extends Transform {
  _transform(chunk, encoding, callback) {
    this.push(chunk, encoding);
    callback(null);
  }
}

function pipeline(...streams) {
  const callback = typeof streams[streams.length - 1] === 'function' ? streams.pop() : null;
  if (!callback) return pipeline.promise(...streams);
  try {
    for (let i = 0; i < streams.length - 1; i++) {
      streams[i].pipe(streams[i + 1]);
    }
    let error = null;
    for (const s of streams) {
      s.on('error', err => { error = err; });
      s.on('close', () => {
        if (error) callback(error);
      });
    }
    streams[streams.length - 1].on('finish', () => {
      callback(error);
    });
  } catch (e) {
    callback(e);
  }
}

pipeline.promise = (...streams) => {
  return new Promise((resolve, reject) => {
    pipeline(...streams, err => err ? reject(err) : resolve());
  });
};

function finished(stream, options, callback) {
  if (typeof options === 'function') { callback = options; options = {}; }
  if (!callback) return finished.promise(stream, options);
  const onFinish = () => { cleanup(); callback(null); };
  const onError = (err) => { cleanup(); callback(err); };
  const onClose = () => {
    if (stream.readable && !stream.readableEnded) return;
    if (stream.writable && !stream.writableEnded) return;
    cleanup();
    callback(null);
  };
  const cleanup = () => {
    stream.removeListener('finish', onFinish);
    stream.removeListener('end', onFinish);
    stream.removeListener('error', onError);
    stream.removeListener('close', onClose);
  };
  stream.on('finish', onFinish);
  stream.on('end', onFinish);
  stream.on('error', onError);
  stream.on('close', onClose);
  return cleanup;
}

finished.promise = (stream, options) => {
  return new Promise((resolve, reject) => {
    finished(stream, options, err => err ? reject(err) : resolve());
  });
};

const promises = {
  pipeline: pipeline.promise,
  finished: finished.promise,
};

const stream = {
  Readable,
  Writable,
  Duplex,
  Transform,
  PassThrough,
  pipeline,
  finished,
  promises,
  Stream: Readable,
  isReadable(stream) { return stream instanceof Readable; },
  isWritable(stream) { return stream instanceof Writable; },
  isDuplex(stream) { return stream instanceof Duplex; },
  isTransform(stream) { return stream instanceof Transform; },
  addAbortSignal(signal, stream) {
    signal.addEventListener('abort', () => stream.destroy(signal.reason));
    return stream;
  },
  getDefaultHighWaterMark() { return 16384; },
  setDefaultHighWaterMark() {},
  consume: async (stream, n) => {
    const chunks = [];
    for await (const chunk of stream) {
      chunks.push(chunk);
      if (n && chunks.length >= n) break;
    }
    return chunks.length === 1 ? chunks[0] : Buffer.concat(chunks);
  },
};

if (typeof module !== 'undefined' && module.exports) {
  module.exports = stream;
}
