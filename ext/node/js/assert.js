export function ok(val, msg) {
  if (!val) throw new AssertionError(msg || "assertion failed");
}

export function equal(actual, expected, msg) {
  if (actual != expected) throw new AssertionError(msg || `${actual} == ${expected}`);
}

export function strictEqual(actual, expected, msg) {
  if (actual !== expected) throw new AssertionError(msg || `${actual} === ${expected}`);
}

export function deepEqual(actual, expected, msg) {
  try { if (!deepEqualImpl(actual, expected)) throw new Error(); }
  catch (e) { throw new AssertionError(msg || `${JSON.stringify(actual)} deepEqual ${JSON.stringify(expected)}`); }
}

function deepEqualImpl(a, b) {
  if (a === b) return true;
  if (a === null || b === null || typeof a !== "object" || typeof b !== "object") return a === b || (Number.isNaN(a) && Number.isNaN(b));
  if (Array.isArray(a)) { if (!Array.isArray(b) || a.length !== b.length) return false; return a.every((v, i) => deepEqualImpl(v, b[i])); }
  const ka = Object.keys(a), kb = Object.keys(b);
  if (ka.length !== kb.length) return false;
  return ka.every(k => deepEqualImpl(a[k], b[k]));
}

export function notEqual(actual, expected, msg) {
  if (actual == expected) throw new AssertionError(msg || `${actual} != ${expected}`);
}

export function notStrictEqual(actual, expected, msg) {
  if (actual === expected) throw new AssertionError(msg || `${actual} !== ${expected}`);
}

export function throws(fn, error, msg) {
  try { fn(); throw new AssertionError(msg || "Expected function to throw"); }
  catch (e) { if (error && !(e instanceof error)) throw new AssertionError(msg || `Expected ${error.name}, got ${e.constructor?.name}`); }
}

export function doesNotThrow(fn, msg) {
  try { fn(); }
  catch (e) { throw new AssertionError(msg || `Unexpected: ${e.message}`); }
}

export function fail(msg) { throw new AssertionError(msg || "Failed"); }

class AssertionError extends Error {
  constructor(msg) { super(msg); this.name = "AssertionError"; }
}

export default { ok, equal, strictEqual, deepEqual, notEqual, notStrictEqual, throws, doesNotThrow, fail, AssertionError };
