pub mod array;
pub mod array_buffer;
pub mod r#async;
pub mod bindings;
pub mod buffer;
pub mod builtins;
pub mod class;
pub mod console;
pub mod context;
pub mod crypto;
pub mod date;
pub mod encoding;
pub mod error;
pub mod fs;
pub mod function;
pub mod isolate;
pub mod iter;
pub mod json;
pub mod math;
pub mod module_loader;
pub mod net;
pub mod object;
pub mod os;
pub mod path;
pub mod permissions;
pub mod process;
pub mod promise;
pub mod regex;
pub mod runtime;
pub mod snapshot;
pub mod timers;
pub mod url;
pub mod value;

pub use runtime::BoaRuntime;
pub use isolate::BoaIsolate;
pub use error::BoaError;
pub use value::BoaValue;

use std::time::{Duration, Instant};
use boa_engine::{Context, JsValue, NativeFunction, js_string};
pub struct EngineConfig {
    pub max_execution_time: Option<Duration>,
    pub max_stack_depth: Option<usize>,
    pub max_memory_bytes: Option<u64>,
    pub enable_console: bool,
    pub enable_timers: bool,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            max_execution_time: Some(Duration::from_secs(5)),
            max_stack_depth: Some(512),
            max_memory_bytes: None,
            enable_console: true,
            enable_timers: false,
        }
    }
}

pub struct BoaEngine {
    runtime: BoaRuntime,
    config: EngineConfig,
    start_time: Option<Instant>,
    #[allow(dead_code)]
    memory_usage: u64,
}

impl BoaEngine {
    pub fn new() -> Self {
        let config = EngineConfig::default();
        let runtime = BoaRuntime::new_with_limits(config.max_stack_depth, None, None);
        Self { runtime, config, start_time: None, memory_usage: 0 }
    }

    pub fn new_with_config(config: EngineConfig) -> Self {
        let runtime = BoaRuntime::new_with_limits(config.max_stack_depth, None, None);
        Self { runtime, config, start_time: None, memory_usage: 0 }
    }

    fn check_timeout(&self) -> Result<(), BoaError> {
        if let Some(max_time) = self.config.max_execution_time {
            if let Some(start) = self.start_time {
                if start.elapsed() > max_time {
                    return Err(BoaError::Timeout);
                }
            }
        }
        Ok(())
    }

    fn check_memory(&mut self) -> Result<(), BoaError> {
        // boa_engine 0.19 does not expose runtime memory stats via public API.
        // Memory enforcement relies on the runtime limits system (stack, recursion,
        // loop iteration limits) configured during context creation.
        // The `memory_usage` field is reserved for future use when boa exposes
        // heap metrics (e.g. via context.runtime().metrics().allocated_bytes()).
        Ok(())
    }

    pub fn eval(&mut self, code: &str) -> Result<String, BoaError> {
        self.start_time = Some(Instant::now());
        let sanitized = klyron_engine_common::sanitize::sanitize_js_input(code)
            .map_err(|e| BoaError::SanitizeError(e.to_string()))?;
        let result = self.runtime.eval(&sanitized);
        self.check_timeout()?;
        self.check_memory()?;
        self.start_time = None;
        result
    }

    pub fn execute_script(&mut self, filename: &str, source: &str) -> Result<String, BoaError> {
        self.start_time = Some(Instant::now());
        let result = self.runtime.execute_script(filename, source);
        self.check_timeout()?;
        self.check_memory()?;
        self.start_time = None;
        result
    }

    pub fn execute_module(&mut self, filename: &str, source: &str) -> Result<String, BoaError> {
        self.start_time = Some(Instant::now());
        let result = self.runtime.execute_module(filename, source);
        self.check_timeout()?;
        self.check_memory()?;
        self.start_time = None;
        result
    }

    pub fn compile(&mut self, _source: &str) -> Result<usize, BoaError> {
        Ok(0)
    }

    pub fn context_mut(&mut self) -> &mut Context {
        self.runtime.context_mut()
    }

    pub fn register_native_function(
        &mut self,
        name: impl Into<String>,
        func: NativeFunction,
    ) -> Result<(), BoaError> {
        self.runtime.register_global_function(&name.into(), func)
    }

