import { op_set_timeout, op_clear_timer } from "ext:core/ops";

let timerCounter = 0;

globalThis.setTimeout = (callback, delay, ...args) => {
  const id = ++timerCounter;
  const delayMs = Math.max(delay || 0, 0);
  op_set_timeout(delayMs);
  if (delayMs === 0) {
    Promise.resolve().then(() => {
      if (globalThis.__klyron_pending_timeouts?.has(id)) {
        globalThis.__klyron_pending_timeouts.delete(id);
        callback(...args);
      }
    });
  }
  if (!globalThis.__klyron_pending_timeouts) {
    globalThis.__klyron_pending_timeouts = new Map();
  }
  globalThis.__klyron_pending_timeouts.set(id, { callback, args });
  return id;
};

globalThis.clearTimeout = (id) => {
  if (globalThis.__klyron_pending_timeouts) {
    globalThis.__klyron_pending_timeouts.delete(id);
  }
  op_clear_timer();
};

globalThis.setInterval = (callback, delay, ...args) => {
  const id = ++timerCounter;
  op_set_timeout(delay || 0);
  return id;
};

globalThis.clearInterval = (id) => {
  op_clear_timer();
};

globalThis.queueMicrotask = (callback) => {
  Promise.resolve().then(callback);
};
