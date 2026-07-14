import { op_set_timeout, op_set_interval, op_clear_timer } from "ext:core/ops";

let timerCounter = 0;

const _timeouts = new Map();
const _intervals = new Map();

globalThis.setTimeout = (callback, delay, ...args) => {
  const id = ++timerCounter;
  const delayMs = Math.max(delay || 0, 0);
  op_set_timeout(delayMs);
  if (delayMs === 0) {
    Promise.resolve().then(() => {
      if (_timeouts.has(id)) {
        _timeouts.delete(id);
        callback(...args);
      }
    });
  }
  _timeouts.set(id, { callback, args });
  return id;
};

globalThis.clearTimeout = (id) => {
  _timeouts.delete(id);
  op_clear_timer(id);
};

globalThis.setInterval = (callback, delay, ...args) => {
  const id = ++timerCounter;
  const delayMs = Math.max(delay || 0, 1);
  op_set_interval(delayMs);
  const run = () => {
    if (!_intervals.has(id)) return;
    try { callback(...args); } catch (_) {}
    if (_intervals.has(id)) {
      _intervals.get(id).nextTick = setTimeout(run, delayMs);
    }
  };
  _intervals.set(id, { callback, args, nextTick: setTimeout(run, delayMs) });
  return id;
};

globalThis.clearInterval = (id) => {
  const iv = _intervals.get(id);
  if (iv) {
    clearTimeout(iv.nextTick);
    _intervals.delete(id);
  }
  op_clear_timer(id);
};

globalThis.queueMicrotask = (callback) => {
  Promise.resolve().then(callback);
};
