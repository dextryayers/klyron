import { op_console_log, op_console_error, op_console_warn, op_console_info } from "ext:core/ops";

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
  log: (...args) => op_console_log(formatArgs(...args)),
  error: (...args) => op_console_error(formatArgs(...args)),
  warn: (...args) => op_console_warn(formatArgs(...args)),
  info: (...args) => op_console_info(formatArgs(...args)),
  debug: (...args) => op_console_log(formatArgs(...args)),
  trace: (...args) => op_console_log(formatArgs(...args)),
};
