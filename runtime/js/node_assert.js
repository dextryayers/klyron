// Klyron Runtime — node:assert polyfill

class AssertionError extends Error {
  constructor(opts = {}) {
    super(typeof opts === 'string' ? opts : (opts.message || 'Assertion failed'));
    if (typeof opts === 'object' && opts !== null) {
      this.name = 'AssertionError';
      this.actual = opts.actual;
      this.expected = opts.expected;
      this.operator = opts.operator || '==';
      this.generatedMessage = !!opts.generatedMessage;
      this.code = 'ERR_ASSERTION';
    } else {
      this.name = 'AssertionError';
      this.code = 'ERR_ASSERTION';
    }
  }
}

function assert(value, message) {
  if (!value) {
    throw new AssertionError({
      message: message || 'assertion failed',
      actual: value,
      expected: true,
      operator: '==',
      generatedMessage: !message,
    });
  }
}

assert.AssertionError = AssertionError;

assert.ok = assert;

assert.strictEqual = function strictEqual(actual, expected, message) {
  if (actual !== expected) {
    throw new AssertionError({
      message: message || `${JSON.stringify(actual)} !== ${JSON.stringify(expected)}`,
      actual,
      expected,
      operator: '===',
      generatedMessage: !message,
    });
  }
};

assert.equal = function equal(actual, expected, message) {
  if (actual != expected) {
    throw new AssertionError({
      message: message || `${JSON.stringify(actual)} == ${JSON.stringify(expected)}`,
      actual,
      expected,
      operator: '==',
      generatedMessage: !message,
    });
  }
};

assert.notEqual = function notEqual(actual, expected, message) {
  if (actual == expected) {
    throw new AssertionError({
      message: message || `${JSON.stringify(actual)} != ${JSON.stringify(expected)}`,
      actual,
      expected,
      operator: '!=',
      generatedMessage: !message,
    });
  }
};

assert.notStrictEqual = function notStrictEqual(actual, expected, message) {
  if (actual === expected) {
    throw new AssertionError({
      message: message || `${JSON.stringify(actual)} !== ${JSON.stringify(expected)}`,
      actual,
      expected,
      operator: '!==',
      generatedMessage: !message,
    });
  }
};

function isPrimitive(val) {
  return val === null || (typeof val !== 'object' && typeof val !== 'function');
}

function deepEqual(a, b, strict) {
  if (a === b) return true;
  if (isPrimitive(a) || isPrimitive(b)) return strict ? a === b : a == b;
  if (a instanceof Date && b instanceof Date) return a.getTime() === b.getTime();
  if (a instanceof RegExp && b instanceof RegExp) return a.toString() === b.toString();
  if (a instanceof Error && b instanceof Error) return a.message === b.message;
  const aKeys = Object.keys(a);
  const bKeys = Object.keys(b);
  if (aKeys.length !== bKeys.length) return false;
  for (const key of aKeys) {
    if (!b.hasOwnProperty(key)) return false;
    if (!deepEqual(a[key], b[key], strict)) return false;
  }
  return true;
}

assert.deepEqual = function deepEqualAssert(actual, expected, message) {
  if (!deepEqual(actual, expected, false)) {
    throw new AssertionError({
      message: message || `${JSON.stringify(actual)} deepEqual ${JSON.stringify(expected)}`,
      actual,
      expected,
      operator: 'deepEqual',
      generatedMessage: !message,
    });
  }
};

assert.deepStrictEqual = function deepStrictEqual(actual, expected, message) {
  if (!deepEqual(actual, expected, true)) {
    throw new AssertionError({
      message: message || `${JSON.stringify(actual)} deepStrictEqual ${JSON.stringify(expected)}`,
      actual,
      expected,
      operator: 'deepStrictEqual',
      generatedMessage: !message,
    });
  }
};

assert.notDeepEqual = function notDeepEqual(actual, expected, message) {
  if (deepEqual(actual, expected, false)) {
    throw new AssertionError({
      message: message || `${JSON.stringify(actual)} notDeepEqual ${JSON.stringify(expected)}`,
      actual,
      expected,
      operator: 'notDeepEqual',
      generatedMessage: !message,
    });
  }
};

assert.notDeepStrictEqual = function notDeepStrictEqual(actual, expected, message) {
  if (deepEqual(actual, expected, true)) {
    throw new AssertionError({
      message: message || `${JSON.stringify(actual)} notDeepStrictEqual ${JSON.stringify(expected)}`,
      actual,
      expected,
      operator: 'notDeepStrictEqual',
      generatedMessage: !message,
    });
  }
};