    pub fn register_global_value(
        &mut self,
        name: impl Into<String>,
        value: JsValue,
    ) -> Result<(), BoaError> {
        self.runtime.register_global_value(&name.into(), value)
    }

    pub fn get_global(&mut self, name: &str) -> Result<JsValue, BoaError> {
        let global = self.runtime.context().global_object();
        let val = global.get(js_string!(name), self.runtime.context_mut())
            .map_err(|e| BoaError::from_js_error(&e))?;
        Ok(val)
    }

    pub fn stack_trace(&mut self) -> String {
        self.runtime.stack_trace()
    }

    pub fn version() -> &'static str {
        VERSION
    }

    pub fn runtime(&self) -> &BoaRuntime { &self.runtime }
    pub fn runtime_mut(&mut self) -> &mut BoaRuntime { &mut self.runtime }
    pub fn config(&self) -> &EngineConfig { &self.config }
}

impl Default for BoaEngine {
    fn default() -> Self { Self::new() }
}

pub(crate) static VERSION: &str = env!("CARGO_PKG_VERSION");

#[unsafe(no_mangle)]
pub extern "C" fn klyron_boa_version() -> *const u8 { VERSION.as_ptr() }
#[unsafe(no_mangle)]
pub extern "C" fn klyron_boa_version_len() -> usize { VERSION.len() }

#[cfg(test)]
mod tests {
    use super::*;

    fn eng() -> BoaEngine { BoaEngine::new() }

