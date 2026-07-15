// Klyron Runtime — node:repl polyfill

const EventEmitter = (typeof require !== 'undefined' ? require('events') : globalThis).EventEmitter || class {};

const REPL_MODE_SLOPPY = Symbol('repl-sloppy');
const REPL_MODE_STRICT = Symbol('repl-strict');
const REPL_MODE_MAGIC = Symbol('repl-magic');

const _recoverableErrors = new Set([
  'SyntaxError: Unexpected end of input',
  'SyntaxError: missing ) after argument list',
  'SyntaxError: Unterminated template literal',
  'SyntaxError: Unexpected token',
  'SyntaxError: expected expression, got end of script',
]);

class Recoverable extends SyntaxError {
  constructor(err) {
    super(err.message);
    this.name = 'Recoverable';
    this.stack = err.stack;
    this.err = err;
  }
}

class REPLServer {
  constructor(options = {}) {
    this._events = new Map();
    this._prompt = options.prompt || '> ';
    this._eval = options.eval || this._defaultEval.bind(this);
    this._writer = options.writer || this._defaultWriter.bind(this);
    this._input = options.input || process.stdin;
    this._output = options.output || process.stdout;
    this._terminal = options.terminal !== undefined ? options.terminal : (this._output && this._output.isTTY);
    this._useColors = options.useColors !== undefined ? options.useColors : this._terminal;
    this._useGlobal = options.useGlobal || false;
    this._ignoreUndefined = options.ignoreUndefined || false;
    this._replMode = options.replMode || REPL_MODE_SLOPPY;
    this._commands = new Map();
    this._bufferedCommand = '';
    this._context = options.context || Object.create(null);
    this._history = [];
    this._historyIndex = -1;
    this._closed = false;
    this._lines = [];
    this._brackets = { '{': '}', '[': ']', '(': ')' };
    this._bracketStack = [];

    this._defineDefaultCommands();

    if (this._input && this._input.on) {
      this._input.on('data', (data) => {
        if (this._closed) return;
        const str = typeof data === 'string' ? data : new TextDecoder().decode(data);
        this._handleInput(str);
      });
      this._input.on('end', () => this._close());
    }

    this._displayPrompt();
  }

