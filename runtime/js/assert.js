// Klyron Runtime — node:assert polyfill

function assert(value, message) {
  if (!value) {
    throw new AssertionError({ message: message || 'assertion failed', actual: value, expected: true, operator: '==' });
  }
}

class AssertionError extends Error {
  constructor(opts = {}) {
    super(opts.message || 'Assertion failed');
    this.name = 'AssertionError';
    this.actual = opts.actual;
    this.expected = opts.expected;
    this.operator = opts.operator || '==';
    this.generatedMessage = !!opts.generatedMessage;
    this.code = 'ERR_ASSERTION';
  }
}

assert.AssertionError = AssertionError;

assert.ok = assert;

assert.strictEqual = (actual, expected, message) => {
  if (actual !== expected) {
    throw new AssertionError({ message: message || `${actual} !== ${expected}`, actual, expected, operator: '===' });
  }
};

assert.equal = (actual, expected, message) => {
  if (actual != expected) {
    throw new AssertionError({ message: message || `${actual} == ${expected}`, actual, expected, operator: '==' });
  }
};

assert.notStrictEqual = (actual, expected, message) => {
  if (actual === expected) {
    throw new AssertionError({ message: message || `${actual} === ${expected}`, actual, expected, operator: '!==' });
  }
};

assert.notEqual = (actual, expected, message) => {
  if (actual == expected) {
    throw new AssertionError({ message: message || `${actual} != ${expected}`, actual, expected, operator: '!=' });
  }
};

assert.deepEqual = (actual, expected, message) => {
  const str = JSON.stringify;
  if (str(actual) !== str(expected)) {
    throw new AssertionError({ message: message || `${str(actual)} deepEqual ${str(expected)}`, actual, expected, operator: 'deepEqual' });
  }
};

assert.deepStrictEqual = assert.deepEqual;

assert.throws = (fn, error, message) => {
  try {
    fn();
    throw new AssertionError({ message: message || 'Missing expected exception', operator: 'throws' });
  } catch (e) {
    if (error) {
      if (typeof error === 'function' && !(e instanceof error)) {
        throw new AssertionError({ message: message || `Expected ${error.name} but got ${e.name}`, actual: e, expected: error, operator: 'throws' });
      }
    }
  }
};

assert.doesNotThrow = (fn, message) => {
  try { fn(); }
  catch (e) { throw new AssertionError({ message: message || `Unexpected exception: ${e.message}`, actual: e, operator: 'doesNotThrow' }); }
};

assert.rejects = async (asyncFn, error, message) => {
  try { await (typeof asyncFn === 'function' ? asyncFn() : asyncFn); }
  catch (e) {
    if (error && typeof error === 'function' && !(e instanceof error)) {
      throw new AssertionError({ message: message || `Expected ${error.name} but got ${e.name}`, actual: e, expected: error, operator: 'rejects' });
    }
    return;
  }
  throw new AssertionError({ message: message || 'Missing expected rejection', operator: 'rejects' });
};

assert.doesNotReject = async (asyncFn, message) => {
  try { await (typeof asyncFn === 'function' ? asyncFn() : asyncFn); }
  catch (e) { throw new AssertionError({ message: message || `Unexpected rejection: ${e.message}`, actual: e, operator: 'doesNotReject' }); }
};

assert.fail = (message) => {
  throw new AssertionError({ message: message || 'Failed' });
};

assert.ifError = (err) => {
  if (err) throw err;
};

assert.match = (str, regex, message) => {
  if (!regex.test(str)) {
    throw new AssertionError({ message: message || `${str} does not match ${regex}`, operator: 'match' });
  }
};

assert.doesNotMatch = (str, regex, message) => {
  if (regex.test(str)) {
    throw new AssertionError({ message: message || `${str} matches ${regex}`, operator: 'doesNotMatch' });
  }
};

if (typeof module !== 'undefined' && module.exports) {
  module.exports = assert;
}