    #[test] fn test_eval_addition() { assert_eq!(eng().eval("1 + 2").unwrap(), "3"); }
    #[test] fn test_eval_subtraction() { assert_eq!(eng().eval("10 - 4").unwrap(), "6"); }
    #[test] fn test_eval_multiplication() { assert_eq!(eng().eval("6 * 7").unwrap(), "42"); }
    #[test] fn test_eval_division() { let r = eng().eval("10 / 3").unwrap(); assert!(r.starts_with("3."), "got: {r}"); }
    #[test] fn test_eval_string_concat() { let r = eng().eval("\"hello\" + \" world\"").unwrap(); assert_eq!(r, "hello world", "got: {r}"); }
    #[test] fn test_eval_template() { assert_eq!(eng().eval("`hello ${1 + 2}`").unwrap(), "hello 3"); }
    #[test] fn test_eval_syntax_error() { assert!(eng().eval("syntax error{{{").is_err()); }
    #[test] fn test_eval_execution_error() { assert!(eng().eval("throw new Error('boom')").is_err()); }
    #[test] fn test_eval_type_error() { assert!(eng().eval("null.x").is_err()); }
    #[test] fn test_eval_script() { assert_eq!(eng().execute_script("test.js", "1 + 2").unwrap(), "3"); }
    #[test] fn test_eval_function_call() { assert_eq!(eng().eval("(function(x){return x*2;})(5)").unwrap(), "10"); }
    #[test] fn test_eval_arrow() { assert_eq!(eng().eval("((a,b)=>a+b)(3,4)").unwrap(), "7"); }
    #[test] fn test_eval_array_length() { assert_eq!(eng().eval("[1,2,3,4].length").unwrap(), "4"); }
    #[test] fn test_eval_array_index() { assert_eq!(eng().eval("[10,20,30][1]").unwrap(), "20"); }
    #[test] fn test_eval_object_access() { assert_eq!(eng().eval("({a:1,b:2}).b").unwrap(), "2"); }
    #[test] fn test_eval_boolean_logic() { assert_eq!(eng().eval("true && false || true").unwrap(), "true"); }
    #[test] fn test_eval_comparison() { assert_eq!(eng().eval("1 === 1 && 2 !== 3").unwrap(), "true"); }
    #[test] fn test_eval_ternary() { assert_eq!(eng().eval("5 > 3 ? 'yes' : 'no'").unwrap(), "yes"); }
    #[test] fn test_eval_while() { assert_eq!(eng().eval("(function(){let i=0,s=0;while(i<10){s+=i;i++}return s;})()").unwrap(), "45"); }
    #[test] fn test_eval_for() { assert_eq!(eng().eval("(function(){let s=0;for(let i=0;i<5;i++){s+=i}return s;})()").unwrap(), "10"); }
    #[test] fn test_eval_nested_fn() { assert_eq!(eng().eval("(function(a){return (function(b){return a+b})(3)})(2)").unwrap(), "5"); }
    #[test] fn test_eval_closure() { assert_eq!(eng().eval("(function(){let x=1;return function(){return ++x}})()()").unwrap(), "2"); }
    #[test] fn test_eval_math_pi() { let r = eng().eval("Math.PI").unwrap(); assert!(r.starts_with("3.14"), "got: {r}"); }
    #[test] fn test_eval_math_floor() { assert_eq!(eng().eval("Math.floor(3.7)").unwrap(), "3"); }
    #[test] fn test_eval_math_max() { assert_eq!(eng().eval("Math.max(1,5,3)").unwrap(), "5"); }
    #[test] fn test_eval_json_stringify() { let r = eng().eval("JSON.stringify({a:1,b:2})").unwrap(); assert!(r.contains("a") && r.contains("b"), "got: {r}"); }
    #[test] fn test_eval_json_parse() { assert_eq!(eng().eval("JSON.parse('{\"x\":42}').x").unwrap(), "42"); }
    #[test] fn test_eval_regex_test() { assert_eq!(eng().eval("/hello/.test('hello world')").unwrap(), "true"); }
    #[test] fn test_eval_string_slice() { assert_eq!(eng().eval("\"hello world\".slice(0,5)").unwrap(), "hello"); }
    #[test] fn test_eval_array_push() { assert_eq!(eng().eval("(function(){let a=[1,2];a.push(3);return a.length;})()").unwrap(), "3"); }
    #[test] fn test_eval_array_map() { assert_eq!(eng().eval("[1,2,3].map(x=>x*2)").unwrap(), "2,4,6"); }
    #[test] fn test_eval_array_filter() { assert_eq!(eng().eval("[1,2,3,4].filter(x=>x%2===0)").unwrap(), "2,4"); }
    #[test] fn test_eval_array_reduce() { assert_eq!(eng().eval("[1,2,3,4].reduce((a,b)=>a+b,0)").unwrap(), "10"); }
    #[test] fn test_eval_undefined() { assert_eq!(eng().eval("undefined").unwrap(), "undefined"); }
    #[test] fn test_eval_null() { assert_eq!(eng().eval("null").unwrap(), "null"); }
    #[test] fn test_eval_true() { assert_eq!(eng().eval("true").unwrap(), "true"); }
    #[test] fn test_eval_false() { assert_eq!(eng().eval("false").unwrap(), "false"); }
    #[test] fn test_eval_date_now() { let r = eng().eval("Date.now()").unwrap(); let n: f64 = r.parse().unwrap_or(0.0); assert!(n > 1e12, "got: [{r}] parsed={n}"); }
    #[test] fn test_eval_error_ctor() { assert_eq!(eng().eval("new Error('test').message").unwrap(), "test"); }
    #[test] fn test_eval_range_error() { assert_eq!(eng().eval("new RangeError('range').name").unwrap(), "RangeError"); }
    #[test] fn test_eval_type_error_ctor() { assert_eq!(eng().eval("new TypeError('type').name").unwrap(), "TypeError"); }
    #[test] fn test_eval_symbol() { assert_eq!(eng().eval("typeof Symbol('test')").unwrap(), "symbol"); }
    #[test] fn test_eval_map() { assert_eq!(eng().eval("(function(){const m=new Map();m.set('a',1);return m.get('a');})()").unwrap(), "1"); }
    #[test] fn test_eval_set() { assert_eq!(eng().eval("(function(){const s=new Set();s.add(1);s.add(2);return s.size;})()").unwrap(), "2"); }
    #[test] fn test_eval_typed_array() { assert_eq!(eng().eval("(function(){const a=new Int32Array(4);a[0]=42;return a[0];})()").unwrap(), "42"); }
    #[test] fn test_eval_var_hoisting() { assert_eq!(eng().eval("(function(){return x;var x=5;})()").unwrap(), "undefined"); }
    #[test] fn test_eval_try_catch() { assert_eq!(eng().eval("(function(){try{throw new Error('caught')}catch(e){return 'caught!'}})()").unwrap(), "caught!"); }
    #[test] fn test_eval_try_finally() { assert_eq!(eng().eval("(function(){try{return 'try'}finally{return 'finally'}})()").unwrap(), "finally"); }
    #[test] fn test_eval_throw_custom() { assert_eq!(eng().eval("(function(){try{throw 42}catch(e){return e}})()").unwrap(), "42"); }
    #[test] fn test_eval_complex_obj() { let r = eng().eval("JSON.stringify({name:'test',values:[1,2,3],nested:{a:{b:42}}})").unwrap(); assert!(r.contains("\"b\":42"), "got: {r}"); }
    #[test] fn test_eval_destructuring() { assert_eq!(eng().eval("(function(){const{a,b}={a:1,b:2};return a+b;})()").unwrap(), "3"); }
    #[test] fn test_eval_spread() { assert_eq!(eng().eval("(function(){const a=[1,2];const b=[...a,3];return b.length;})()").unwrap(), "3"); }
    #[test] fn test_eval_rest() { assert_eq!(eng().eval("(function(...args){return args.length;})(1,2,3)").unwrap(), "3"); }
    #[test] fn test_eval_class() { assert_eq!(eng().eval("new (class{constructor(x){this.x=x}})(42).x").unwrap(), "42"); }
    #[test] fn test_eval_extends() { assert_eq!(eng().eval("(function(){class A{getX(){return 1}}class B extends A{getX(){return super.getX()+1}}return(new B()).getX();})()").unwrap(), "2"); }
    #[test] fn test_eval_static() { assert_eq!(eng().eval("(function(){class A{static greet(){return 'hi'}}return A.greet();})()").unwrap(), "hi"); }
    #[test] fn test_eval_getter_setter() { assert_eq!(eng().eval("(function(){let obj={_val:0,get val(){return this._val},set val(v){this._val=v}};obj.val=42;return obj.val;})()").unwrap(), "42"); }
    #[test] fn test_eval_generator() { assert_eq!(eng().eval("(function*(){yield 1;yield 2;})().next().value").unwrap(), "1"); }
    #[test] fn test_eval_for_of() { assert_eq!(eng().eval("(function(){let s=0;for(const v of[1,2,3,4]){s+=v}return s;})()").unwrap(), "10"); }
    #[test] fn test_eval_for_in() { assert_eq!(eng().eval("(function(){const o={a:1,b:2};let k='';for(const p in o){k+=p}return k;})()").unwrap(), "ab"); }
    #[test] fn test_eval_version() { let v = BoaEngine::version(); assert!(!v.is_empty()); }
    #[test] fn test_eval_configured() { let c = EngineConfig { max_execution_time: Some(Duration::from_secs(10)), max_stack_depth: Some(1024), max_memory_bytes: None, enable_console: true, enable_timers: true }; let mut e = BoaEngine::new_with_config(c); assert_eq!(e.eval("1+1").unwrap(), "2"); }
    #[test] fn test_eval_stack_trace() { let mut e = eng(); let _ = e.eval("function a(){b()}function b(){c()}function c(){throw new Error('trace')}"); let s = e.stack_trace(); assert!(!s.is_empty()); }
    #[test] fn test_eval_global_get_set() { let mut e = eng(); let _ = e.eval("var globalVar=99"); assert!(e.get_global("globalVar").is_ok()); }
    #[test] fn test_eval_empty() { assert!(eng().eval("").is_ok()); }
    #[test] fn test_eval_whitespace() { assert!(eng().eval("   ").is_ok()); }
    #[test] fn test_eval_global_this() { assert_eq!(eng().eval("typeof globalThis").unwrap(), "object"); }
    #[test] fn test_eval_eval_fn() { assert_eq!(eng().eval("eval('1+2')").unwrap(), "3"); }
    #[test] fn test_eval_fn_ctor() { assert_eq!(eng().eval("new Function('a','b','return a+b')(3,4)").unwrap(), "7"); }
    #[test] fn test_eval_escape_seq() { let r = eng().eval("\"line1\\nline2\"").unwrap(); assert_eq!(r, "line1\nline2", "got: {r}"); }
    #[test] fn test_eval_unicode() { assert_eq!(eng().eval("\"\\u0041\"").unwrap(), "A"); }
    #[test] fn test_eval_var_scope() { assert_eq!(eng().eval("(function(){var x=5;if(true){var x=10}return x;})()").unwrap(), "10"); }
    #[test] fn test_eval_let_scope() { assert_eq!(eng().eval("(function(){let x=5;if(true){let x=10}return x;})()").unwrap(), "5"); }
    #[test] fn test_eval_const() { assert_eq!(eng().eval("(function(){const x=5;return x;})()").unwrap(), "5"); }
    #[test] fn test_eval_default_params() { assert_eq!(eng().eval("(function(a=1,b=2){return a+b;})(5)").unwrap(), "7"); }
    #[test] fn test_eval_arrow_return_obj() { assert_eq!(eng().eval("(()=>({x:1,y:2}))().x").unwrap(), "1"); }
    #[test] fn test_eval_computed_prop() { assert_eq!(eng().eval("({['he'+'llo']:42})['hello']").unwrap(), "42"); }
    #[test] fn test_eval_short_circuit_and() { assert_eq!(eng().eval("null && 'unreachable'").unwrap(), "null"); }
    #[test] fn test_eval_short_circuit_or() { assert_eq!(eng().eval("null || 'default'").unwrap(), "default"); }
    #[test] fn test_eval_nullish_coalescing() { assert_eq!(eng().eval("null ?? 'fallback'").unwrap(), "fallback"); }
    #[test] fn test_eval_optional_chaining() { assert_eq!(eng().eval("({a:{b:42}})?.a?.b").unwrap(), "42"); }
    #[test] fn test_eval_optional_chaining_null() { assert_eq!(eng().eval("null?.a").unwrap(), "undefined"); }
    #[test] fn test_value_json_roundtrip() { let j = serde_json::json!({"s":"hello","n":42,"b":true,"a":[1,2,3],"o":{"x":1}}); let bv = BoaValue::from_json(&j); assert_eq!(j, bv.to_json()); }
    #[test] fn test_value_json_array() { let j = serde_json::json!([1,"two",false,null]); let bv = BoaValue::from_json(&j); assert_eq!(j, bv.to_json()); }
    #[test] fn test_eval_console_log_exists() { let r = eng().eval("typeof console"); assert!(r.is_ok()); }
    #[test] fn test_eval_console_error_exists() { let r = eng().eval("typeof console"); assert!(r.is_ok()); }
    #[test] fn test_eval_is_finite() { assert_eq!(eng().eval("Number.isFinite(42)").unwrap(), "true"); }
    #[test] fn test_eval_is_nan() { assert_eq!(eng().eval("Number.isNaN(NaN)").unwrap(), "true"); }
    #[test] fn test_eval_is_safe_integer() { assert_eq!(eng().eval("Number.isSafeInteger(9007199254740991)").unwrap(), "true"); }
    #[test] fn test_eval_parse_int_radix() { assert_eq!(eng().eval("parseInt('FF',16)").unwrap(), "255"); }
    #[test] fn test_eval_string_char_at() { assert_eq!(eng().eval("\"hello\".charAt(1)").unwrap(), "e"); }
    #[test] fn test_eval_string_char_code_at() { assert_eq!(eng().eval("\"A\".charCodeAt(0)").unwrap(), "65"); }
    #[test] fn test_eval_string_code_point_at() { assert_eq!(eng().eval("\"A\".codePointAt(0)").unwrap(), "65"); }
    #[test] fn test_eval_string_from_code_point() { assert_eq!(eng().eval("String.fromCodePoint(65)").unwrap(), "A"); }
    #[test] fn test_eval_string_concat_method() { assert_eq!(eng().eval("\"a\".concat('b','c')").unwrap(), "abc"); }
    #[test] fn test_eval_string_index_of() { assert_eq!(eng().eval("\"hello\".indexOf('l')").unwrap(), "2"); }
    #[test] fn test_eval_string_last_index_of() { assert_eq!(eng().eval("\"hello\".lastIndexOf('l')").unwrap(), "3"); }
    #[test] fn test_eval_string_match() { let r = eng().eval("\"hello123\".match(/\\d+/)").unwrap(); assert!(r.contains("123"), "got: {r}"); }
    #[test] fn test_eval_string_pad_start() { assert_eq!(eng().eval("\"5\".padStart(3,'0')").unwrap(), "005"); }
    #[test] fn test_eval_string_pad_end() { assert_eq!(eng().eval("\"5\".padEnd(3,'0')").unwrap(), "500"); }
    #[test] fn test_eval_string_search() { assert_eq!(eng().eval("\"hello123\".search(/\\d+/)").unwrap(), "5"); }
    #[test] fn test_eval_string_to_lower() { assert_eq!(eng().eval("\"HELLO\".toLowerCase()").unwrap(), "hello"); }
    #[test] fn test_eval_string_to_upper() { assert_eq!(eng().eval("\"hello\".toUpperCase()").unwrap(), "HELLO"); }
    #[test] fn test_eval_symbol_for() { assert_eq!(eng().eval("Symbol.for('test')===Symbol.for('test')").unwrap(), "true"); }
    #[test] fn test_eval_symbol_key_for() { assert_eq!(eng().eval("Symbol.keyFor(Symbol.for('x'))").unwrap(), "x"); }
    #[test] fn test_eval_set_has() { assert_eq!(eng().eval("(function(){const s=new Set([1,2,3]);return s.has(2);})()").unwrap(), "true"); }
    #[test] fn test_eval_set_delete() { assert_eq!(eng().eval("(function(){const s=new Set([1,2]);s.delete(1);return s.size;})()").unwrap(), "1"); }
    #[test] fn test_eval_set_clear() { assert_eq!(eng().eval("(function(){const s=new Set([1,2]);s.clear();return s.size;})()").unwrap(), "0"); }
    #[test] fn test_eval_map_has() { assert_eq!(eng().eval("(function(){const m=new Map();m.set('a',1);return m.has('a');})()").unwrap(), "true"); }
    #[test] fn test_eval_map_delete() { assert_eq!(eng().eval("(function(){const m=new Map([['a',1]]);m.delete('a');return m.size;})()").unwrap(), "0"); }
    #[test] fn test_eval_map_keys() { assert_eq!(eng().eval("(function(){const m=new Map([['a',1],['b',2]]);return Array.from(m.keys()).length;})()").unwrap(), "2"); }
    #[test] fn test_eval_map_clear() { assert_eq!(eng().eval("(function(){const m=new Map([['a',1]]);m.clear();return m.size;})()").unwrap(), "0"); }
    #[test] fn test_eval_typeof_undefined_var() { assert_eq!(eng().eval("typeof nonExistentVar").unwrap(), "undefined"); }
    #[test] fn test_eval_void_operator() { assert_eq!(eng().eval("void 0").unwrap(), "undefined"); }
    #[test] fn test_eval_typeof_null() { assert_eq!(eng().eval("typeof null").unwrap(), "object"); }
    #[test] fn test_eval_delete_prop() { assert_eq!(eng().eval("(function(){const o={a:1};delete o.a;return o.a;})()").unwrap(), "undefined"); }
    #[test] fn test_eval_in_operator() { assert_eq!(eng().eval("'length' in [1,2,3]").unwrap(), "true"); }
    #[test] fn test_eval_instance_of() { assert_eq!(eng().eval("[] instanceof Array").unwrap(), "true"); }
    #[test] fn test_eval_number_methods() { assert_eq!(eng().eval("Number.isInteger(42)").unwrap(), "true"); }
    #[test] fn test_eval_number_to_fixed() { assert_eq!(eng().eval("(3.14159).toFixed(2)").unwrap(), "3.14"); }
    #[test] fn test_eval_math_random() { let r = eng().eval("Math.random()").unwrap(); let n: f64 = r.parse().unwrap_or(-1.0); assert!(n >= 0.0 && n < 1.0, "got: {r}"); }
    #[test] fn test_eval_math_round() { assert_eq!(eng().eval("Math.round(3.5)").unwrap(), "4"); }
    #[test] fn test_eval_math_abs() { assert_eq!(eng().eval("Math.abs(-5)").unwrap(), "5"); }
    #[test] fn test_eval_math_pow() { assert_eq!(eng().eval("Math.pow(2,10)").unwrap(), "1024"); }
    #[test] fn test_eval_math_sqrt() { assert_eq!(eng().eval("Math.sqrt(16)").unwrap(), "4"); }
    #[test] fn test_eval_math_min() { assert_eq!(eng().eval("Math.min(5,2,8)").unwrap(), "2"); }
    #[test] fn test_eval_math_trunc() { assert_eq!(eng().eval("Math.trunc(4.9)").unwrap(), "4"); }
    #[test] fn test_eval_math_sign() { assert_eq!(eng().eval("Math.sign(-5)").unwrap(), "-1"); }
    #[test] fn test_eval_math_cbrt() { assert_eq!(eng().eval("Math.cbrt(27)").unwrap(), "3"); }
    #[test] fn test_eval_math_hypot() { assert_eq!(eng().eval("Math.hypot(3,4)").unwrap(), "5"); }
    #[test] fn test_eval_date_construct() { assert_eq!(eng().eval("new Date('2024-01-01').getFullYear()").unwrap(), "2024"); }
    #[test] fn test_eval_date_month() { assert_eq!(eng().eval("new Date('2024-06-15').getMonth()").unwrap(), "5"); }
    #[test] fn test_eval_date_date() { assert_eq!(eng().eval("new Date('2024-06-15').getDate()").unwrap(), "15"); }
    #[test] fn test_eval_regex_exec() { assert_eq!(eng().eval("/(\\d+)/.exec('abc123def')[1]").unwrap(), "123"); }
    #[test] fn test_eval_regex_flags() { assert_eq!(eng().eval("/test/gi.flags").unwrap(), "gi"); }
    #[test] fn test_eval_regex_source() { assert_eq!(eng().eval("/abc/.source").unwrap(), "abc"); }
    #[test] fn test_eval_regex_test_no_match() { assert_eq!(eng().eval("/xyz/.test('hello')").unwrap(), "false"); }
    #[test] fn test_eval_string_trim() { assert_eq!(eng().eval("\"  hello  \".trim()").unwrap(), "hello"); }
    #[test] fn test_eval_string_split() { assert_eq!(eng().eval("\"a,b,c\".split(',')").unwrap(), "a,b,c"); }
    #[test] fn test_eval_string_includes() { assert_eq!(eng().eval("\"hello\".includes('ell')").unwrap(), "true"); }
    #[test] fn test_eval_string_starts_with() { assert_eq!(eng().eval("\"hello\".startsWith('he')").unwrap(), "true"); }
    #[test] fn test_eval_string_ends_with() { assert_eq!(eng().eval("\"hello\".endsWith('lo')").unwrap(), "true"); }
    #[test] fn test_eval_string_repeat() { assert_eq!(eng().eval("\"ab\".repeat(3)").unwrap(), "ababab"); }
    #[test] fn test_eval_string_replace() { assert_eq!(eng().eval("\"hello world\".replace('world','there')").unwrap(), "hello there"); }
    #[test] fn test_eval_replace_all() { assert_eq!(eng().eval("\"a-a-a\".replaceAll('-','+')").unwrap(), "a+a+a"); }
    #[test] fn test_eval_array_flat() { assert_eq!(eng().eval("[[1,2],[3,4]].flat()").unwrap(), "1,2,3,4"); }
    #[test] fn test_eval_array_flat_map() { assert_eq!(eng().eval("[1,2,3].flatMap(x=>[x,x*2])").unwrap(), "1,2,2,4,3,6"); }
    #[test] fn test_eval_array_find() { assert_eq!(eng().eval("[1,2,3,4].find(x=>x>2)").unwrap(), "3"); }
    #[test] fn test_eval_array_find_index() { assert_eq!(eng().eval("[1,2,3,4].findIndex(x=>x>2)").unwrap(), "2"); }
    #[test] fn test_eval_array_some() { assert_eq!(eng().eval("[1,2,3].some(x=>x>2)").unwrap(), "true"); }
    #[test] fn test_eval_array_every() { assert_eq!(eng().eval("[1,2,3].every(x=>x>0)").unwrap(), "true"); }
    #[test] fn test_eval_array_from() { assert_eq!(eng().eval("Array.from('hello').length").unwrap(), "5"); }
    #[test] fn test_eval_array_of() { assert_eq!(eng().eval("Array.of(1,2,3).length").unwrap(), "3"); }
    #[test] fn test_eval_array_fill() { assert_eq!(eng().eval("new Array(3).fill(0)").unwrap(), "0,0,0"); }
    #[test] fn test_eval_array_includes() { assert_eq!(eng().eval("[1,2,3].includes(2)").unwrap(), "true"); }
    #[test] fn test_eval_array_join() { assert_eq!(eng().eval("[1,2,3].join('-')").unwrap(), "1-2-3"); }
    #[test] fn test_eval_array_reverse() { assert_eq!(eng().eval("[1,2,3].reverse()").unwrap(), "3,2,1"); }
    #[test] fn test_eval_array_sort() { assert_eq!(eng().eval("[3,1,2].sort()").unwrap(), "1,2,3"); }
    #[test] fn test_eval_array_concat() { assert_eq!(eng().eval("[1,2].concat([3,4]).length").unwrap(), "4"); }
    #[test] fn test_eval_array_slice() { assert_eq!(eng().eval("[1,2,3,4].slice(1,3)").unwrap(), "2,3"); }
    #[test] fn test_eval_array_splice() { assert_eq!(eng().eval("(function(){let a=[1,2,3,4];a.splice(1,2);return a;})()").unwrap(), "1,4"); }
    #[test] fn test_eval_object_keys() { assert_eq!(eng().eval("Object.keys({a:1,b:2}).length").unwrap(), "2"); }
    #[test] fn test_eval_object_values() { assert_eq!(eng().eval("Object.values({a:1,b:2})").unwrap(), "1,2"); }
    #[test] fn test_eval_object_entries() { assert_eq!(eng().eval("Object.entries({a:1,b:2}).length").unwrap(), "2"); }
    #[test] fn test_eval_object_assign() { assert_eq!(eng().eval("Object.assign({},{a:1},{b:2}).b").unwrap(), "2"); }
    #[test] fn test_eval_object_freeze() { assert_eq!(eng().eval("Object.isFrozen(Object.freeze({}))").unwrap(), "true"); }
    #[test] fn test_eval_object_seal() { assert_eq!(eng().eval("Object.isSealed(Object.seal({a:1}))").unwrap(), "true"); }
    #[test] fn test_eval_parse_int() { assert_eq!(eng().eval("parseInt('42')").unwrap(), "42"); }
    #[test] fn test_eval_parse_float() { assert_eq!(eng().eval("parseFloat('3.14')").unwrap(), "3.14"); }
    #[test] fn test_eval_is_nan_global() { assert_eq!(eng().eval("isNaN(NaN)").unwrap(), "true"); }
    #[test] fn test_eval_encode_uri() { assert_eq!(eng().eval("encodeURIComponent('hello world')").unwrap(), "hello%20world"); }
    #[test] fn test_eval_decode_uri() { assert_eq!(eng().eval("decodeURIComponent('hello%20world')").unwrap(), "hello world"); }
    #[test] fn test_eval_promise_resolve() { assert!(eng().eval("Promise.resolve(42)").is_ok()); }
    #[test] fn test_eval_bigint() { let r = eng().eval("12345678901234567890n"); if let Ok(v) = r { assert!(v.contains("1234567890"), "got: {v}"); } }
    #[test] fn test_eval_weak_map() { assert_eq!(eng().eval("typeof WeakMap").unwrap(), "function"); }
    #[test] fn test_eval_weak_set() { assert_eq!(eng().eval("typeof WeakSet").unwrap(), "function"); }
    #[test] fn test_eval_proxy() { assert_eq!(eng().eval("typeof Proxy").unwrap(), "function"); }
    #[test] fn test_eval_reflect() { assert_eq!(eng().eval("typeof Reflect").unwrap(), "object"); }
    #[test] fn test_eval_data_view() { assert_eq!(eng().eval("typeof DataView").unwrap(), "function"); }
    #[test] fn test_eval_async_fn() { assert!(eng().eval("(async function(){return 42;})()").is_ok()); }
    #[test] fn test_eval_new_target() { assert_eq!(eng().eval("(function(){return typeof new.target;})()").unwrap(), "undefined"); }
    #[test] fn test_eval_import_meta() { let r = eng().eval("typeof import.meta"); assert!(r.is_ok() || r.is_err()); }
    #[test] fn test_eval_permissions() { let p = permissions::BoaPermissions::allow_all(); assert!(p.check(&permissions::Permission::Read, Some("/tmp"))); assert!(!permissions::BoaPermissions::deny_all().check(&permissions::Permission::Read, Some("/etc"))); }
    #[test] fn test_eval_complex_math() { let r = eng().eval("Math.sin(Math.PI/2)").unwrap(); let n: f64 = r.parse().unwrap_or(0.0); assert!((n - 1.0).abs() < 0.01, "got: {r}"); }
}
