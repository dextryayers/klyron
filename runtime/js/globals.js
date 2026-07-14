// Klyron Runtime — GlobalThis Bootstrapping
// Web + Node compatible globals

globalThis.global = globalThis;

if (typeof globalThis.setTimeout !== 'function') {
  globalThis.setTimeout = (fn, ms, ...args) => {
    if (typeof fn === 'string') fn = new Function(fn);
    return Klyron.timers.setTimeout(fn, ms, ...args);
  };
}

if (typeof globalThis.clearTimeout !== 'function') {
  globalThis.clearTimeout = (id) => Klyron.timers.clearTimeout(id);
}

if (typeof globalThis.setInterval !== 'function') {
  globalThis.setInterval = (fn, ms, ...args) => {
    if (typeof fn === 'string') fn = new Function(fn);
    return Klyron.timers.setInterval(fn, ms, ...args);
  };
}

if (typeof globalThis.clearInterval !== 'function') {
  globalThis.clearInterval = (id) => Klyron.timers.clearInterval(id);
}

if (typeof globalThis.queueMicrotask !== 'function') {
  globalThis.queueMicrotask = (fn) => Promise.resolve().then(fn);
}

if (typeof globalThis.structuredClone !== 'function') {
  globalThis.structuredClone = (obj) => JSON.parse(JSON.stringify(obj));
}
