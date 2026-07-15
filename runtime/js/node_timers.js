// Klyron Runtime — node:timers polyfill

function setImmediate(callback, ...args) {
  if (typeof callback !== 'function') throw new TypeError('callback must be a function');
  const id = {};
  queueMicrotask(() => {
    try {
      callback(...args);
    } catch (e) {
      queueMicrotask(() => { throw e; });
    }
  });
  return id;
}

function clearImmediate(immediateId) {}

function setTimeoutPromise(delay, value, options) {
  return new Promise((resolve) => {
    globalThis.setTimeout(() => resolve(value), delay);
  });
}

function setImmediatePromise(value) {
  return new Promise((resolve) => {
    setImmediate(() => resolve(value));
  });
}

async function* setIntervalIterator(delay, value, options) {
  while (true) {
    yield await new Promise((resolve) => {
      globalThis.setTimeout(() => resolve(value), delay);
    });
  }
}

const timersPromises = {
  setTimeout: setTimeoutPromise,
  setImmediate: setImmediatePromise,
  setInterval: setIntervalIterator,
  scheduler: {
    wait: (delay, options) => setTimeoutPromise(delay, undefined, options),
    yield: () => setImmediatePromise(),
    postTask: (task, options) => setImmediate(task),
  },
};

const timers = {
  setImmediate,
  clearImmediate,
  promises: timersPromises,
  setTimeout: globalThis.setTimeout,
  clearTimeout: globalThis.clearTimeout,
  setInterval: globalThis.setInterval,
  clearInterval: globalThis.clearInterval,
};

if (typeof module !== 'undefined' && module.exports) {
  module.exports = timers;
}
