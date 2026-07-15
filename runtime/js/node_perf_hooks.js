// Klyron Runtime — node:perf_hooks polyfill

const perf = typeof globalThis.performance !== 'undefined' ? globalThis.performance : {
  now() {
    if (typeof Klyron !== 'undefined' && Klyron.hrtime) {
      const [sec, nsec] = Klyron.hrtime();
      return sec * 1000 + nsec / 1e6;
    }
    return Date.now();
  },
  mark(name) { this._marks = this._marks || new Map(); this._marks.set(name, this.now()); },
  measure(name, startMark, endMark) {
    this._marks = this._marks || new Map();
    const start = startMark ? (this._marks.get(startMark) || 0) : 0;
    const end = endMark ? (this._marks.get(endMark) || this.now()) : this.now();
    return end - start;
  },
  clearMarks(name) { this._marks = this._marks || new Map(); if (name) this._marks.delete(name); else this._marks.clear(); },
  clearMeasures(name) { this._measures = this._measures || new Map(); if (name) this._measures.delete(name); else this._measures.clear(); },
  getEntriesByType(type) { return []; },
  toJSON() { return {}; },
};

class PerformanceEntry {
  constructor(init = {}) {
    this.name = init.name || '';
    this.entryType = init.entryType || '';
    this.startTime = init.startTime || 0;
    this.duration = init.duration || 0;
    this._detail = init.detail || null;
  }

  toJSON() {
    return {
      name: this.name,
      entryType: this.entryType,
      startTime: this.startTime,
      duration: this.duration,
    };
  }
}

class PerformanceMark extends PerformanceEntry {
  constructor(name, options = {}) {
    super({ name, entryType: 'mark', startTime: options.startTime || perf.now(), duration: 0, detail: options.detail });
  }

  get detail() { return this._detail; }
}

class PerformanceMeasure extends PerformanceEntry {
  constructor(name, options = {}) {
    super({ name, entryType: 'measure', startTime: options.startTime || 0, duration: options.duration || 0, detail: options.detail });
  }

  get detail() { return this._detail; }
}

class PerformanceObserver {
  constructor(callback) {
    if (typeof callback !== 'function') throw new TypeError('callback must be a function');
    this._callback = callback;
    this._buffer = [];
    this._observed = false;
    this._entryTypes = [];
  }

  observe(options) {
    if (typeof options === 'object') {
      this._entryTypes = options.entryTypes || [];
      this._buffered = options.buffered || false;
      this._observed = true;
      this._type = 'single';
    } else if (typeof options === 'string') {
      this._entryTypes = [options];
      this._observed = true;
    }
  }

  disconnect() {
    this._observed = false;
    this._buffer = [];
  }

  takeRecords() {
    const records = this._buffer;
    this._buffer = [];
    return records;
  }

  get supportedEntryTypes() {
    return ['mark', 'measure', 'function', 'gc', 'http', 'http2'];
  }
}

const constants = {
  NODE_PERFORMANCE_GC_MAJOR: 0,
  NODE_PERFORMANCE_GC_MINOR: 1,
  NODE_PERFORMANCE_GC_INCREMENTAL: 2,
  NODE_PERFORMANCE_GC_WEAKCB: 3,
  NODE_PERFORMANCE_GC_FLAGS_NO: 0,
  NODE_PERFORMANCE_GC_FLAGS_CONSTRUCT_RETAINED: 1,
  NODE_PERFORMANCE_GC_FLAGS_FORCED: 2,
  NODE_PERFORMANCE_GC_FLAGS_SYNCHRONOUS_PHANTOM_PROCESSING: 4,
  NODE_PERFORMANCE_GC_FLAGS_ALL_AVAILABLE_GARBAGE: 8,
  NODE_PERFORMANCE_GC_FLAGS_ALL_EXTERNAL_MEMORY: 16,
  NODE_PERFORMANCE_GC_FLAGS_SCHEDULE_IDLE: 32,
  NODE_PERFORMANCE_ENTRY_TYPE_GC: 0,
  NODE_PERFORMANCE_ENTRY_TYPE_HTTP: 1,
  NODE_PERFORMANCE_ENTRY_TYPE_HTTP2: 2,
  NODE_PERFORMANCE_ENTRY_TYPE_FUNCTION: 3,
  NODE_PERFORMANCE_ENTRY_TYPE_NET: 4,
};

class IntervalHistogram {
  constructor() {
    this._min = Infinity;
    this._max = -Infinity;
    this._mean = 0;
    this._stddev = 0;
    this._count = 0;
    this._exceeds = 0;
    this._total = 0;
    this._id = null;
    this._started = false;
  }

  get min() { return this._min === Infinity ? 0 : this._min; }
  get max() { return this._max === -Infinity ? 0 : this._max; }
  get mean() { return this._mean; }
  get stddev() { return this._stddev; }
  get count() { return this._count; }
  get exceeds() { return this._exceeds; }

  reset() {
    this._min = Infinity;
    this._max = -Infinity;
    this._mean = 0;
    this._stddev = 0;
    this._count = 0;
    this._exceeds = 0;
    this._total = 0;
  }

  enable() {
    if (this._started) return true;
    this._started = true;
    return true;
  }

  disable() {
    if (!this._started) return false;
    this._started = false;
    return true;
  }

  _record(value) {
    if (!this._started) return;
    this._count++;
    this._total += value;
    this._mean = this._total / this._count;
    this._min = Math.min(this._min, value);
    this._max = Math.max(this._max, value);
  }
}

class RecordableHistogram {
  constructor(options) {
    this._records = [];
    this._count = 0;
    this._max = -Infinity;
    this._min = Infinity;
    this._sum = 0;
  }

  record(val) {
    this._records.push(val);
    this._count++;
    this._sum += val;
    this._min = Math.min(this._min, val);
    this._max = Math.max(this._max, val);
  }

  recordDelta() {
    return 0;
  }

  get count() { return this._count; }
  get max() { return this._max === -Infinity ? 0 : this._max; }
  get min() { return this._min === Infinity ? 0 : this._min; }
  get mean() { return this._count > 0 ? this._sum / this._count : 0; }
  get exceeds() { return 0; }
  get stddev() { return 0; }

  percentile(percentile) {
    if (this._records.length === 0) return 0;
    const sorted = [...this._records].sort((a, b) => a - b);
    const index = Math.ceil((percentile / 100) * sorted.length) - 1;
    return sorted[Math.max(0, index)];
  }

  reset() {
    this._records = [];
    this._count = 0;
    this._max = -Infinity;
    this._min = Infinity;
    this._sum = 0;
  }
}

function monitorEventLoopDelay(options = {}) {
  return new IntervalHistogram();
}

function createHistogram(options = {}) {
  return new RecordableHistogram(options);
}

const performance = perf;

const perf_hooks = {
  performance,
  PerformanceObserver,
  PerformanceEntry,
  PerformanceMark,
  PerformanceMeasure,
  monitorEventLoopDelay,
  createHistogram,
  constants,
  IntervalHistogram,
  RecordableHistogram,
};

if (typeof module !== 'undefined' && module.exports) {
  module.exports = perf_hooks;
}
