// Klyron Runtime — Performance API (Web/Node compatible)
// performance.now, performance.mark, performance.measure

class Performance {
  constructor() {
    this._marks = new Map();
    this._measures = new Map();
  }

  now() {
    const [sec, nsec] = Klyron.hrtime();
    return sec * 1000 + nsec / 1e6;
  }

  mark(name) {
    this._marks.set(name, this.now());
  }

  measure(name, startMark, endMark) {
    const start = startMark ? this._marks.get(startMark) : 0;
    const end = endMark ? this._marks.get(endMark) : this.now();
    const duration = end - start;
    this._measures.set(name, { name, duration, startTime: start, entryType: 'measure' });
    return duration;
  }

  clearMarks(name) {
    if (name) this._marks.delete(name);
    else this._marks.clear();
  }

  clearMeasures(name) {
    if (name) this._measures.delete(name);
    else this._measures.clear();
  }

  getEntriesByType(type) {
    if (type === 'mark') return Array.from(this._marks.entries()).map(([n, t]) => ({ name: n, startTime: t, entryType: 'mark', duration: 0 }));
    if (type === 'measure') return Array.from(this._measures.values());
    return [];
  }

  toJSON() { return {}; }
}

if (typeof globalThis.performance !== 'object') {
  globalThis.performance = new Performance();
}
