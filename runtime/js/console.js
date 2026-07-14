// Klyron Runtime — Console API
// Web + Node compatible console

class Console {
  constructor(stdout, stderr) {
    this._stdout = stdout || Klyron.io.stdout;
    this._stderr = stderr || Klyron.io.stderr;
    this._times = new Map();
    this._counters = new Map();
  }

  log(...args) { this._write(this._stdout, args); }
  info(...args) { this._write(this._stdout, args); }
  warn(...args) { this._write(this._stderr, args); }
  error(...args) { this._write(this._stderr, args); }
  debug(...args) { this._write(this._stdout, args); }

  table(data) {
    if (!data || typeof data !== 'object') return this.log(data);
    const isArray = Array.isArray(data);
    const keys = isArray ? Object.keys(data[0] || {}) : Object.keys(data);
    if (keys.length === 0) return this.log(data);
    const rows = isArray ? data : [data];
    const colWidths = keys.map(k => Math.max(k.length, ...rows.map(r => String(r[k] || '').length)));
    const sep = '+' + colWidths.map(w => '-'.repeat(w + 2)).join('+') + '+';
    const header = '| ' + keys.map((k, i) => k.padEnd(colWidths[i])).join(' | ') + ' |';
    this._stdout.write(sep + '\n');
    this._stdout.write(header + '\n');
    this._stdout.write(sep + '\n');
    for (const row of rows) {
      const line = '| ' + keys.map((k, i) => String(row[k] || '').padEnd(colWidths[i])).join(' | ') + ' |';
      this._stdout.write(line + '\n');
    }
    this._stdout.write(sep + '\n');
  }

  time(label = 'default') { this._times.set(label, Date.now()); }
  timeLog(label = 'default') {
    const start = this._times.get(label);
    if (start) this.log(`${label}: ${Date.now() - start}ms`);
  }
  timeEnd(label = 'default') {
    const start = this._times.get(label);
    if (start) {
      this.log(`${label}: ${Date.now() - start}ms`);
      this._times.delete(label);
    }
  }

  count(label = 'default') {
    this._counters.set(label, (this._counters.get(label) || 0) + 1);
    this.log(`${label}: ${this._counters.get(label)}`);
  }

  countReset(label = 'default') { this._counters.delete(label); }

  group(...args) {
    if (args.length) this.log(...args);
    this._indent = (this._indent || 0) + 1;
  }

  groupEnd() { this._indent = Math.max(0, (this._indent || 1) - 1); }
  trace() { this.error(new Error().stack); }
  dir(obj) { this.log(JSON.stringify(obj, null, 2)); }
  assert(condition, ...args) {
    if (!condition) throw new Error(`Assertion failed: ${args.join(' ')}`);
  }

  _write(stream, args) {
    const indent = '  '.repeat(this._indent || 0);
    stream.write(indent + args.map(a => typeof a === 'object' ? JSON.stringify(a, null, 2) : String(a)).join(' ') + '\n');
  }
}

if (typeof globalThis.console !== 'object') {
  globalThis.console = new Console();
}