assert.throws = function throws(fn, error, message) {
  if (typeof error === 'string') { message = error; error = undefined; }
  try {
    fn();
  } catch (e) {
    if (error) {
      if (typeof error === 'function') {
        if (!(e instanceof error)) {
          throw new AssertionError({
            message: message || `Expected error of type ${error.name} but got ${e.constructor.name}`,
            actual: e,
            expected: error,
            operator: 'throws',
            generatedMessage: !message,
          });
        }
      } else if (error instanceof RegExp) {
        if (!error.test(e.message)) {
          throw new AssertionError({
            message: message || `Expected error message to match ${error} but got '${e.message}'`,
            actual: e,
            expected: error,
            operator: 'throws',
            generatedMessage: !message,
          });
        }
      } else if (typeof error === 'object') {
        for (const k of Object.keys(error)) {
          if (e[k] !== error[k]) {
            throw new AssertionError({
              message: message || `Expected error.${k} to be ${error[k]} but got ${e[k]}`,
              actual: e,
              expected: error,
              operator: 'throws',
              generatedMessage: !message,
            });
          }
        }
      }
    }
    return;
  }
  throw new AssertionError({
    message: message || 'Missing expected exception',
    operator: 'throws',
    generatedMessage: !message,
  });
};

assert.doesNotThrow = function doesNotThrow(fn, message) {
  try {
    fn();
  } catch (e) {
    throw new AssertionError({
      message: message || `Unexpected exception: ${e.message}`,
      actual: e,
      expected: 'no exception',
      operator: 'doesNotThrow',
      generatedMessage: !message,
    });
  }
};

assert.rejects = async function rejects(asyncFn, error, message) {
  if (typeof error === 'string') { message = error; error = undefined; }
  try {
    await (typeof asyncFn === 'function' ? asyncFn() : asyncFn);
  } catch (e) {
    if (error) {
      if (typeof error === 'function') {
        if (!(e instanceof error)) {
          throw new AssertionError({
            message: message || `Expected rejection of type ${error.name} but got ${e.constructor.name}`,
            actual: e,
            expected: error,
            operator: 'rejects',
            generatedMessage: !message,
          });
        }
      } else if (error instanceof RegExp) {
        if (!error.test(e.message)) {
          throw new AssertionError({
            message: message || `Expected rejection message to match ${error} but got '${e.message}'`,
            actual: e,
            expected: error,
            operator: 'rejects',
            generatedMessage: !message,
          });
        }
      } else if (typeof error === 'object') {
        for (const k of Object.keys(error)) {
          if (e[k] !== error[k]) {
            throw new AssertionError({
              message: message || `Expected error.${k} to be ${error[k]}`,
              actual: e,
              expected: error,
              operator: 'rejects',
              generatedMessage: !message,
            });
          }
        }
      }
    }
    return;
  }
  throw new AssertionError({
    message: message || 'Missing expected rejection',
    operator: 'rejects',
    generatedMessage: !message,
  });
};

assert.doesNotReject = async function doesNotReject(asyncFn, message) {
  try {
    await (typeof asyncFn === 'function' ? asyncFn() : asyncFn);
  } catch (e) {
    throw new AssertionError({
      message: message || `Unexpected rejection: ${e.message}`,
      actual: e,
      expected: 'no rejection',
      operator: 'doesNotReject',
      generatedMessage: !message,
    });
  }
};

assert.fail = function fail(message) {
  throw new AssertionError({
    message: message || 'Failed',
    operator: 'fail',
    generatedMessage: !message,
  });
};

assert.ifError = function ifError(value) {
  if (value) {
    throw new AssertionError({
      message: `ifError got unwanted exception: ${value.message || value}`,
      actual: value,
      operator: 'ifError',
    });
  }
};

assert.match = function match(str, regex, message) {
  if (!regex.test(str)) {
    throw new AssertionError({
      message: message || `'${str}' does not match '${regex}'`,
      actual: str,
      expected: regex,
      operator: 'match',
      generatedMessage: !message,
    });
  }
};

assert.doesNotMatch = function doesNotMatch(str, regex, message) {
  if (regex.test(str)) {
    throw new AssertionError({
      message: message || `'${str}' matches '${regex}'`,
      actual: str,
      expected: regex,
      operator: 'doesNotMatch',
      generatedMessage: !message,
    });
  }
};

if (typeof module !== 'undefined' && module.exports) {
  module.exports = assert;
}
