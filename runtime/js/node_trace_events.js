// Klyron Runtime — node:trace_events polyfill

const _enabledCategories = new Set();

class Tracing {
  constructor(options = {}) {
    this._enabled = false;
    this._categories = options.categories || '';
    this._categorySet = new Set(this._categories.split(',').map(s => s.trim()).filter(Boolean));
  }

  enable() {
    this._enabled = true;
    for (const cat of this._categorySet) {
      _enabledCategories.add(cat);
    }
  }

  disable() {
    this._enabled = false;
    for (const cat of this._categorySet) {
      _enabledCategories.delete(cat);
    }
  }

  get enabled() { return this._enabled; }
  get categories() { return this._categories; }
}

function createTracing(options = {}) {
  return new Tracing(options);
}

function getEnabledCategories() {
  if (_enabledCategories.size === 0) return undefined;
  return Array.from(_enabledCategories).join(',');
}

const tracing = {
  createTracing,
  getEnabledCategories,
  Tracing,
};

if (typeof module !== 'undefined' && module.exports) {
  module.exports = tracing;
}
