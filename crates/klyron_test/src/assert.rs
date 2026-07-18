use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsAssertionResult {
    pub passed: bool,
    pub description: String,
    pub error: Option<String>,
    pub location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsTestCaseResult {
    pub name: String,
    pub passed: bool,
    pub assertions: Vec<JsAssertionResult>,
    pub duration_ms: u64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsTestSuiteResult {
    pub name: String,
    pub passed: bool,
    pub tests: Vec<JsTestCaseResult>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JsTestResults {
    pub suites: Vec<JsTestSuiteResult>,
    pub passed: usize,
    pub failed: usize,
    pub total: usize,
    pub duration_ms: u64,
}

impl JsTestResults {
    pub fn passed(&self) -> usize {
        self.tests().iter().filter(|t| t.passed).count()
    }

    pub fn failed(&self) -> usize {
        self.tests().iter().filter(|t| !t.passed).count()
    }

    pub fn tests(&self) -> Vec<&JsTestCaseResult> {
        self.suites.iter().flat_map(|s| s.tests.iter()).collect()
    }

    pub fn summary(&self) -> String {
        format!(
            "Tests: {} passed, {} failed, {} total ({}.{:03}s)",
            self.passed(),
            self.failed(),
            self.total,
            self.duration_ms / 1000,
            self.duration_ms % 1000,
        )
    }
}

pub fn generate_assertion_globals() -> String {
    r#"
(function() {
    const __klyron = {
        suites: [],
        currentSuite: null,
        currentTest: null,
        assertions: [],
    };

    function describe(name, fn) {
        const suite = { name, tests: [], before: [], after: [], beforeEach: [], afterEach: [] };
        __klyron.currentSuite = suite;
        fn();
        __klyron.suites.push(suite);
        __klyron.currentSuite = null;
    }

    function it(name, fn) {
        if (!__klyron.currentSuite) throw new Error('it() must be called inside describe()');
        const test = { name, fn, assertions: [] };
        __klyron.currentSuite.tests.push(test);
    }

    function test(name, fn) { it(name, fn); }

    function before(fn) { if (__klyron.currentSuite) __klyron.currentSuite.before.push(fn); }
    function after(fn) { if (__klyron.currentSuite) __klyron.currentSuite.after.push(fn); }
    function beforeEach(fn) { if (__klyron.currentSuite) __klyron.currentSuite.beforeEach.push(fn); }
    function afterEach(fn) { if (__klyron.currentSuite) __klyron.currentSuite.afterEach.push(fn); }

    function expect(actual) {
        return {
            toBe(expected) {
                const pass = Object.is(actual, expected);
                if (!pass) throw new Error(`Expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
            },
            not: {
                toBe(expected) {
                    const pass = !Object.is(actual, expected);
                    if (!pass) throw new Error(`Expected not ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
                }
            },
            toEqual(expected) {
                const pass = JSON.stringify(actual) === JSON.stringify(expected);
                if (!pass) throw new Error(`Expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
            },
            toBeNull() { if (actual !== null) throw new Error(`Expected null, got ${JSON.stringify(actual)}`); },
            toBeUndefined() { if (actual !== undefined) throw new Error(`Expected undefined, got ${JSON.stringify(actual)}`); },
            toBeDefined() { if (actual === undefined) throw new Error(`Expected defined, got undefined`); },
            toBeTruthy() { if (!actual) throw new Error(`Expected truthy, got ${JSON.stringify(actual)}`); },
            toBeFalsy() { if (actual) throw new Error(`Expected falsy, got ${JSON.stringify(actual)}`); },
            toBeGreaterThan(n) { if (!(actual > n)) throw new Error(`Expected ${actual} > ${n}`); },
            toBeLessThan(n) { if (!(actual < n)) throw new Error(`Expected ${actual} < ${n}`); },
            toContain(item) {
                if (!Array.isArray(actual) || !actual.includes(item))
                    throw new Error(`Expected array to contain ${JSON.stringify(item)}`);
            },
            toHaveLength(n) {
                if (!actual || actual.length !== n)
                    throw new Error(`Expected length ${n}, got ${actual?.length}`);
            },
            toMatch(regex) {
                if (typeof actual !== 'string' || !regex.test(actual))
                    throw new Error(`Expected "${actual}" to match ${regex}`);
            },
            toThrow() {
                let threw = false;
                if (typeof actual === 'function') {
                    try { actual(); } catch { threw = true; }
                }
                if (!threw) throw new Error('Expected function to throw');
            },
        };
    }

    globalThis.describe = describe;
    globalThis.it = it;
    globalThis.test = test;
    globalThis.expect = expect;
    globalThis.before = before;
    globalThis.after = after;
    globalThis.beforeEach = beforeEach;
    globalThis.afterEach = afterEach;
    globalThis.__klyron = __klyron;
})();
"#
    .to_string()
}

pub fn prepare_js_test(user_code: &str) -> String {
    format!(
        r#"{globals}
try {{
{user_code}
}} catch(e) {{
    if (typeof __klyron !== 'undefined' && __klyron.currentTest) {{
        __klyron.currentTest.assertions.push({{
            passed: false,
            description: e.message || "Unknown error",
            error: e.stack || e.message,
            location: null,
        }});
    }} else {{
        console.error("Unhandled error:", e);
    }}
}}
(function() {{
    const output = JSON.stringify({{ __klyron_suites: __klyron ? __klyron.suites : [] }});
    console.log(output);
}})();
"#,
        globals = generate_assertion_globals()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prepare_js_test_basic() {
        let code = "describe('suite', () => { it('passes', () => { expect(1).toBe(1); }); });";
        let wrapped = prepare_js_test(code);
        assert!(wrapped.contains("function describe"));
        assert!(wrapped.contains("function it"));
        assert!(wrapped.contains("function expect"));
    }

    #[test]
    fn test_generate_assertion_globals_contains_globals() {
        let globals = generate_assertion_globals();
        assert!(globals.contains("globalThis.describe"));
        assert!(globals.contains("globalThis.it"));
        assert!(globals.contains("globalThis.expect"));
    }

    #[test]
    fn test_js_test_results_summary() {
        let results = JsTestResults {
            suites: vec![],
            passed: 5,
            failed: 1,
            total: 6,
            duration_ms: 1234,
        };
        let summary = results.summary();
        assert!(summary.contains("5 passed"));
        assert!(summary.contains("1 failed"));
    }
}
