// Klyron Runtime — Timers (Node.js compatible)
// setImmediate, clearImmediate

if (typeof globalThis.setImmediate !== 'function') {
  globalThis.setImmediate = (fn, ...args) => {
    if (typeof fn === 'string') fn = new Function(fn);
    return setTimeout(fn, 0, ...args);
  };
}

if (typeof globalThis.clearImmediate !== 'function') {
  globalThis.clearImmediate = (id) => clearTimeout(id);
}
