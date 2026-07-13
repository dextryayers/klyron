export function inherits(ctor, superCtor) {
  Object.setPrototypeOf(ctor.prototype, superCtor.prototype);
  ctor.super_ = superCtor;
}

export function promisify(fn) {
  return function (...args) {
    return new Promise((resolve, reject) => {
      fn(...args, (err, ...results) => {
        if (err) reject(err);
        else resolve(results.length > 1 ? results : results[0]);
      });
    });
  };
}

export function format(f, ...args) {
  if (typeof f !== "string") return args.map(String).join(" ");
  let i = 0;
  return f.replace(/%[sdifjNo]/g, m => { const val = args[i++]; if (val === undefined) return ""; if (m === "%s") return String(val); if (m === "%d" || m === "%i") return parseInt(val); if (m === "%f") return parseFloat(val); if (m === "%j") return JSON.stringify(val); return String(val); });
}

export function deprecate(fn, msg) {
  let warned = false;
  return function (...args) {
    if (!warned) { console.warn("DeprecationWarning:", msg); warned = true; }
    return fn.apply(this, args);
  };
}

export function types() {
  return {
    isDate: v => v instanceof Date,
    isRegExp: v => v instanceof RegExp,
    isArray: v => Array.isArray(v),
    isBoolean: v => typeof v === "boolean",
    isNumber: v => typeof v === "number",
    isString: v => typeof v === "string",
    isFunction: v => typeof v === "function",
    isObject: v => v !== null && typeof v === "object" && !Array.isArray(v),
    isNull: v => v === null,
    isUndefined: v => v === undefined,
  };
}

export function debuglog(section) {
  return function(...args) { console.error(`[${section}]`, ...args); };
}

export function inspect(obj, opts) {
  if (obj === null) return "null";
  if (obj === undefined) return "undefined";
  if (typeof obj === "string") return `'${obj}'`;
  if (typeof obj === "number" || typeof obj === "boolean") return String(obj);
  if (typeof obj === "function") return `[Function${obj.name ? ": " + obj.name : ""}]`;
  if (Array.isArray(obj)) return `[${obj.map(v => inspect(v, opts)).join(", ")}]`;
  if (obj instanceof Date) return obj.toISOString();
  if (obj instanceof RegExp) return obj.toString();
  if (obj instanceof Error) return `${obj.name}: ${obj.message}`;
  const keys = Object.keys(obj);
  if (keys.length === 0) return "{}";
  const depth = opts?.depth ?? 2;
  if (depth <= 0) return "[Object]";
  return `{ ${keys.map(k => `${k}: ${inspect(obj[k], { ...opts, depth: depth - 1 })}`).join(", ")} }`;
}

export default { inherits, promisify, format, deprecate, types, debuglog, inspect };
