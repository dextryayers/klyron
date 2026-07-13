((globalThis) => {
  const core = globalThis.Deno?.core || globalThis.__bootstrap__?.core;

  let timerCounter = 0;

  globalThis.setTimeout = (callback, delay, ...args) => {
    const id = ++timerCounter;
    const delayMs = Math.max(delay || 0, 0);
    core?.ops?.op_set_timeout(delayMs);
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
    globalThis.__klyron_pending_timeouts.set(id, { callback, args, delay: delayMs });
    if (delayMs <= 0) {
      queueMicrotask(() => {
        if (globalThis.__klyron_pending_timeouts?.has(id)) {
          const entry = globalThis.__klyron_pending_timeouts.get(id);
          globalThis.__klyron_pending_timeouts.delete(id);
          entry.callback(...entry.args);
        }
      });
    }
    return id;
  };

  globalThis.clearTimeout = (id) => {
    globalThis.__klyron_pending_timeouts?.delete(id);
    core?.ops?.op_clear_timer();
  };

  globalThis.setInterval = (callback, delay, ...args) => {
    const id = ++timerCounter;
    core?.ops?.op_set_timeout(delay || 0);
    const tick = () => {
      if (!globalThis.__klyron_pending_intervals?.has(id)) return;
      callback(...args);
      if (globalThis.__klyron_pending_intervals?.has(id)) {
        core?.ops?.op_set_timeout(delay || 0);
      }
    };
    if (!globalThis.__klyron_pending_intervals) {
      globalThis.__klyron_pending_intervals = new Map();
    }
    globalThis.__klyron_pending_intervals.set(id, { callback, args, delay });
    queueMicrotask(tick);
    return id;
  };

  globalThis.clearInterval = (id) => {
    globalThis.__klyron_pending_intervals?.delete(id);
    core?.ops?.op_clear_timer();
  };

  globalThis.queueMicrotask = (callback) => {
    Promise.resolve().then(callback);
  };
})(globalThis);
