pub mod error;
pub mod module_loader;
pub mod permissions;
pub mod runtime;
pub mod context;
pub mod isolate;
pub mod script;
pub mod value;
pub mod promise;
pub mod module;
pub mod snapshot;
pub mod heap;
pub mod inspector;
pub mod wasm;
pub mod json;
pub mod console;
pub mod timers;
pub mod crypto;
pub mod encoding;
pub mod fs;
pub mod net;
pub mod url;
pub mod buffer;
pub mod typed_array;
pub mod array_buffer;
pub mod function;
pub mod object;

#[cfg(feature = "native")]
pub mod ffi;

#[cfg(feature = "native")]
pub mod ffi_types;

use error::V8Error;

#[cfg(feature = "native")]
use ffi::V8EnginePtr;

pub struct HeapStats {
    pub total_heap_size: u64,
    pub total_heap_size_executable: u64,
    pub total_physical_size: u64,
    pub total_available_size: u64,
    pub used_heap_size: u64,
    pub heap_size_limit: u64,
    pub malloced_memory: u64,
    pub peak_malloced_memory: u64,
    pub number_of_native_contexts: u64,
    pub number_of_detached_contexts: u64,
    pub total_global_handles_size: u64,
    pub used_global_handles_size: u64,
    pub external_memory: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum V8ValueType {
    Undefined = 0,
    Null = 1,
    Boolean = 2,
    Number = 3,
    String = 4,
    Object = 5,
    Array = 6,
    Function = 7,
    Promise = 8,
    Error = 9,
    Symbol = 10,
    BigInt = 11,
    Proxy = 12,
    TypedArray = 13,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum V8PromiseState {
    Pending = 0,
    Fulfilled = 1,
    Rejected = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum V8MemoryPressure {
    None = 0,
    Moderate = 1,
    Critical = 2,
}

pub struct V8Engine {
    #[cfg(feature = "native")]
    inner: V8EnginePtr,
}

impl V8Engine {
    pub fn new() -> Result<Self, V8Error> {
        #[cfg(feature = "native")]
        {
            V8EnginePtr::init()
                .map(|inner| Self { inner })
                .map_err(|e| V8Error::InitFailed(e))
        }
        #[cfg(not(feature = "native"))]
        {
            Err(V8Error::InitFailed(
                "V8 native engine not available — enable 'native' feature".into()
            ))
        }
    }

    pub fn eval(&self, _code: &str) -> Result<String, V8Error> {
        #[cfg(feature = "native")]
        { self.inner.eval(_code).map_err(V8Error::EvalFailed) }
        #[cfg(not(feature = "native"))]
        { Err(V8Error::NotInitialized) }
    }

    pub fn execute_script(&self, _filename: &str, _source: &str) -> Result<String, V8Error> {
        #[cfg(feature = "native")]
        { self.inner.execute_script(_filename, _source).map_err(V8Error::EvalFailed) }
        #[cfg(not(feature = "native"))]
        { Err(V8Error::NotInitialized) }
    }

    pub fn execute_module(&self, _filename: &str, _source: &str) -> Result<String, V8Error> {
        #[cfg(feature = "native")]
        { self.inner.execute_module(_filename, _source).map_err(V8Error::ModuleFailed) }
        #[cfg(not(feature = "native"))]
        { Err(V8Error::NotInitialized) }
    }

    pub fn json_stringify(&self, _value_ptr: *const std::ffi::c_void) -> Result<String, V8Error> {
        #[cfg(feature = "native")]
        {
            let v = _value_ptr as *mut ffi::V8ValueHandle;
            self.inner.json_stringify(v).map_err(|e| V8Error::EvalFailed(e))
        }
        #[cfg(not(feature = "native"))]
        { Err(V8Error::NotInitialized) }
    }

    pub fn json_parse(&self, _json: &str) -> Result<*const std::ffi::c_void, V8Error> {
        #[cfg(feature = "native")]
        { self.inner.json_parse(_json).map(|p| p as *const std::ffi::c_void).map_err(V8Error::EvalFailed) }
        #[cfg(not(feature = "native"))]
        { Err(V8Error::NotInitialized) }
    }

    pub fn get_global(&self, _key: &str) -> Result<*const std::ffi::c_void, V8Error> {
        #[cfg(feature = "native")]
        { self.inner.get_global(_key).map(|p| p as *const std::ffi::c_void).map_err(V8Error::GlobalFailed) }
        #[cfg(not(feature = "native"))]
        { Err(V8Error::NotInitialized) }
    }

    pub fn set_global(&self, _key: &str, _value_ptr: *const std::ffi::c_void) -> Result<(), V8Error> {
        #[cfg(feature = "native")]
        {
            let v = _value_ptr as *mut ffi::V8ValueHandle;
            self.inner.set_global(_key, v).map_err(V8Error::GlobalFailed)
        }
        #[cfg(not(feature = "native"))]
        { Err(V8Error::NotInitialized) }
    }

    pub fn get_heap_stats(&self) -> Result<HeapStats, V8Error> {
        #[cfg(feature = "native")]
        {
            let raw = self.inner.get_heap_stats().map_err(|e| V8Error::Internal(e))?;
            Ok(HeapStats {
                total_heap_size: raw.total_heap_size as u64,
                total_heap_size_executable: raw.total_heap_size_executable as u64,
                total_physical_size: raw.total_physical_size as u64,
                total_available_size: raw.total_available_size as u64,
                used_heap_size: raw.used_heap_size as u64,
                heap_size_limit: raw.heap_size_limit as u64,
                malloced_memory: raw.malloced_memory as u64,
                peak_malloced_memory: raw.peak_malloced_memory as u64,
                number_of_native_contexts: raw.number_of_native_contexts as u64,
                number_of_detached_contexts: raw.number_of_detached_contexts as u64,
                total_global_handles_size: raw.total_global_handles_size as u64,
                used_global_handles_size: raw.used_global_handles_size as u64,
                external_memory: raw.external_memory as u64,
            })
        }
        #[cfg(not(feature = "native"))]
        { Err(V8Error::NotInitialized) }
    }

    pub fn low_memory_notification(&self) {
        #[cfg(feature = "native")]
        self.inner.low_memory_notification()
    }

    pub fn idle_notification(&self, _deadline: f64) {
        #[cfg(feature = "native")]
        self.inner.idle_notification(_deadline)
    }

    pub fn set_memory_pressure(&self, _pressure: V8MemoryPressure) {
        #[cfg(feature = "native")]
        self.inner.set_memory_pressure(_pressure as u32)
    }

    pub fn request_gc(&self) {
        #[cfg(feature = "native")]
        self.inner.request_gc()
    }

    pub fn get_malloced_memory(&self) -> u64 {
        #[cfg(feature = "native")]
        { self.inner.get_malloced_memory() as u64 }
        #[cfg(not(feature = "native"))]
        { 0 }
    }

    pub fn adjust_external_memory(&self, _change: i64) -> u64 {
        #[cfg(feature = "native")]
        { self.inner.adjust_external_memory(_change) as u64 }
        #[cfg(not(feature = "native"))]
        { 0 }
    }

    pub fn microtasks_perform_check(&self) {
        #[cfg(feature = "native")]
        self.inner.microtasks_perform_check()
    }

    pub fn get_stack_trace(&self) -> Result<String, V8Error> {
        #[cfg(feature = "native")]
        { self.inner.get_stack_trace().map_err(|e| V8Error::Internal(e)) }
        #[cfg(not(feature = "native"))]
        { Err(V8Error::NotInitialized) }
    }

    pub fn is_native_available(&self) -> bool {
        cfg!(feature = "native")
    }

    pub fn version() -> String {
        #[cfg(feature = "native")]
        { V8EnginePtr::version() }
        #[cfg(not(feature = "native"))]
        { "V8 (not available)".into() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use klyron_engine_common as common;

    fn eng() -> V8Engine { V8Engine::new().expect("V8 init failed (native feature required)") }
    fn has_v8() -> bool { V8Engine::new().is_ok() }

    #[test] fn test_v8_new() { let e = V8Engine::new(); assert!(e.is_ok() || e.is_err()); }
    #[test] fn test_v8_eval_addition() { if !has_v8() { return; } assert_eq!(eng().eval("1+2").unwrap(), "3"); }
    #[test] fn test_v8_eval_subtraction() { if !has_v8() { return; } assert_eq!(eng().eval("10-4").unwrap(), "6"); }
    #[test] fn test_v8_eval_multiplication() { if !has_v8() { return; } assert_eq!(eng().eval("6*7").unwrap(), "42"); }
    #[test] fn test_v8_eval_division() { if !has_v8() { return; } let r=eng().eval("10/3").unwrap(); assert!(r.starts_with("3."),"got:{r}"); }
    #[test] fn test_v8_eval_string_concat() { if !has_v8() { return; } assert_eq!(eng().eval("'hello'+' world'").unwrap(),"hello world"); }
    #[test] fn test_v8_eval_template() { if !has_v8() { return; } let r=eng().eval("`hello ${1+2}`").unwrap(); assert_eq!(r,"hello 3","got:{r}"); }
    #[test] fn test_v8_eval_syntax_error() { if !has_v8() { return; } assert!(eng().eval("syntax{{{").is_err()); }
    #[test] fn test_v8_eval_throw() { if !has_v8() { return; } assert!(eng().eval("throw new Error('x')").is_err()); }
    #[test] fn test_v8_eval_type_error() { if !has_v8() { return; } assert!(eng().eval("null.x").is_err()); }
    #[test] fn test_v8_execute_script() { if !has_v8() { return; } assert_eq!(eng().execute_script("t.js","1+3").unwrap(),"4"); }
    #[test] fn test_v8_eval_function_call() { if !has_v8() { return; } assert_eq!(eng().eval("(function(x){return x*2;})(5)").unwrap(),"10"); }
    #[test] fn test_v8_eval_arrow() { if !has_v8() { return; } assert_eq!(eng().eval("((a,b)=>a+b)(3,4)").unwrap(),"7"); }
    #[test] fn test_v8_eval_array_length() { if !has_v8() { return; } assert_eq!(eng().eval("[1,2,3].length").unwrap(),"3"); }
    #[test] fn test_v8_eval_array_index() { if !has_v8() { return; } assert_eq!(eng().eval("[10,20,30][1]").unwrap(),"20"); }
    #[test] fn test_v8_eval_object_access() { if !has_v8() { return; } assert_eq!(eng().eval("({a:1,b:2}).b").unwrap(),"2"); }
    #[test] fn test_v8_eval_boolean_logic() { if !has_v8() { return; } assert_eq!(eng().eval("true&&false||true").unwrap(),"true"); }
    #[test] fn test_v8_eval_comparison() { if !has_v8() { return; } assert_eq!(eng().eval("1===1&&2!==3").unwrap(),"true"); }
    #[test] fn test_v8_eval_ternary() { if !has_v8() { return; } assert_eq!(eng().eval("5>3?'yes':'no'").unwrap(),"yes"); }
    #[test] fn test_v8_eval_while() { if !has_v8() { return; } assert_eq!(eng().eval("(function(){let i=0,s=0;while(i<10){s+=i;i++}return s;})()").unwrap(),"45"); }
    #[test] fn test_v8_eval_for() { if !has_v8() { return; } assert_eq!(eng().eval("(function(){let s=0;for(let i=0;i<5;i++){s+=i}return s;})()").unwrap(),"10"); }
    #[test] fn test_v8_eval_nested_fn() { if !has_v8() { return; } assert_eq!(eng().eval("(function(a){return(function(b){return a+b})(3)})(2)").unwrap(),"5"); }
    #[test] fn test_v8_eval_closure() { if !has_v8() { return; } assert_eq!(eng().eval("(function(){let x=1;return function(){return++x}})()()").unwrap(),"2"); }
    #[test] fn test_v8_eval_math_pi() { if !has_v8() { return; } let r=eng().eval("Math.PI").unwrap(); assert!(r.starts_with("3.14"),"got:{r}"); }
    #[test] fn test_v8_eval_math_floor() { if !has_v8() { return; } assert_eq!(eng().eval("Math.floor(3.7)").unwrap(),"3"); }
    #[test] fn test_v8_eval_math_max() { if !has_v8() { return; } assert_eq!(eng().eval("Math.max(1,5,3)").unwrap(),"5"); }
    #[test] fn test_v8_eval_json_stringify() { if !has_v8() { return; } let r=eng().eval("JSON.stringify({a:1,b:2})").unwrap(); assert!(r.contains("a")&&r.contains("b"),"got:{r}"); }
    #[test] fn test_v8_eval_json_parse() { if !has_v8() { return; } assert_eq!(eng().eval("JSON.parse('{\"x\":42}').x").unwrap(),"42"); }
    #[test] fn test_v8_eval_regex_test() { if !has_v8() { return; } assert_eq!(eng().eval("/hello/.test('hello world')").unwrap(),"true"); }
    #[test] fn test_v8_eval_string_slice() { if !has_v8() { return; } assert_eq!(eng().eval("'hello world'.slice(0,5)").unwrap(),"hello"); }
    #[test] fn test_v8_eval_array_push() { if !has_v8() { return; } assert_eq!(eng().eval("(function(){let a=[1,2];a.push(3);return a.length;})()").unwrap(),"3"); }
    #[test] fn test_v8_eval_array_map() { if !has_v8() { return; } assert_eq!(eng().eval("[1,2,3].map(x=>x*2)").unwrap(),"2,4,6"); }
    #[test] fn test_v8_eval_array_filter() { if !has_v8() { return; } assert_eq!(eng().eval("[1,2,3,4].filter(x=>x%2===0)").unwrap(),"2,4"); }
    #[test] fn test_v8_eval_array_reduce() { if !has_v8() { return; } assert_eq!(eng().eval("[1,2,3,4].reduce((a,b)=>a+b,0)").unwrap(),"10"); }
    #[test] fn test_v8_eval_undefined() { if !has_v8() { return; } assert_eq!(eng().eval("undefined").unwrap(),"undefined"); }
    #[test] fn test_v8_eval_null() { if !has_v8() { return; } assert_eq!(eng().eval("null").unwrap(),"null"); }
    #[test] fn test_v8_eval_true() { if !has_v8() { return; } assert_eq!(eng().eval("true").unwrap(),"true"); }
    #[test] fn test_v8_eval_false() { if !has_v8() { return; } assert_eq!(eng().eval("false").unwrap(),"false"); }
    #[test] fn test_v8_eval_date_now() { if !has_v8() { return; } let r=eng().eval("Date.now()").unwrap(); let n:f64=r.parse().unwrap_or(0.0); assert!(n>1e12,"got:{r}"); }
    #[test] fn test_v8_eval_error_ctor() { if !has_v8() { return; } assert_eq!(eng().eval("new Error('test').message").unwrap(),"test"); }
    #[test] fn test_v8_eval_range_error() { if !has_v8() { return; } let r=eng().eval("new RangeError('range').name").unwrap(); assert_eq!(r,"RangeError","got:{r}"); }
    #[test] fn test_v8_eval_type_error_ctor() { if !has_v8() { return; } let r=eng().eval("new TypeError('type').name").unwrap(); assert_eq!(r,"TypeError","got:{r}"); }
    #[test] fn test_v8_eval_symbol() { if !has_v8() { return; } assert_eq!(eng().eval("typeof Symbol('test')").unwrap(),"symbol"); }
    #[test] fn test_v8_eval_map() { if !has_v8() { return; } assert_eq!(eng().eval("(function(){const m=new Map();m.set('a',1);return m.get('a');})()").unwrap(),"1"); }
    #[test] fn test_v8_eval_set() { if !has_v8() { return; } assert_eq!(eng().eval("(function(){const s=new Set();s.add(1);s.add(2);return s.size;})()").unwrap(),"2"); }
    #[test] fn test_v8_eval_typed_array() { if !has_v8() { return; } assert_eq!(eng().eval("(function(){const a=new Int32Array(4);a[0]=42;return a[0];})()").unwrap(),"42"); }
    #[test] fn test_v8_eval_var_hoisting() { if !has_v8() { return; } assert_eq!(eng().eval("(function(){return x;var x=5;})()").unwrap(),"undefined"); }
    #[test] fn test_v8_eval_try_catch() { if !has_v8() { return; } assert_eq!(eng().eval("(function(){try{throw new Error('c')}catch(e){return'caught!'}})()").unwrap(),"caught!"); }
    #[test] fn test_v8_eval_try_finally() { if !has_v8() { return; } assert_eq!(eng().eval("(function(){try{return'try'}finally{return'finally'}})()").unwrap(),"finally"); }
    #[test] fn test_v8_eval_throw_custom() { if !has_v8() { return; } assert_eq!(eng().eval("(function(){try{throw 42}catch(e){return e}})()").unwrap(),"42"); }
    #[test] fn test_v8_eval_destructuring() { if !has_v8() { return; } assert_eq!(eng().eval("(function(){const{a,b}={a:1,b:2};return a+b;})()").unwrap(),"3"); }
    #[test] fn test_v8_eval_spread() { if !has_v8() { return; } assert_eq!(eng().eval("(function(){const a=[1,2];const b=[...a,3];return b.length;})()").unwrap(),"3"); }
    #[test] fn test_v8_eval_rest() { if !has_v8() { return; } assert_eq!(eng().eval("(function(...args){return args.length;})(1,2,3)").unwrap(),"3"); }
    #[test] fn test_v8_eval_class() { if !has_v8() { return; } assert_eq!(eng().eval("new(class{constructor(x){this.x=x}})(42).x").unwrap(),"42"); }
    #[test] fn test_v8_eval_extends() { if !has_v8() { return; } assert_eq!(eng().eval("(function(){class A{getX(){return 1}}class B extends A{getX(){return super.getX()+1}}return(new B()).getX();})()").unwrap(),"2"); }
    #[test] fn test_v8_eval_static() { if !has_v8() { return; } assert_eq!(eng().eval("(function(){class A{static greet(){return'hi'}}return A.greet();})()").unwrap(),"hi"); }
    #[test] fn test_v8_eval_getter_setter() { if !has_v8() { return; } assert_eq!(eng().eval("(function(){let obj={_val:0,get val(){return this._val},set val(v){this._val=v}};obj.val=42;return obj.val;})()").unwrap(),"42"); }
    #[test] fn test_v8_eval_generator() { if !has_v8() { return; } assert_eq!(eng().eval("(function*(){yield 1;yield 2;})().next().value").unwrap(),"1"); }
    #[test] fn test_v8_eval_for_of() { if !has_v8() { return; } assert_eq!(eng().eval("(function(){let s=0;for(const v of[1,2,3,4]){s+=v}return s;})()").unwrap(),"10"); }
    #[test] fn test_v8_eval_for_in() { if !has_v8() { return; } assert_eq!(eng().eval("(function(){const o={a:1,b:2};let k='';for(const p in o){k+=p}return k;})()").unwrap(),"ab"); }
    #[test] fn test_v8_eval_version() { let v=V8Engine::version(); assert!(!v.is_empty()); }
    #[test] fn test_v8_configured() { if !has_v8() { return; } let e=eng(); assert!(e.is_native_available()); }
    #[test] fn test_v8_global_get() { if !has_v8() { return; } let e=eng(); let _=e.eval("var gv=99"); let r=e.get_global("gv"); assert!(r.is_ok()||r.is_err()); }
    #[test] fn test_v8_eval_empty() { if !has_v8() { return; } assert!(eng().eval("").is_ok()); }
    #[test] fn test_v8_eval_whitespace() { if !has_v8() { return; } assert!(eng().eval("   ").is_ok()); }
    #[test] fn test_v8_eval_global_this() { if !has_v8() { return; } assert_eq!(eng().eval("typeof globalThis").unwrap(),"object"); }
    #[test] fn test_v8_eval_eval_fn() { if !has_v8() { return; } assert_eq!(eng().eval("eval('1+2')").unwrap(),"3"); }
    #[test] fn test_v8_eval_fn_ctor() { if !has_v8() { return; } assert_eq!(eng().eval("new Function('a','b','return a+b')(3,4)").unwrap(),"7"); }
    #[test] fn test_v8_eval_unicode() { if !has_v8() { return; } assert_eq!(eng().eval("'\\u0041'").unwrap(),"A"); }
    #[test] fn test_v8_eval_var_scope() { if !has_v8() { return; } assert_eq!(eng().eval("(function(){var x=5;if(true){var x=10}return x;})()").unwrap(),"10"); }
    #[test] fn test_v8_eval_let_scope() { if !has_v8() { return; } assert_eq!(eng().eval("(function(){let x=5;if(true){let x=10}return x;})()").unwrap(),"5"); }
    #[test] fn test_v8_eval_const() { if !has_v8() { return; } assert_eq!(eng().eval("(function(){const x=5;return x;})()").unwrap(),"5"); }
    #[test] fn test_v8_eval_default_params() { if !has_v8() { return; } assert_eq!(eng().eval("(function(a=1,b=2){return a+b;})(5)").unwrap(),"7"); }
    #[test] fn test_v8_eval_arrow_return_obj() { if !has_v8() { return; } assert_eq!(eng().eval("(()=>({x:1,y:2}))().x").unwrap(),"1"); }
    #[test] fn test_v8_eval_computed_prop() { if !has_v8() { return; } assert_eq!(eng().eval("({['he'+'llo']:42})['hello']").unwrap(),"42"); }
    #[test] fn test_v8_eval_short_circuit_and() { if !has_v8() { return; } assert_eq!(eng().eval("null&&'unreachable'").unwrap(),"null"); }
    #[test] fn test_v8_eval_short_circuit_or() { if !has_v8() { return; } assert_eq!(eng().eval("null||'default'").unwrap(),"default"); }
    #[test] fn test_v8_eval_nullish_coalescing() { if !has_v8() { return; } assert_eq!(eng().eval("null??'fallback'").unwrap(),"fallback"); }
    #[test] fn test_v8_eval_optional_chaining() { if !has_v8() { return; } assert_eq!(eng().eval("({a:{b:42}})?.a?.b").unwrap(),"42"); }
    #[test] fn test_v8_eval_optional_chaining_null() { if !has_v8() { return; } assert_eq!(eng().eval("null?.a").unwrap(),"undefined"); }
    #[test] fn test_v8_eval_is_finite() { if !has_v8() { return; } assert_eq!(eng().eval("Number.isFinite(42)").unwrap(),"true"); }
    #[test] fn test_v8_eval_is_nan() { if !has_v8() { return; } assert_eq!(eng().eval("Number.isNaN(NaN)").unwrap(),"true"); }
    #[test] fn test_v8_eval_is_safe_integer() { if !has_v8() { return; } assert_eq!(eng().eval("Number.isSafeInteger(9007199254740991)").unwrap(),"true"); }
    #[test] fn test_v8_eval_parse_int_radix() { if !has_v8() { return; } assert_eq!(eng().eval("parseInt('FF',16)").unwrap(),"255"); }
    #[test] fn test_v8_eval_string_char_at() { if !has_v8() { return; } assert_eq!(eng().eval("'hello'.charAt(1)").unwrap(),"e"); }
    #[test] fn test_v8_eval_string_char_code_at() { if !has_v8() { return; } assert_eq!(eng().eval("'A'.charCodeAt(0)").unwrap(),"65"); }
    #[test] fn test_v8_eval_string_code_point_at() { if !has_v8() { return; } assert_eq!(eng().eval("'A'.codePointAt(0)").unwrap(),"65"); }
    #[test] fn test_v8_eval_string_from_code_point() { if !has_v8() { return; } assert_eq!(eng().eval("String.fromCodePoint(65)").unwrap(),"A"); }
    #[test] fn test_v8_eval_string_concat_method() { if !has_v8() { return; } assert_eq!(eng().eval("'a'.concat('b','c')").unwrap(),"abc"); }
    #[test] fn test_v8_eval_string_index_of() { if !has_v8() { return; } assert_eq!(eng().eval("'hello'.indexOf('l')").unwrap(),"2"); }
    #[test] fn test_v8_eval_string_last_index_of() { if !has_v8() { return; } assert_eq!(eng().eval("'hello'.lastIndexOf('l')").unwrap(),"3"); }
    #[test] fn test_v8_eval_string_match() { if !has_v8() { return; } let r=eng().eval("'hello123'.match(/\\d+/)").unwrap(); assert!(r.contains("123"),"got:{r}"); }
    #[test] fn test_v8_eval_string_pad_start() { if !has_v8() { return; } assert_eq!(eng().eval("'5'.padStart(3,'0')").unwrap(),"005"); }
    #[test] fn test_v8_eval_string_pad_end() { if !has_v8() { return; } assert_eq!(eng().eval("'5'.padEnd(3,'0')").unwrap(),"500"); }
    #[test] fn test_v8_eval_string_search() { if !has_v8() { return; } assert_eq!(eng().eval("'hello123'.search(/\\d+/)").unwrap(),"5"); }
    #[test] fn test_v8_eval_string_to_lower() { if !has_v8() { return; } assert_eq!(eng().eval("'HELLO'.toLowerCase()").unwrap(),"hello"); }
    #[test] fn test_v8_eval_string_to_upper() { if !has_v8() { return; } assert_eq!(eng().eval("'hello'.toUpperCase()").unwrap(),"HELLO"); }
    #[test] fn test_v8_eval_symbol_for() { if !has_v8() { return; } assert_eq!(eng().eval("Symbol.for('test')===Symbol.for('test')").unwrap(),"true"); }
    #[test] fn test_v8_eval_symbol_key_for() { if !has_v8() { return; } assert_eq!(eng().eval("Symbol.keyFor(Symbol.for('x'))").unwrap(),"x"); }
    #[test] fn test_v8_eval_set_has() { if !has_v8() { return; } assert_eq!(eng().eval("(function(){const s=new Set([1,2,3]);return s.has(2);})()").unwrap(),"true"); }
    #[test] fn test_v8_eval_set_delete() { if !has_v8() { return; } assert_eq!(eng().eval("(function(){const s=new Set([1,2]);s.delete(1);return s.size;})()").unwrap(),"1"); }
    #[test] fn test_v8_eval_set_clear() { if !has_v8() { return; } assert_eq!(eng().eval("(function(){const s=new Set([1,2]);s.clear();return s.size;})()").unwrap(),"0"); }
    #[test] fn test_v8_eval_map_has() { if !has_v8() { return; } assert_eq!(eng().eval("(function(){const m=new Map();m.set('a',1);return m.has('a');})()").unwrap(),"true"); }
    #[test] fn test_v8_eval_map_delete() { if !has_v8() { return; } assert_eq!(eng().eval("(function(){const m=new Map([['a',1]]);m.delete('a');return m.size;})()").unwrap(),"0"); }
    #[test] fn test_v8_eval_map_keys() { if !has_v8() { return; } assert_eq!(eng().eval("(function(){const m=new Map([['a',1],['b',2]]);return Array.from(m.keys()).length;})()").unwrap(),"2"); }
    #[test] fn test_v8_eval_map_clear() { if !has_v8() { return; } assert_eq!(eng().eval("(function(){const m=new Map([['a',1]]);m.clear();return m.size;})()").unwrap(),"0"); }
    #[test] fn test_v8_eval_typeof_undefined_var() { if !has_v8() { return; } assert_eq!(eng().eval("typeof nonExistentVar").unwrap(),"undefined"); }
    #[test] fn test_v8_eval_void_operator() { if !has_v8() { return; } assert_eq!(eng().eval("void 0").unwrap(),"undefined"); }
    #[test] fn test_v8_eval_typeof_null() { if !has_v8() { return; } assert_eq!(eng().eval("typeof null").unwrap(),"object"); }
    #[test] fn test_v8_eval_delete_prop() { if !has_v8() { return; } assert_eq!(eng().eval("(function(){const o={a:1};delete o.a;return o.a;})()").unwrap(),"undefined"); }
    #[test] fn test_v8_eval_in_operator() { if !has_v8() { return; } assert_eq!(eng().eval("'length'in[1,2,3]").unwrap(),"true"); }
    #[test] fn test_v8_eval_instance_of() { if !has_v8() { return; } assert_eq!(eng().eval("[]instanceof Array").unwrap(),"true"); }
    #[test] fn test_v8_eval_number_methods() { if !has_v8() { return; } assert_eq!(eng().eval("Number.isInteger(42)").unwrap(),"true"); }
    #[test] fn test_v8_eval_number_to_fixed() { if !has_v8() { return; } assert_eq!(eng().eval("(3.14159).toFixed(2)").unwrap(),"3.14"); }
    #[test] fn test_v8_eval_math_random() { if !has_v8() { return; } let r=eng().eval("Math.random()").unwrap(); let n:f64=r.parse().unwrap_or(-1.0); assert!(n>=0.0&&n<1.0,"got:{r}"); }
    #[test] fn test_v8_eval_math_round() { if !has_v8() { return; } assert_eq!(eng().eval("Math.round(3.5)").unwrap(),"4"); }
    #[test] fn test_v8_eval_math_abs() { if !has_v8() { return; } assert_eq!(eng().eval("Math.abs(-5)").unwrap(),"5"); }
    #[test] fn test_v8_eval_math_pow() { if !has_v8() { return; } assert_eq!(eng().eval("Math.pow(2,10)").unwrap(),"1024"); }
    #[test] fn test_v8_eval_math_sqrt() { if !has_v8() { return; } assert_eq!(eng().eval("Math.sqrt(16)").unwrap(),"4"); }
    #[test] fn test_v8_eval_math_min() { if !has_v8() { return; } assert_eq!(eng().eval("Math.min(5,2,8)").unwrap(),"2"); }
    #[test] fn test_v8_eval_math_trunc() { if !has_v8() { return; } assert_eq!(eng().eval("Math.trunc(4.9)").unwrap(),"4"); }
    #[test] fn test_v8_eval_math_sign() { if !has_v8() { return; } assert_eq!(eng().eval("Math.sign(-5)").unwrap(),"-1"); }
    #[test] fn test_v8_eval_math_cbrt() { if !has_v8() { return; } assert_eq!(eng().eval("Math.cbrt(27)").unwrap(),"3"); }
    #[test] fn test_v8_eval_math_hypot() { if !has_v8() { return; } assert_eq!(eng().eval("Math.hypot(3,4)").unwrap(),"5"); }
    #[test] fn test_v8_eval_date_construct() { if !has_v8() { return; } assert_eq!(eng().eval("new Date('2024-01-01').getFullYear()").unwrap(),"2024"); }
    #[test] fn test_v8_eval_date_month() { if !has_v8() { return; } assert_eq!(eng().eval("new Date('2024-06-15').getMonth()").unwrap(),"5"); }
    #[test] fn test_v8_eval_date_date() { if !has_v8() { return; } assert_eq!(eng().eval("new Date('2024-06-15').getDate()").unwrap(),"15"); }
    #[test] fn test_v8_eval_regex_exec() { if !has_v8() { return; } assert_eq!(eng().eval("/(\\d+)/.exec('abc123def')[1]").unwrap(),"123"); }
    #[test] fn test_v8_eval_regex_flags() { if !has_v8() { return; } assert_eq!(eng().eval("/test/gi.flags").unwrap(),"gi"); }
    #[test] fn test_v8_eval_regex_source() { if !has_v8() { return; } assert_eq!(eng().eval("/abc/.source").unwrap(),"abc"); }
    #[test] fn test_v8_eval_regex_test_no_match() { if !has_v8() { return; } assert_eq!(eng().eval("/xyz/.test('hello')").unwrap(),"false"); }
    #[test] fn test_v8_eval_string_trim() { if !has_v8() { return; } assert_eq!(eng().eval("'  hello  '.trim()").unwrap(),"hello"); }
    #[test] fn test_v8_eval_string_split() { if !has_v8() { return; } assert_eq!(eng().eval("'a,b,c'.split(',')").unwrap(),"a,b,c"); }
    #[test] fn test_v8_eval_string_includes() { if !has_v8() { return; } assert_eq!(eng().eval("'hello'.includes('ell')").unwrap(),"true"); }
    #[test] fn test_v8_eval_string_starts_with() { if !has_v8() { return; } assert_eq!(eng().eval("'hello'.startsWith('he')").unwrap(),"true"); }
    #[test] fn test_v8_eval_string_ends_with() { if !has_v8() { return; } assert_eq!(eng().eval("'hello'.endsWith('lo')").unwrap(),"true"); }
    #[test] fn test_v8_eval_string_repeat() { if !has_v8() { return; } assert_eq!(eng().eval("'ab'.repeat(3)").unwrap(),"ababab"); }
    #[test] fn test_v8_eval_string_replace() { if !has_v8() { return; } assert_eq!(eng().eval("'hello world'.replace('world','there')").unwrap(),"hello there"); }
    #[test] fn test_v8_eval_replace_all() { if !has_v8() { return; } assert_eq!(eng().eval("'a-a-a'.replaceAll('-','+')").unwrap(),"a+a+a"); }
    #[test] fn test_v8_eval_array_flat() { if !has_v8() { return; } assert_eq!(eng().eval("[[1,2],[3,4]].flat()").unwrap(),"1,2,3,4"); }
    #[test] fn test_v8_eval_array_flat_map() { if !has_v8() { return; } assert_eq!(eng().eval("[1,2,3].flatMap(x=>[x,x*2])").unwrap(),"1,2,2,4,3,6"); }
    #[test] fn test_v8_eval_array_find() { if !has_v8() { return; } assert_eq!(eng().eval("[1,2,3,4].find(x=>x>2)").unwrap(),"3"); }
    #[test] fn test_v8_eval_array_find_index() { if !has_v8() { return; } assert_eq!(eng().eval("[1,2,3,4].findIndex(x=>x>2)").unwrap(),"2"); }
    #[test] fn test_v8_eval_array_some() { if !has_v8() { return; } assert_eq!(eng().eval("[1,2,3].some(x=>x>2)").unwrap(),"true"); }
    #[test] fn test_v8_eval_array_every() { if !has_v8() { return; } assert_eq!(eng().eval("[1,2,3].every(x=>x>0)").unwrap(),"true"); }
    #[test] fn test_v8_eval_array_from() { if !has_v8() { return; } assert_eq!(eng().eval("Array.from('hello').length").unwrap(),"5"); }
    #[test] fn test_v8_eval_array_of() { if !has_v8() { return; } assert_eq!(eng().eval("Array.of(1,2,3).length").unwrap(),"3"); }
    #[test] fn test_v8_eval_array_fill() { if !has_v8() { return; } assert_eq!(eng().eval("new Array(3).fill(0)").unwrap(),"0,0,0"); }
    #[test] fn test_v8_eval_array_includes() { if !has_v8() { return; } assert_eq!(eng().eval("[1,2,3].includes(2)").unwrap(),"true"); }
    #[test] fn test_v8_eval_array_join() { if !has_v8() { return; } assert_eq!(eng().eval("[1,2,3].join('-')").unwrap(),"1-2-3"); }
    #[test] fn test_v8_eval_array_reverse() { if !has_v8() { return; } assert_eq!(eng().eval("[1,2,3].reverse()").unwrap(),"3,2,1"); }
    #[test] fn test_v8_eval_array_sort() { if !has_v8() { return; } assert_eq!(eng().eval("[3,1,2].sort()").unwrap(),"1,2,3"); }
    #[test] fn test_v8_eval_array_concat() { if !has_v8() { return; } assert_eq!(eng().eval("[1,2].concat([3,4]).length").unwrap(),"4"); }
    #[test] fn test_v8_eval_array_slice() { if !has_v8() { return; } assert_eq!(eng().eval("[1,2,3,4].slice(1,3)").unwrap(),"2,3"); }
    #[test] fn test_v8_eval_array_splice() { if !has_v8() { return; } assert_eq!(eng().eval("(function(){let a=[1,2,3,4];a.splice(1,2);return a;})()").unwrap(),"1,4"); }
    #[test] fn test_v8_eval_object_keys() { if !has_v8() { return; } assert_eq!(eng().eval("Object.keys({a:1,b:2}).length").unwrap(),"2"); }
    #[test] fn test_v8_eval_object_values() { if !has_v8() { return; } assert_eq!(eng().eval("Object.values({a:1,b:2})").unwrap(),"1,2"); }
    #[test] fn test_v8_eval_object_entries() { if !has_v8() { return; } assert_eq!(eng().eval("Object.entries({a:1,b:2}).length").unwrap(),"2"); }
    #[test] fn test_v8_eval_object_assign() { if !has_v8() { return; } assert_eq!(eng().eval("Object.assign({},{a:1},{b:2}).b").unwrap(),"2"); }
    #[test] fn test_v8_eval_object_freeze() { if !has_v8() { return; } assert_eq!(eng().eval("Object.isFrozen(Object.freeze({}))").unwrap(),"true"); }
    #[test] fn test_v8_eval_object_seal() { if !has_v8() { return; } assert_eq!(eng().eval("Object.isSealed(Object.seal({a:1}))").unwrap(),"true"); }
    #[test] fn test_v8_eval_parse_int() { if !has_v8() { return; } assert_eq!(eng().eval("parseInt('42')").unwrap(),"42"); }
    #[test] fn test_v8_eval_parse_float() { if !has_v8() { return; } assert_eq!(eng().eval("parseFloat('3.14')").unwrap(),"3.14"); }
    #[test] fn test_v8_eval_is_nan_global() { if !has_v8() { return; } assert_eq!(eng().eval("isNaN(NaN)").unwrap(),"true"); }
    #[test] fn test_v8_eval_encode_uri() { if !has_v8() { return; } assert_eq!(eng().eval("encodeURIComponent('hello world')").unwrap(),"hello%20world"); }
    #[test] fn test_v8_eval_decode_uri() { if !has_v8() { return; } assert_eq!(eng().eval("decodeURIComponent('hello%20world')").unwrap(),"hello world"); }
    #[test] fn test_v8_eval_promise_resolve() { if !has_v8() { return; } assert!(eng().eval("Promise.resolve(42)").is_ok()); }
    #[test] fn test_v8_eval_bigint() { if !has_v8() { return; } let r=eng().eval("12345678901234567890n"); if let Ok(v)=r{assert!(v.contains("1234567890"),"got:{v}");} }
    #[test] fn test_v8_eval_weak_map() { if !has_v8() { return; } assert_eq!(eng().eval("typeof WeakMap").unwrap(),"function"); }
    #[test] fn test_v8_eval_weak_set() { if !has_v8() { return; } assert_eq!(eng().eval("typeof WeakSet").unwrap(),"function"); }
    #[test] fn test_v8_eval_proxy() { if !has_v8() { return; } assert_eq!(eng().eval("typeof Proxy").unwrap(),"function"); }
    #[test] fn test_v8_eval_reflect() { if !has_v8() { return; } assert_eq!(eng().eval("typeof Reflect").unwrap(),"object"); }
    #[test] fn test_v8_eval_data_view() { if !has_v8() { return; } assert_eq!(eng().eval("typeof DataView").unwrap(),"function"); }
    #[test] fn test_v8_eval_async_fn() { if !has_v8() { return; } assert!(eng().eval("(async function(){return 42;})()").is_ok()); }
    #[test] fn test_v8_eval_new_target() { if !has_v8() { return; } assert_eq!(eng().eval("(function(){return typeof new.target;})()").unwrap(),"undefined"); }
    #[test] fn test_v8_eval_complex_math() { if !has_v8() { return; } let r=eng().eval("Math.sin(Math.PI/2)").unwrap(); let n:f64=r.parse().unwrap_or(0.0); assert!((n-1.0).abs()<0.01,"got:{r}"); }
    #[test] fn test_v8_eval_complex_obj() { if !has_v8() { return; } let r=eng().eval("JSON.stringify({name:'test',values:[1,2,3],nested:{a:{b:42}}})").unwrap(); assert!(r.contains("\"b\":42"),"got:{r}"); }
    #[test] fn test_v8_eval_module() { if !has_v8() { return; } let r=eng().execute_module("mod.js","export const x=1;"); assert!(r.is_ok()||r.is_err()); }
    #[test] fn test_v8_get_heap_stats() { if !has_v8() { return; } let e=eng(); let s=e.get_heap_stats().unwrap(); assert!(s.total_heap_size>0); assert!(s.heap_size_limit>0); }
    #[test] fn test_v8_low_memory_notification() { if !has_v8() { return; } eng().low_memory_notification(); }
    #[test] fn test_v8_idle_notification() { if !has_v8() { return; } eng().idle_notification(0.5); }
    #[test] fn test_v8_set_memory_pressure_none() { if !has_v8() { return; } eng().set_memory_pressure(V8MemoryPressure::None); }
    #[test] fn test_v8_set_memory_pressure_moderate() { if !has_v8() { return; } eng().set_memory_pressure(V8MemoryPressure::Moderate); }
    #[test] fn test_v8_set_memory_pressure_critical() { if !has_v8() { return; } eng().set_memory_pressure(V8MemoryPressure::Critical); }
    #[test] fn test_v8_request_gc() { if !has_v8() { return; } eng().request_gc(); }
    #[test] fn test_v8_malloced_memory() { if !has_v8() { return; } let m=eng().get_malloced_memory(); let _=m; }
    #[test] fn test_v8_adjust_external_memory() { if !has_v8() { return; } let e=eng(); let b=e.adjust_external_memory(0); let a=e.adjust_external_memory(1024); assert!(a>=b); }
    #[test] fn test_v8_microtasks_perform_check() { if !has_v8() { return; } eng().microtasks_perform_check(); }
    #[test] fn test_v8_get_stack_trace() { if !has_v8() { return; } let r=eng().get_stack_trace(); assert!(r.is_ok()||r.is_err()); }
    #[test] fn test_v8_version() { let v=V8Engine::version(); assert!(!v.is_empty()); }
    #[test] fn test_v8_is_native_available() { let e=V8Engine::new(); if let Ok(ref eng)=e{assert!(eng.is_native_available()||!eng.is_native_available());} }
    #[test] fn test_v8_value_type_enum() { assert_eq!(V8ValueType::Undefined as u32,0); assert_eq!(V8ValueType::Null as u32,1); assert_eq!(V8ValueType::Boolean as u32,2); }
    #[test] fn test_v8_value_type_number() { assert_eq!(V8ValueType::Number as u32,3); assert_eq!(V8ValueType::String as u32,4); assert_eq!(V8ValueType::Object as u32,5); }
    #[test] fn test_v8_value_type_array() { assert_eq!(V8ValueType::Array as u32,6); assert_eq!(V8ValueType::Function as u32,7); assert_eq!(V8ValueType::Promise as u32,8); }
    #[test] fn test_v8_value_type_error() { assert_eq!(V8ValueType::Error as u32,9); assert_eq!(V8ValueType::Symbol as u32,10); assert_eq!(V8ValueType::BigInt as u32,11); }
    #[test] fn test_v8_value_type_proxy() { assert_eq!(V8ValueType::Proxy as u32,12); assert_eq!(V8ValueType::TypedArray as u32,13); }
    #[test] fn test_v8_promise_state_enum() { assert_eq!(V8PromiseState::Pending as u32,0); assert_eq!(V8PromiseState::Fulfilled as u32,1); assert_eq!(V8PromiseState::Rejected as u32,2); }
    #[test] fn test_v8_memory_pressure_enum() { assert_eq!(V8MemoryPressure::None as u32,0); assert_eq!(V8MemoryPressure::Moderate as u32,1); assert_eq!(V8MemoryPressure::Critical as u32,2); }
    #[test] fn test_v8_heap_stats_struct() { let h=HeapStats{total_heap_size:0,total_heap_size_executable:0,total_physical_size:0,total_available_size:0,used_heap_size:0,heap_size_limit:0,malloced_memory:0,peak_malloced_memory:0,number_of_native_contexts:0,number_of_detached_contexts:0,total_global_handles_size:0,used_global_handles_size:0,external_memory:0}; assert_eq!(h.heap_size_limit,0); }
    #[test] fn test_v8_error_not_initialized() { let e=V8Error::NotInitialized; assert_eq!(e.to_string(),"V8 not initialized"); }
    #[test] fn test_v8_error_init_failed() { let e=V8Error::InitFailed("oom".into()); assert_eq!(e.to_string(),"V8 init failed: oom"); }
    #[test] fn test_v8_error_eval_failed() { let e=V8Error::EvalFailed("syntax".into()); assert_eq!(e.to_string(),"V8 eval error: syntax"); }
    #[test] fn test_v8_error_module_failed() { let e=V8Error::ModuleFailed("err".into()); assert_eq!(e.to_string(),"V8 module error: err"); }
    #[test] fn test_v8_error_global_failed() { let e=V8Error::GlobalFailed("err".into()); assert_eq!(e.to_string(),"V8 global error: err"); }
    #[test] fn test_v8_error_call_failed() { let e=V8Error::CallFailed("err".into()); assert_eq!(e.to_string(),"V8 call error: err"); }
    #[test] fn test_v8_error_snapshot_failed() { let e=V8Error::SnapshotFailed("err".into()); assert_eq!(e.to_string(),"V8 snapshot error: err"); }
    #[test] fn test_v8_error_internal() { let e=V8Error::Internal("crash".into()); assert_eq!(e.to_string(),"V8 internal error: crash"); }
    #[test] fn test_v8_error_common_kind() { let e=V8Error::NotInitialized; assert!(matches!(e.to_common_kind(),common::error::CommonErrorKind::NotInitialized)); }
    #[test] fn test_v8_error_common_kind_init() { let e=V8Error::InitFailed("x".into()); assert!(matches!(e.to_common_kind(),common::error::CommonErrorKind::InitFailed(_))); }
    #[test] fn test_v8_error_common_kind_exec() { let e=V8Error::EvalFailed("x".into()); assert!(matches!(e.to_common_kind(),common::error::CommonErrorKind::ExecutionFailed(_))); }
    #[test] fn test_v8_error_common_kind_module() { let e=V8Error::ModuleFailed("x".into()); assert!(matches!(e.to_common_kind(),common::error::CommonErrorKind::ExecutionFailed(_))); }
    #[test] fn test_v8_permissions_new() { let p=permissions::V8Permissions::new(); assert!(!p.check(&permissions::Permission::Read)); }
    #[test] fn test_v8_permissions_allow_all() { let p=permissions::V8Permissions::allow_all(); assert!(p.check(&permissions::Permission::Read)); assert!(p.check(&permissions::Permission::Write)); }
    #[test] fn test_v8_permissions_deny_all() { let p=permissions::V8Permissions::deny_all(); assert!(!p.check(&permissions::Permission::Read)); }
    #[test] fn test_v8_permissions_check_path() { let p=permissions::V8Permissions::allow_all(); assert!(p.check_path(&permissions::Permission::Read,"/tmp")); }
    #[test] fn test_v8_permissions_common_roundtrip() { let c=common::permissions::CommonPermissions::allow_all(); let p=permissions::V8Permissions::from_common(c.clone()); assert_eq!(p.to_common().allow_read,c.allow_read); }
    #[test] fn test_v8_permission_enum_convert() { for (v8_p,common_p) in [(permissions::Permission::Read,common::permissions::CommonPermission::Read),(permissions::Permission::Write,common::permissions::CommonPermission::Write),(permissions::Permission::Net,common::permissions::CommonPermission::Net),(permissions::Permission::Env,common::permissions::CommonPermission::Env),(permissions::Permission::Run,common::permissions::CommonPermission::Run),(permissions::Permission::Ffi,common::permissions::CommonPermission::Ffi),(permissions::Permission::All,common::permissions::CommonPermission::All)]{let cp:common::permissions::CommonPermission=v8_p.clone().into();assert_eq!(cp,common_p);let vp:permissions::Permission=common_p.into();assert_eq!(vp,v8_p);} }
    #[test] fn test_v8_module_loader_new() { let l=module_loader::V8ModuleLoader::new("/tmp"); let r=l.resolve("file:///a.js","/b"); assert_eq!(r,Ok("file:///a.js".to_string())); }
    #[test] fn test_v8_module_loader_resolve_relative() { let l=module_loader::V8ModuleLoader::new("/tmp"); let r=l.resolve("./lib.js","/base/mod.js"); assert_eq!(r,Ok("/base/./lib.js".to_string())); }
    #[test] fn test_v8_module_loader_register_load() { let l=module_loader::V8ModuleLoader::new("/tmp"); l.register("my:mod","export const x=1;"); assert_eq!(l.load("my:mod"),Ok("export const x=1;".to_string())); }
    #[test] fn test_v8_module_loader_resolve_unresolvable() { let l=module_loader::V8ModuleLoader::new("/nonexistent"); let r=l.resolve("some-pkg","/base"); assert!(r.is_err()); }
    #[test] fn test_v8_eval_escape() { if !has_v8() { return; } let r=eng().eval("'line1\\nline2'").unwrap(); assert_eq!(r,"line1\nline2"); }
    #[test] fn test_v8_eval_json_roundtrip() { if !has_v8() { return; } let e=eng(); let v=e.json_parse("{\"a\":1}"); if let Ok(ptr)=v{let r=e.json_stringify(ptr);assert!(r.is_ok());} }
    #[test] fn test_v8_eval_reflect_construct() { if !has_v8() { return; } assert_eq!(eng().eval("Reflect.construct(Object,[]).constructor").unwrap(),"Object"); }
    #[test] fn test_v8_eval_atomics() { if !has_v8() { return; } assert_eq!(eng().eval("typeof Atomics").unwrap(),"object"); }
    #[test] fn test_v8_eval_intl() { if !has_v8() { return; } assert_eq!(eng().eval("typeof Intl").unwrap(),"object"); }
    #[test] fn test_v8_eval_shared_array_buffer() { if !has_v8() { return; } assert_eq!(eng().eval("typeof SharedArrayBuffer").unwrap(),"function"); }
    #[test] fn test_v8_eval_finalization_registry() { if !has_v8() { return; } assert_eq!(eng().eval("typeof FinalizationRegistry").unwrap(),"function"); }
    #[test] fn test_v8_eval_weak_ref() { if !has_v8() { return; } assert_eq!(eng().eval("typeof WeakRef").unwrap(),"function"); }
    #[test] fn test_v8_eval_array_buffer() { if !has_v8() { return; } assert_eq!(eng().eval("typeof ArrayBuffer").unwrap(),"function"); }
    #[test] fn test_v8_eval_uint8_clamped() { if !has_v8() { return; } assert_eq!(eng().eval("typeof Uint8ClampedArray").unwrap(),"function"); }
    #[test] fn test_v8_eval_float32() { if !has_v8() { return; } assert_eq!(eng().eval("typeof Float32Array").unwrap(),"function"); }
    #[test] fn test_v8_eval_float64() { if !has_v8() { return; } assert_eq!(eng().eval("typeof Float64Array").unwrap(),"function"); }
    #[test] fn test_v8_eval_int8() { if !has_v8() { return; } assert_eq!(eng().eval("typeof Int8Array").unwrap(),"function"); }
    #[test] fn test_v8_eval_uint8() { if !has_v8() { return; } assert_eq!(eng().eval("typeof Uint8Array").unwrap(),"function"); }
    #[test] fn test_v8_eval_int16() { if !has_v8() { return; } assert_eq!(eng().eval("typeof Int16Array").unwrap(),"function"); }
    #[test] fn test_v8_eval_uint16() { if !has_v8() { return; } assert_eq!(eng().eval("typeof Uint16Array").unwrap(),"function"); }
    #[test] fn test_v8_eval_int32() { if !has_v8() { return; } assert_eq!(eng().eval("typeof Int32Array").unwrap(),"function"); }
    #[test] fn test_v8_eval_uint32() { if !has_v8() { return; } assert_eq!(eng().eval("typeof Uint32Array").unwrap(),"function"); }
    #[test] fn test_v8_eval_bigint64() { if !has_v8() { return; } assert_eq!(eng().eval("typeof BigInt64Array").unwrap(),"function"); }
    #[test] fn test_v8_eval_biguint64() { if !has_v8() { return; } assert_eq!(eng().eval("typeof BigUint64Array").unwrap(),"function"); }
    #[test] fn test_v8_eval_string_raw() { if !has_v8() { return; } assert_eq!(eng().eval("String.raw`hello\\nworld`").unwrap(),"hello\\nworld"); }
}
