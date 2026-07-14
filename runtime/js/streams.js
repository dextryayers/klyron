// Klyron Runtime — Web Streams API polyfill
// ReadableStream, WritableStream, TransformStream (minimal)

class ReadableStream {
  constructor(underlyingSource = {}, strategy = {}) {
    this._state = 'readable';
    this._controller = new ReadableStreamDefaultController(this, strategy);
    this._reader = null;
    if (underlyingSource.start) underlyingSource.start(this._controller);
    this._pull = underlyingSource.pull || null;
    this._cancel = underlyingSource.cancel || null;
  }
  get locked() { return this._reader !== null; }

  getReader() {
    return new ReadableStreamDefaultReader(this);
  }

  cancel(reason) {
    this._state = 'closed';
    if (this._cancel) this._cancel(reason);
    return Promise.resolve();
  }

  [Symbol.asyncIterator]() { return this.getReader(); }
}

class ReadableStreamDefaultController {
  constructor(stream, strategy) {
    this._stream = stream;
    this._queue = [];
    this._pulling = false;
    this._requests = [];
  }
  enqueue(chunk) {
    if (this._stream._state !== 'readable') return;
    if (this._requests.length > 0) {
      this._requests.shift()({ value: chunk, done: false });
    } else {
      this._queue.push(chunk);
    }
  }
  close() {
    this._stream._state = 'closed';
    for (const r of this._requests) r({ value: undefined, done: true });
    this._requests = [];
  }
  error(e) {
    this._stream._state = 'errored';
    for (const r of this._requests) r(Promise.reject(e));
    this._requests = [];
  }
}

class ReadableStreamDefaultReader {
  constructor(stream) {
    this._stream = stream;
    this._stream._reader = this;
  }
  async read() {
    if (this._stream._controller._queue.length > 0) {
      return { value: this._stream._controller._queue.shift(), done: false };
    }
    if (this._stream._state === 'closed') return { value: undefined, done: true };
    return new Promise(resolve => {
      this._stream._controller._requests.push(resolve);
    });
  }
  releaseLock() { this._stream._reader = null; }
  async next() { return this.read(); }
}

class WritableStream {
  constructor(underlyingSink = {}, strategy = {}) {
    this._state = 'writable';
    this._sink = underlyingSink;
    if (underlyingSink.start) underlyingSink.start(this);
  }
  get locked() { return false; }
  getWriter() { return new WritableStreamDefaultWriter(this); }
}

class WritableStreamDefaultWriter {
  constructor(stream) { this._stream = stream; }
  async write(chunk) {
    if (this._stream._sink.write) await this._stream._sink.write(chunk);
  }
  async close() {
    if (this._stream._sink.close) await this._stream._sink.close();
    this._stream._state = 'closed';
  }
  async abort(reason) {
    if (this._stream._sink.abort) await this._stream._sink.abort(reason);
  }
}

class TransformStream {
  constructor(transformer = {}, writableStrategy, readableStrategy) {
    this._readable = new ReadableStream({
      start: (controller) => { this._readableController = controller; }
    }, readableStrategy);
    this._writable = new WritableStream({
      write: async (chunk) => {
        if (transformer.transform) {
          await transformer.transform(chunk, this._readableController);
        }
      },
      close: () => {
        if (transformer.flush) transformer.flush(this._readableController);
        this._readableController.close();
      },
    }, writableStrategy);
  }
  get readable() { return this._readable; }
  get writable() { return this._writable; }
}

if (typeof globalThis.ReadableStream !== 'function') {
  globalThis.ReadableStream = ReadableStream;
  globalThis.WritableStream = WritableStream;
  globalThis.TransformStream = TransformStream;
}