  _defineDefaultCommands() {
    this.defineCommand('break', {
      help: 'Cancel current input',
      action: () => {
        this._bufferedCommand = '';
        this._bracketStack = [];
        this._lines = [];
        this._displayPrompt();
      },
    });
    this.defineCommand('clear', {
      help: 'Clear the REPL context',
      action: () => {
        this._context = Object.create(null);
        this._displayPrompt();
      },
    });
    this.defineCommand('exit', {
      help: 'Exit the REPL',
      action: () => this._close(),
    });
    this.defineCommand('help', {
      help: 'Show this help message',
      action: () => {
        const lines = ['REPL commands:', ''];
        for (const [name, cmd] of this._commands) {
          lines.push(`  .${name} - ${cmd.help || ''}`);
        }
        this._output.write(lines.join('\n') + '\n');
        this._displayPrompt();
      },
    });
    this.defineCommand('save', {
      help: 'Save all evaluated commands in this REPL session to a file',
      action: (filename) => {
        if (!filename) return this._output.write('Usage: .save <file>\n');
        try {
          const fs = globalThis.require('fs');
          fs.writeFileSync(filename, this._history.join('\n'));
          this._output.write('Session saved to: ' + filename + '\n');
        } catch (e) {
          this._output.write('Error saving: ' + e.message + '\n');
        }
        this._displayPrompt();
      },
    });
    this.defineCommand('load', {
      help: 'Load JS from a file into the REPL session',
      action: (filename) => {
        if (!filename) return this._output.write('Usage: .load <file>\n');
        try {
          const fs = globalThis.require('fs');
          const content = fs.readFileSync(filename, 'utf8');
          this._output.write(content + '\n');
          this._evalCommand(content);
        } catch (e) {
          this._output.write('Error loading: ' + e.message + '\n');
        }
      },
    });
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

  defineCommand(keyword, cmd) {
    if (typeof cmd === 'function') {
      this._commands.set(keyword, { action: cmd, help: '' });
    } else {
      this._commands.set(keyword, {
        action: cmd.action || (() => {}),
        help: cmd.help || '',
      });
    }
  }

  displayPrompt(preserveCursor) {
    this._output.write(this._prompt);
  }

  _displayPrompt() {
    if (!this._closed) {
      const continuation = this._bracketStack.length > 0;
      this._output.write(continuation ? '... ' : this._prompt);
    }
  }

  _handleInput(str) {
    for (const ch of str) {
      if (ch === '\n' || ch === '\r') {
        const line = this._lines.join('\n');
        if (line) {
          this._bufferedCommand += line;
          this._evalCommand(line);
        }
        this._lines = [];
        this._displayPrompt();
      } else {
        this._lines.push(ch);
      }
    }
  }

  _evalCommand(input) {
    const trimmed = input.trim();
    if (!trimmed) {
      this._displayPrompt();
      return;
    }

    if (trimmed.startsWith('.')) {
      const parts = trimmed.slice(1).split(/\s+/);
      const cmdName = parts[0];
      const cmdArgs = parts.slice(1).join(' ');
      const cmd = this._commands.get(cmdName);
      if (cmd) {
        try {
          cmd.action(cmdArgs);
        } catch (e) {
          this._output.write(`Error executing .${cmdName}: ${e.message}\n`);
          this._displayPrompt();
        }
      } else {
        this._output.write(`Invalid REPL command: .${cmdName}\n`);
        this._displayPrompt();
      }
      return;
    }

    const isIncomplete = this._isIncomplete(input);
    if (isIncomplete) {
      this._bufferedCommand += '\n';
      this._displayPrompt();
      return;
    }

    this._history.push(input);
    this._historyIndex = -1;

    const output = this._bufferedCommand || input;
    this._bufferedCommand = '';

    try {
      const result = this._eval(this._eval, this._context, output);
      if (result !== undefined && !this._ignoreUndefined) {
        const display = this._writer(result);
        this._output.write(display + '\n');
      }
    } catch (e) {
      if (e instanceof Recoverable) {
        this._bufferedCommand += '\n' + input;
        this._displayPrompt();
      } else {
        const display = this._writer(e);
        if (this._output && this._output.write) {
          this._output.write(display + '\n');
        }
      }
    }
    this._displayPrompt();
  }

  _isIncomplete(input) {
    const opens = (input.match(/[{[(]/g) || []).length;
    const closes = (input.match(/[}\])]/g) || []).length;
    if (opens > closes) return true;
    try {
      new Function(input);
      return false;
    } catch (e) {
      return _recoverableErrors.has(e.toString());
    }
  }

  _defaultEval(cmd, context, filename, callback) {
    try {
      let result;
      if (this._useGlobal) {
        result = eval(cmd);
      } else {
        const keys = Object.keys(context);
        const vals = keys.map(k => context[k]);
        const fn = new Function('require', ...keys, `"use strict"; return (${cmd})`);
        result = fn.call(context, globalThis.require, ...vals);
      }
      callback(null, result);
    } catch (e) {
      callback(e);
    }
  }

  _defaultWriter(obj) {
    if (obj === null) return 'null';
    if (obj === undefined) return 'undefined';
    if (typeof obj === 'string') return `'${obj}'`;
    if (obj instanceof Error) return obj.stack || obj.message;
    if (typeof obj === 'function') return `[Function: ${obj.name || 'anonymous'}]`;
    if (typeof obj === 'object') {
      try {
        const util = globalThis.require('util');
        return util.inspect(obj, { colors: this._useColors, depth: 2 });
      } catch (e) {
        return JSON.stringify(obj, null, 2);
      }
    }
    return String(obj);
  }

  clearBufferedCommand() {
    this._bufferedCommand = '';
    this._bracketStack = [];
    this._lines = [];
  }

  setupHistory(path, callback) {
    try {
      const fs = globalThis.require('fs');
      if (fs.existsSync(path)) {
        const content = fs.readFileSync(path, 'utf8');
        this._history = content.split('\n').filter(Boolean).slice(-1000);
      }
      this._historyPath = path;
      if (callback) callback(null);
    } catch (e) {
      if (callback) callback(e);
    }
  }

  _close() {
    if (this._closed) return;
    this._closed = true;
    if (this._historyPath && this._history.length > 0) {
      try {
        const fs = globalThis.require('fs');
        fs.writeFileSync(this._historyPath, this._history.join('\n'));
      } catch (e) {}
    }
    this.emit('exit');
  }

  get prompt() { return this._prompt; }
  set prompt(v) { this._prompt = v; }
  get closed() { return this._closed; }
  get context() { return this._context; }
  get history() { return [...this._history]; }
  get bufferedCommand() { return this._bufferedCommand; }
}

function start(options = {}) {
  return new REPLServer(options);
}

const writer = (obj) => {
  try {
    const util = globalThis.require('util');
    return util.inspect(obj, { colors: false, depth: 2 });
  } catch (e) {
    return JSON.stringify(obj, null, 2);
  }
};

const repl = {
  REPLServer,
  start,
  writer,
  Recoverable,
  REPL_MODE_SLOPPY,
  REPL_MODE_STRICT,
  REPL_MODE_MAGIC,
};

if (typeof module !== 'undefined' && module.exports) {
  module.exports = repl;
}
