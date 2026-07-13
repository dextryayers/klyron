((globalThis) => {
  const core = globalThis.Deno?.core || globalThis.__bootstrap__?.core;

  const formatArgs = (...args) => {
    return args.map(a => {
      if (typeof a === 'object') {
        try { return JSON.stringify(a, null, 2); }
        catch { return String(a); }
      }
      return String(a);
    }).join(' ');
  };

  globalThis.console = {
    log: (...args) => core?.ops?.op_console_log(formatArgs(...args)),
    error: (...args) => core?.ops?.op_console_error(formatArgs(...args)),
    warn: (...args) => core?.ops?.op_console_warn(formatArgs(...args)),
    info: (...args) => core?.ops?.op_console_info(formatArgs(...args)),
    debug: (...args) => core?.ops?.op_console_log(formatArgs(...args)),
    trace: (...args) => {
      const err = new Error();
      const stack = err.stack?.split('\n').slice(2).join('\n') || '';
      core?.ops?.op_console_log(formatArgs(...args) + '\n' + stack);
    },
    assert: (condition, ...args) => {
      if (!condition) {
        throw new Error('Assertion failed: ' + formatArgs(...args));
      }
    },
    time: (label = 'default') => {
      if (!globalThis.__klyron_timers) globalThis.__klyron_timers = new Map();
      globalThis.__klyron_timers.set(label, performance.now());
    },
    timeEnd: (label = 'default') => {
      const start = globalThis.__klyron_timers?.get(label);
      if (start === undefined) {
        console.warn(`Timer '${label}' does not exist`);
        return;
      }
      const duration = performance.now() - start;
      console.log(`${label}: ${duration.toFixed(2)} ms`);
      globalThis.__klyron_timers.delete(label);
    },
    count: (label = 'default') => {
      if (!globalThis.__klyron_counts) globalThis.__klyron_counts = new Map();
      const count = (globalThis.__klyron_counts.get(label) || 0) + 1;
      globalThis.__klyron_counts.set(label, count);
      console.log(`${label}: ${count}`);
    },
    countReset: (label = 'default') => {
      globalThis.__klyron_counts?.set(label, 0);
    },
    group: () => {},
    groupEnd: () => {},
  };
})(globalThis);
