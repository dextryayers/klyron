// Klyron Runtime — node:readline polyfill

const EventEmitter = (typeof require !== 'undefined' ? require('events') : globalThis).EventEmitter || class {};

class Interface {
  constructor(options = {}) {
    this._events = new Map();
    this.input = options.input || process.stdin;
    this.output = options.output || process.stdout;
    this.completer = options.completer || null;
    this.terminal = options.terminal || (this.output && this.output.isTTY);
    this.historySize = options.historySize || 30;
    this._prompt = options.prompt || '> ';
    this._line = '';
    this._cursor = 0;
    this._history = [];
    this._historyIndex = -1;
    this._closed = false;
    this._paused = false;
    this._tabComplete = null;
    this.line = '';
    this.cursor = 0;
    this.history = [];

    if (this.input && this.input.on) {
      this.input.on('data', (data) => {
        if (this._paused) return;
        const str = typeof data === 'string' ? data : new TextDecoder().decode(data);
        for (const ch of str) {
          if (ch === '\n' || ch === '\r') {
            const line = this._line;
            this._line = '';
            this._cursor = 0;
            this.line = line;
            this.cursor = 0;
            if (line.length > 0) {
              this._history.unshift(line);
              if (this._history.length > this.historySize) this._history.pop();
              this.history = [...this._history];
            }
            this._historyIndex = -1;
            this.emit('line', line);
          } else if (ch === '\x7f' || ch === '\b') {
            if (this._cursor > 0) {
              this._line = this._line.slice(0, this._cursor - 1) + this._line.slice(this._cursor);
              this._cursor--;
              this.cursor = this._cursor;
            }
          } else if (ch === '\t' && this.completer) {
            this._handleTab();
          } else if (ch === '\x03') {
            this.emit('SIGINT');
          } else if (ch === '\x1a') {
            this.emit('SIGTSTP');
          } else if (ch === '\x11') {
            this.emit('SIGCONT');
          } else {
            this._line = this._line.slice(0, this._cursor) + ch + this._line.slice(this._cursor);
            this._cursor++;
            this.cursor = this._cursor;
            this.line = this._line;
          }
        }
      });
      this.input.on('end', () => {
        this._close();
      });
      if (this.input.on) {
        this.input.on('close', () => {
          this._close();
        });
      }
    }
  }

  _handleTab() {
    if (!this.completer) return;
    const line = this._line;
    const cb = (err, result) => {
      if (err) return;
      const [matches, matchStart] = result;
      if (matches.length === 1) {
        const prefix = line.slice(0, matchStart);
        this._line = prefix + matches[0];
        this._cursor = this._line.length;
        this.cursor = this._cursor;
        this.line = this._line;
      } else if (matches.length > 1) {
        this.output.write('\n');
        for (const m of matches) {
          this.output.write(m + '  ');
        }
        this.output.write('\n' + this._prompt + this._line);
      }
    };
    if (this.completer.length === 2) {
      this.completer(line, cb);
    } else {
      const result = this.completer(line);
      if (result && typeof result.then === 'function') {
        result.then(([matches, start]) => cb(null, [matches, start]), cb);
      } else if (Array.isArray(result)) {
        cb(null, result);
      }
    }
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

  question(query, callback) {
    if (this.output && this.output.write) {
      this.output.write(query);
    }
    this.once('line', (line) => {
      if (callback) callback(line);
    });
  }

  write(data, key) {
    if (this.output && this.output.write) {
      this.output.write(data);
    }
    if (this.input && this.input.emit) {
      this.input.emit('data', data);
    }
    return true;
  }

  pause() {
    this._paused = true;
    this.emit('pause');
    return this;
  }

  resume() {
    this._paused = false;
    this.emit('resume');
    return this;
  }

  close() {
    this._close();
  }

  _close() {
    if (this._closed) return;
    this._closed = true;
    this.emit('close');
    if (this.input && this.input.removeListener) {
      this.input.removeListener('data', this._onData);
    }
  }

  setPrompt(prompt) {
    this._prompt = prompt;
  }

  getPrompt() {
    return this._prompt;
  }

  get closed() { return this._closed; }
  get paused() { return this._paused; }
}

function createInterface(options) {
  return new Interface(options);
}

function moveCursor(stream, dx, dy) {
  if (stream && typeof stream.moveCursor === 'function') {
    stream.moveCursor(dx, dy);
    return true;
  }
  if (stream && stream.write) {
    if (dx > 0) stream.write('\x1b[' + dx + 'C');
    else if (dx < 0) stream.write('\x1b[' + (-dx) + 'D');
    if (dy > 0) stream.write('\x1b[' + dy + 'B');
    else if (dy < 0) stream.write('\x1b[' + (-dy) + 'A');
  }
  return true;
}

function clearLine(stream, dir) {
  if (stream && typeof stream.clearLine === 'function') {
    stream.clearLine(dir);
    return true;
  }
  if (stream && stream.write) {
    if (dir === -1) stream.write('\x1b[1K');
    else if (dir === 1) stream.write('\x1b[0K');
    else stream.write('\x1b[2K');
    if (dir === 0 || dir === -1) stream.write('\x1b[0G');
  }
  return true;
}

function clearScreenDown(stream) {
  if (stream && typeof stream.clearScreenDown === 'function') {
    stream.clearScreenDown();
    return true;
  }
  if (stream && stream.write) {
    stream.write('\x1b[J');
  }
  return true;
}

function cursorTo(stream, x, y) {
  if (stream && typeof stream.cursorTo === 'function') {
    stream.cursorTo(x, y);
    return true;
  }
  if (stream && stream.write) {
    if (y !== undefined) stream.write(`\x1b[${y + 1};${x + 1}H`);
    else stream.write(`\x1b[${x + 1}G`);
  }
  return true;
}

function emitKeypressEvents(stream, interface) {
  if (stream && stream.on) {
    stream.on('data', (data) => {
      const str = typeof data === 'string' ? data : new TextDecoder().decode(data);
      for (const ch of str) {
        const key = { sequence: ch, name: ch, ctrl: false, meta: false, shift: false };
        if (ch.charCodeAt(0) < 32) {
          key.ctrl = true;
          const names = { 1: 'a', 2: 'b', 3: 'c', 4: 'd', 5: 'e', 6: 'f', 7: 'g', 8: 'h', 9: 'i', 10: 'j', 11: 'k', 12: 'l', 13: 'm', 14: 'n', 15: 'o', 16: 'p', 17: 'q', 18: 'r', 19: 's', 20: 't', 21: 'u', 22: 'v', 23: 'w', 24: 'x', 25: 'y', 26: 'z' };
          key.name = names[ch.charCodeAt(0)] || ch;
        }
        if (interface) interface.emit('keypress', str, key);
      }
    });
  }
}

const readline = {
  Interface,
  createInterface,
  moveCursor,
  clearLine,
  clearScreenDown,
  cursorTo,
  emitKeypressEvents,
};

if (typeof module !== 'undefined' && module.exports) {
  module.exports = readline;
}
