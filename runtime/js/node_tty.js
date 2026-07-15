// Klyron Runtime — node:tty polyfill

function isatty(fd) {
  if (fd === 0) {
    return typeof process !== 'undefined' && process.stdin && process.stdin.isTTY || false;
  }
  if (fd === 1) {
    return typeof process !== 'undefined' && process.stdout && process.stdout.isTTY || false;
  }
  if (fd === 2) {
    return typeof process !== 'undefined' && process.stderr && process.stderr.isTTY || false;
  }
  return false;
}

class ReadStream {
  constructor(fd) {
    this.fd = fd;
    this._isRaw = false;
    this.isTTY = true;
    this._events = new Map();
  }

  on(event, handler) {
    if (!this._events.has(event)) this._events.set(event, []);
    this._events.get(event).push(handler);
    return this;
  }

  once(event, handler) {
    const wrapper = (...args) => { handler(...args); this.removeListener(event, wrapper); };
    return this.on(event, wrapper);
  }

  removeListener(event, handler) {
    const handlers = this._events.get(event);
    if (handlers) {
      this._events.set(event, handlers.filter(h => h !== handler));
    }
    return this;
  }

  emit(event, ...args) {
    const handlers = this._events.get(event) || [];
    for (const h of [...handlers]) {
      try { h.call(this, ...args); } catch (e) { queueMicrotask(() => { throw e; }); }
    }
    return true;
  }

  setRawMode(mode) {
    this._isRaw = !!mode;
    return this;
  }

  get isRaw() { return this._isRaw; }
  set isRaw(v) { this._isRaw = !!v; }

  pause() { return this; }
  resume() { return this; }
  destroy() { return this; }
  setEncoding(encoding) { return this; }
  read(size) { return null; }
  pipe(dest) { return dest; }
  unpipe(dest) { return this; }
  push(chunk) { return true; }

  get readable() { return true; }
  get readableFlowing() { return null; }
}

class WriteStream {
  constructor(fd) {
    this.fd = fd;
    this.isTTY = true;
    this.columns = 80;
    this.rows = 24;
    this._events = new Map();
  }

  on(event, handler) {
    if (!this._events.has(event)) this._events.set(event, []);
    this._events.get(event).push(handler);
    return this;
  }

  once(event, handler) {
    const wrapper = (...args) => { handler(...args); this.removeListener(event, wrapper); };
    return this.on(event, wrapper);
  }

  removeListener(event, handler) {
    const handlers = this._events.get(event);
    if (handlers) {
      this._events.set(event, handlers.filter(h => h !== handler));
    }
    return this;
  }

  emit(event, ...args) {
    const handlers = this._events.get(event) || [];
    for (const h of [...handlers]) {
      try { h.call(this, ...args); } catch (e) { queueMicrotask(() => { throw e; }); }
    }
    return true;
  }

  write(data) {
    if (typeof process !== 'undefined' && process.stdout) {
      process.stdout.write(data);
    }
    return true;
  }

  end(data) {
    if (data) this.write(data);
    return this;
  }

  clearLine(dir) {
    if (dir === -1) this.write('\x1b[1K');
    else if (dir === 1) this.write('\x1b[0K');
    else this.write('\x1b[2K');
    if (dir === 0 || dir === -1) this.write('\x1b[0G');
    return true;
  }

  cursorTo(x, y) {
    if (y !== undefined) this.write(`\x1b[${y + 1};${x + 1}H`);
    else this.write(`\x1b[${x + 1}G`);
    return true;
  }

  moveCursor(dx, dy) {
    if (dx > 0) this.write(`\x1b[${dx}C`);
    else if (dx < 0) this.write(`\x1b[${-dx}D`);
    if (dy > 0) this.write(`\x1b[${dy}B`);
    else if (dy < 0) this.write(`\x1b[${-dy}A`);
    return true;
  }

  clearScreenDown() {
    this.write('\x1b[J');
    return true;
  }

  getColorDepth(env) {
    const term = (env && env.TERM) || process.env.TERM || '';
    if (term.includes('truecolor') || term.includes('24bit')) return 24;
    if (term.includes('256')) return 8;
    if (term.includes('color')) return 4;
    return 1;
  }

  hasColors(count, env) {
    if (count === undefined) count = 8;
    return this.getColorDepth(env) >= count;
  }

  get windowSize() { return [this.columns, this.rows]; }

  get writable() { return true; }
  get writableLength() { return 0; }
  destroy() { return this; }
  ref() { return this; }
  unref() { return this; }
}

const tty = {
  isatty,
  ReadStream,
  WriteStream,
};

if (typeof module !== 'undefined' && module.exports) {
  module.exports = tty;
}
