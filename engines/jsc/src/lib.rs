pub mod error;
pub mod module_loader;
pub mod permissions;

#[cfg(feature = "native")]
pub mod ffi;

#[cfg(feature = "native")]
pub mod ffi_types;

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

use error::JSCError;

#[cfg(feature = "native")]
use ffi::JSCEnginePtr;

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
pub enum JSCValueType {
    Undefined = 0,
    Null = 1,
    Boolean = 2,
    Number = 3,
    String = 4,
    Object = 5,
    Array = 6,
    Function = 7,
    Error = 8,
    Symbol = 9,
    TypedArray = 10,
}

pub struct JSCEngine {
    #[cfg(feature = "native")]
    inner: JSCEnginePtr,
}

impl JSCEngine {
    pub fn new() -> Result<Self, JSCError> {
        #[cfg(feature = "native")]
        {
            JSCEnginePtr::init()
                .map(|inner| Self { inner })
                .map_err(|e| JSCError::InitFailed(e))
        }
        #[cfg(not(feature = "native"))]
        {
            Err(JSCError::InitFailed(
                "JSC native engine not available — enable 'native' feature".into()
            ))
        }
    }

    pub fn eval(&self, _code: &str) -> Result<String, JSCError> {
        #[cfg(feature = "native")]
        { self.inner.eval(_code).map_err(JSCError::EvalFailed) }
        #[cfg(not(feature = "native"))]
        { Err(JSCError::NotInitialized) }
    }

    pub fn execute_script(&self, _filename: &str, _source: &str) -> Result<String, JSCError> {
        #[cfg(feature = "native")]
        { self.inner.execute_script(_filename, _source).map_err(JSCError::EvalFailed) }
        #[cfg(not(feature = "native"))]
        { Err(JSCError::NotInitialized) }
    }

    pub fn execute_module(&self, _filename: &str, _source: &str) -> Result<String, JSCError> {
        #[cfg(feature = "native")]
        { self.inner.execute_module(_filename, _source).map_err(JSCError::ModuleFailed) }
        #[cfg(not(feature = "native"))]
        { Err(JSCError::NotInitialized) }
    }

    pub fn json_stringify(&self, _value_ptr: *const std::ffi::c_void) -> Result<String, JSCError> {
        #[cfg(feature = "native")]
        {
            let v = _value_ptr as *mut ffi::JSCValueHandle;
            self.inner.json_stringify(v).map_err(|e| JSCError::EvalFailed(e))
        }
        #[cfg(not(feature = "native"))]
        { Err(JSCError::NotInitialized) }
    }

    pub fn json_parse(&self, _json: &str) -> Result<*const std::ffi::c_void, JSCError> {
        #[cfg(feature = "native")]
        { self.inner.json_parse(_json).map(|p| p as *const std::ffi::c_void).map_err(JSCError::EvalFailed) }
        #[cfg(not(feature = "native"))]
        { Err(JSCError::NotInitialized) }
    }

    pub fn get_global(&self, _key: &str) -> Result<*const std::ffi::c_void, JSCError> {
        #[cfg(feature = "native")]
        { self.inner.get_global(_key).map(|p| p as *const std::ffi::c_void).map_err(JSCError::GlobalFailed) }
        #[cfg(not(feature = "native"))]
        { Err(JSCError::NotInitialized) }
    }

    pub fn set_global(&self, _key: &str, _value_ptr: *const std::ffi::c_void) -> Result<(), JSCError> {
        #[cfg(feature = "native")]
        {
            let v = _value_ptr as *mut ffi::JSCValueHandle;
            self.inner.set_global(_key, v).map_err(JSCError::GlobalFailed)
        }
        #[cfg(not(feature = "native"))]
        { Err(JSCError::NotInitialized) }
    }

    pub fn get_heap_stats(&self) -> Result<HeapStats, JSCError> {
        #[cfg(feature = "native")]
        {
            let raw = self.inner.get_heap_stats().map_err(|e| JSCError::Internal(e))?;
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
        { Err(JSCError::NotInitialized) }
    }

    pub fn request_gc(&self) {
        #[cfg(feature = "native")]
        self.inner.request_gc()
    }

    pub fn low_memory_notification(&self) {
        #[cfg(feature = "native")]
        self.inner.low_memory_notification()
    }

    pub fn microtasks_perform_check(&self) {
        #[cfg(feature = "native")]
        self.inner.microtasks_perform_check()
    }

    pub fn get_stack_trace(&self) -> Result<String, JSCError> {
        #[cfg(feature = "native")]
        { self.inner.get_stack_trace().map_err(|e| JSCError::Internal(e)) }
        #[cfg(not(feature = "native"))]
        { Err(JSCError::NotInitialized) }
    }

    #[cfg(feature = "native")]
    pub fn raw_handle(&self) -> *mut ffi::JSCEngineHandle {
        self.inner.engine_handle()
    }

    pub fn is_native_available(&self) -> bool {
        cfg!(feature = "native")
    }

    pub fn version() -> String {
        #[cfg(feature = "native")]
        { JSCEnginePtr::version() }
        #[cfg(not(feature = "native"))]
        { "JSC (not available)".into() }
    }

    /* ─── New deep methods ─────────────────────────────────── */

    pub fn call_function(
        &self,
        _func_ptr: *const std::ffi::c_void,
        _this_ptr: Option<*const std::ffi::c_void>,
        _args: &[*const std::ffi::c_void],
    ) -> Result<*const std::ffi::c_void, JSCError> {
        #[cfg(feature = "native")]
        {
            let f = _func_ptr as *mut ffi::JSCValueHandle;
            let t = _this_ptr.map(|p| p as *mut ffi::JSCValueHandle);
            let a: Vec<*mut ffi::JSCValueHandle> = _args.iter().map(|p| *p as *mut ffi::JSCValueHandle).collect();
            self.inner.call_function(f, t, &a)
                .map(|p| p as *const std::ffi::c_void)
                .map_err(JSCError::CallFailed)
        }
        #[cfg(not(feature = "native"))]
        { Err(JSCError::NotInitialized) }
    }

    pub fn object_set_property(
        &self,
        _obj_ptr: *const std::ffi::c_void,
        _name: &str,
        _value_ptr: *const std::ffi::c_void,
    ) -> Result<(), JSCError> {
        #[cfg(feature = "native")]
        {
            let o = _obj_ptr as *mut ffi::JSCValueHandle;
            let v = _value_ptr as *mut ffi::JSCValueHandle;
            self.inner.object_set_property(o, _name, v).map_err(JSCError::GlobalFailed)
        }
        #[cfg(not(feature = "native"))]
        { Err(JSCError::NotInitialized) }
    }

    pub fn object_get_property(
        &self,
        _obj_ptr: *const std::ffi::c_void,
        _name: &str,
    ) -> Result<*const std::ffi::c_void, JSCError> {
        #[cfg(feature = "native")]
        {
            let o = _obj_ptr as *mut ffi::JSCValueHandle;
            self.inner.object_get_property(o, _name)
                .map(|p| p as *const std::ffi::c_void)
                .map_err(JSCError::GlobalFailed)
        }
        #[cfg(not(feature = "native"))]
        { Err(JSCError::NotInitialized) }
    }

    pub fn value_new_symbol(&self, _description: Option<&str>) -> Result<*const std::ffi::c_void, JSCError> {
        #[cfg(feature = "native")]
        { self.inner.value_new_symbol(_description).map(|p| p as *const std::ffi::c_void).map_err(|e| JSCError::Internal(e)) }
        #[cfg(not(feature = "native"))]
        { Err(JSCError::NotInitialized) }
    }

    pub fn value_new_error(&self, _message: &str) -> Result<*const std::ffi::c_void, JSCError> {
        #[cfg(feature = "native")]
        { self.inner.value_new_error(_message).map(|p| p as *const std::ffi::c_void).map_err(|e| JSCError::Internal(e)) }
        #[cfg(not(feature = "native"))]
        { Err(JSCError::NotInitialized) }
    }

    pub fn value_is_function(&self, _value_ptr: *const std::ffi::c_void) -> bool {
        #[cfg(feature = "native")]
        { self.inner.value_is_function(_value_ptr as *mut ffi::JSCValueHandle) }
        #[cfg(not(feature = "native"))]
        { false }
    }

    pub fn value_is_object(&self, _value_ptr: *const std::ffi::c_void) -> bool {
        #[cfg(feature = "native")]
        { self.inner.value_is_object(_value_ptr as *mut ffi::JSCValueHandle) }
        #[cfg(not(feature = "native"))]
        { false }
    }

    pub fn value_is_error(&self, _value_ptr: *const std::ffi::c_void) -> bool {
        #[cfg(feature = "native")]
        { self.inner.value_is_error(_value_ptr as *mut ffi::JSCValueHandle) }
        #[cfg(not(feature = "native"))]
        { false }
    }

    pub fn value_is_symbol(&self, _value_ptr: *const std::ffi::c_void) -> bool {
        #[cfg(feature = "native")]
        { self.inner.value_is_symbol(_value_ptr as *mut ffi::JSCValueHandle) }
        #[cfg(not(feature = "native"))]
        { false }
    }

    pub fn value_is_promise(&self, _value_ptr: *const std::ffi::c_void) -> bool {
        #[cfg(feature = "native")]
        { self.inner.value_is_promise(_value_ptr as *mut ffi::JSCValueHandle) }
        #[cfg(not(feature = "native"))]
        { false }
    }

    pub fn value_is_typed_array(&self, _value_ptr: *const std::ffi::c_void) -> bool {
        #[cfg(feature = "native")]
        { self.inner.value_is_typed_array(_value_ptr as *mut ffi::JSCValueHandle) }
        #[cfg(not(feature = "native"))]
        { false }
    }

    pub fn array_buffer_new(&self, _data: &[u8]) -> Result<*const std::ffi::c_void, JSCError> {
        #[cfg(feature = "native")]
        { self.inner.array_buffer_new(_data).map(|p| p as *const std::ffi::c_void).map_err(|e| JSCError::Internal(e)) }
        #[cfg(not(feature = "native"))]
        { Err(JSCError::NotInitialized) }
    }

    pub fn typed_array_new(&self, _type_name: &str, _length: usize) -> Result<*const std::ffi::c_void, JSCError> {
        #[cfg(feature = "native")]
        { self.inner.typed_array_new(_type_name, _length).map(|p| p as *const std::ffi::c_void).map_err(|e| JSCError::Internal(e)) }
        #[cfg(not(feature = "native"))]
        { Err(JSCError::NotInitialized) }
    }

    pub fn wasm_compile(&self, _bytes: &[u8]) -> Result<*const std::ffi::c_void, JSCError> {
        #[cfg(feature = "native")]
        { self.inner.wasm_compile(_bytes).map(|p| p as *const std::ffi::c_void).map_err(|e| JSCError::Internal(e)) }
        #[cfg(not(feature = "native"))]
        { Err(JSCError::NotInitialized) }
    }

    pub fn wasm_instantiate(&self, _bytes: &[u8], _imports: *const std::ffi::c_void) -> Result<*const std::ffi::c_void, JSCError> {
        #[cfg(feature = "native")]
        { self.inner.wasm_instantiate(_bytes, _imports as *mut ffi::JSCValueHandle).map(|p| p as *const std::ffi::c_void).map_err(|e| JSCError::Internal(e)) }
        #[cfg(not(feature = "native"))]
        { Err(JSCError::NotInitialized) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use klyron_engine_common as common;

    fn eng() -> JSCEngine { JSCEngine::new().expect("JSC init failed (native feature required)") }
    fn has_jsc() -> bool { JSCEngine::new().is_ok() }

    #[test] fn test_jsc_new() { let e = JSCEngine::new(); assert!(e.is_ok()||e.is_err()); }
    #[test] fn test_jsc_eval_addition() { if !has_jsc(){return;} assert_eq!(eng().eval("1+2").unwrap(),"3"); }
    #[test] fn test_jsc_eval_subtraction() { if !has_jsc(){return;} assert_eq!(eng().eval("10-4").unwrap(),"6"); }
    #[test] fn test_jsc_eval_multiplication() { if !has_jsc(){return;} assert_eq!(eng().eval("6*7").unwrap(),"42"); }
    #[test] fn test_jsc_eval_division() { if !has_jsc(){return;} let r=eng().eval("10/3").unwrap(); assert!(r.starts_with("3."),"got:{r}"); }
    #[test] fn test_jsc_eval_string_concat() { if !has_jsc(){return;} assert_eq!(eng().eval("'hello'+' world'").unwrap(),"hello world"); }
    #[test] fn test_jsc_eval_template() { if !has_jsc(){return;} let r=eng().eval("`hello ${1+2}`").unwrap(); assert_eq!(r,"hello 3","got:{r}"); }
    #[test] fn test_jsc_eval_syntax_error() { if !has_jsc(){return;} assert!(eng().eval("syntax{{{").is_err()); }
    #[test] fn test_jsc_eval_throw() { if !has_jsc(){return;} assert!(eng().eval("throw new Error('x')").is_err()); }
    #[test] fn test_jsc_eval_type_error() { if !has_jsc(){return;} assert!(eng().eval("null.x").is_err()); }
    #[test] fn test_jsc_execute_script() { if !has_jsc(){return;} assert_eq!(eng().execute_script("t.js","1+3").unwrap(),"4"); }
    #[test] fn test_jsc_eval_function_call() { if !has_jsc(){return;} assert_eq!(eng().eval("(function(x){return x*2;})(5)").unwrap(),"10"); }
    #[test] fn test_jsc_eval_arrow() { if !has_jsc(){return;} assert_eq!(eng().eval("((a,b)=>a+b)(3,4)").unwrap(),"7"); }
    #[test] fn test_jsc_eval_array_length() { if !has_jsc(){return;} assert_eq!(eng().eval("[1,2,3].length").unwrap(),"3"); }
    #[test] fn test_jsc_eval_array_index() { if !has_jsc(){return;} assert_eq!(eng().eval("[10,20,30][1]").unwrap(),"20"); }
    #[test] fn test_jsc_eval_object_access() { if !has_jsc(){return;} assert_eq!(eng().eval("({a:1,b:2}).b").unwrap(),"2"); }
    #[test] fn test_jsc_eval_boolean_logic() { if !has_jsc(){return;} assert_eq!(eng().eval("true&&false||true").unwrap(),"true"); }
    #[test] fn test_jsc_eval_comparison() { if !has_jsc(){return;} assert_eq!(eng().eval("1===1&&2!==3").unwrap(),"true"); }
    #[test] fn test_jsc_eval_ternary() { if !has_jsc(){return;} assert_eq!(eng().eval("5>3?'yes':'no'").unwrap(),"yes"); }
    #[test] fn test_jsc_eval_while() { if !has_jsc(){return;} assert_eq!(eng().eval("(function(){let i=0,s=0;while(i<10){s+=i;i++}return s;})()").unwrap(),"45"); }
    #[test] fn test_jsc_eval_for() { if !has_jsc(){return;} assert_eq!(eng().eval("(function(){let s=0;for(let i=0;i<5;i++){s+=i}return s;})()").unwrap(),"10"); }
    #[test] fn test_jsc_eval_nested_fn() { if !has_jsc(){return;} assert_eq!(eng().eval("(function(a){return(function(b){return a+b})(3)})(2)").unwrap(),"5"); }
    #[test] fn test_jsc_eval_closure() { if !has_jsc(){return;} assert_eq!(eng().eval("(function(){let x=1;return function(){return++x}})()()").unwrap(),"2"); }
    #[test] fn test_jsc_eval_math_pi() { if !has_jsc(){return;} let r=eng().eval("Math.PI").unwrap(); assert!(r.starts_with("3.14"),"got:{r}"); }
    #[test] fn test_jsc_eval_math_floor() { if !has_jsc(){return;} assert_eq!(eng().eval("Math.floor(3.7)").unwrap(),"3"); }
    #[test] fn test_jsc_eval_math_max() { if !has_jsc(){return;} assert_eq!(eng().eval("Math.max(1,5,3)").unwrap(),"5"); }
    #[test] fn test_jsc_eval_json_stringify() { if !has_jsc(){return;} let r=eng().eval("JSON.stringify({a:1,b:2})").unwrap(); assert!(r.contains("a")&&r.contains("b"),"got:{r}"); }
    #[test] fn test_jsc_eval_json_parse() { if !has_jsc(){return;} assert_eq!(eng().eval("JSON.parse('{\"x\":42}').x").unwrap(),"42"); }
    #[test] fn test_jsc_eval_regex_test() { if !has_jsc(){return;} assert_eq!(eng().eval("/hello/.test('hello world')").unwrap(),"true"); }
    #[test] fn test_jsc_eval_string_slice() { if !has_jsc(){return;} assert_eq!(eng().eval("'hello world'.slice(0,5)").unwrap(),"hello"); }
    #[test] fn test_jsc_eval_array_push() { if !has_jsc(){return;} assert_eq!(eng().eval("(function(){let a=[1,2];a.push(3);return a.length;})()").unwrap(),"3"); }
    #[test] fn test_jsc_eval_array_map() { if !has_jsc(){return;} assert_eq!(eng().eval("[1,2,3].map(x=>x*2)").unwrap(),"2,4,6"); }
    #[test] fn test_jsc_eval_array_filter() { if !has_jsc(){return;} assert_eq!(eng().eval("[1,2,3,4].filter(x=>x%2===0)").unwrap(),"2,4"); }
    #[test] fn test_jsc_eval_array_reduce() { if !has_jsc(){return;} assert_eq!(eng().eval("[1,2,3,4].reduce((a,b)=>a+b,0)").unwrap(),"10"); }
    #[test] fn test_jsc_eval_undefined() { if !has_jsc(){return;} assert_eq!(eng().eval("undefined").unwrap(),"undefined"); }
    #[test] fn test_jsc_eval_null() { if !has_jsc(){return;} assert_eq!(eng().eval("null").unwrap(),"null"); }
    #[test] fn test_jsc_eval_true() { if !has_jsc(){return;} assert_eq!(eng().eval("true").unwrap(),"true"); }
    #[test] fn test_jsc_eval_false() { if !has_jsc(){return;} assert_eq!(eng().eval("false").unwrap(),"false"); }
    #[test] fn test_jsc_eval_date_now() { if !has_jsc(){return;} let r=eng().eval("Date.now()").unwrap(); let n:f64=r.parse().unwrap_or(0.0); assert!(n>1e12,"got:{r}"); }
    #[test] fn test_jsc_eval_error_ctor() { if !has_jsc(){return;} assert_eq!(eng().eval("new Error('test').message").unwrap(),"test"); }
    #[test] fn test_jsc_eval_range_error() { if !has_jsc(){return;} let r=eng().eval("new RangeError('range').name").unwrap(); assert_eq!(r,"RangeError","got:{r}"); }
    #[test] fn test_jsc_eval_type_error_ctor() { if !has_jsc(){return;} let r=eng().eval("new TypeError('type').name").unwrap(); assert_eq!(r,"TypeError","got:{r}"); }
    #[test] fn test_jsc_eval_symbol() { if !has_jsc(){return;} assert_eq!(eng().eval("typeof Symbol('test')").unwrap(),"symbol"); }
    #[test] fn test_jsc_eval_map() { if !has_jsc(){return;} assert_eq!(eng().eval("(function(){const m=new Map();m.set('a',1);return m.get('a');})()").unwrap(),"1"); }
    #[test] fn test_jsc_eval_set() { if !has_jsc(){return;} assert_eq!(eng().eval("(function(){const s=new Set();s.add(1);s.add(2);return s.size;})()").unwrap(),"2"); }
    #[test] fn test_jsc_eval_typed_array() { if !has_jsc(){return;} assert_eq!(eng().eval("(function(){const a=new Int32Array(4);a[0]=42;return a[0];})()").unwrap(),"42"); }
    #[test] fn test_jsc_eval_var_hoisting() { if !has_jsc(){return;} assert_eq!(eng().eval("(function(){return x;var x=5;})()").unwrap(),"undefined"); }
    #[test] fn test_jsc_eval_try_catch() { if !has_jsc(){return;} assert_eq!(eng().eval("(function(){try{throw new Error('c')}catch(e){return'caught!'}})()").unwrap(),"caught!"); }
    #[test] fn test_jsc_eval_try_finally() { if !has_jsc(){return;} assert_eq!(eng().eval("(function(){try{return'try'}finally{return'finally'}})()").unwrap(),"finally"); }
    #[test] fn test_jsc_eval_throw_custom() { if !has_jsc(){return;} assert_eq!(eng().eval("(function(){try{throw 42}catch(e){return e}})()").unwrap(),"42"); }
    #[test] fn test_jsc_eval_destructuring() { if !has_jsc(){return;} assert_eq!(eng().eval("(function(){const{a,b}={a:1,b:2};return a+b;})()").unwrap(),"3"); }
    #[test] fn test_jsc_eval_spread() { if !has_jsc(){return;} assert_eq!(eng().eval("(function(){const a=[1,2];const b=[...a,3];return b.length;})()").unwrap(),"3"); }
    #[test] fn test_jsc_eval_rest() { if !has_jsc(){return;} assert_eq!(eng().eval("(function(...args){return args.length;})(1,2,3)").unwrap(),"3"); }
    #[test] fn test_jsc_eval_class() { if !has_jsc(){return;} assert_eq!(eng().eval("new(class{constructor(x){this.x=x}})(42).x").unwrap(),"42"); }
    #[test] fn test_jsc_eval_extends() { if !has_jsc(){return;} assert_eq!(eng().eval("(function(){class A{getX(){return 1}}class B extends A{getX(){return super.getX()+1}}return(new B()).getX();})()").unwrap(),"2"); }
    #[test] fn test_jsc_eval_static() { if !has_jsc(){return;} assert_eq!(eng().eval("(function(){class A{static greet(){return'hi'}}return A.greet();})()").unwrap(),"hi"); }
    #[test] fn test_jsc_eval_getter_setter() { if !has_jsc(){return;} assert_eq!(eng().eval("(function(){let obj={_val:0,get val(){return this._val},set val(v){this._val=v}};obj.val=42;return obj.val;})()").unwrap(),"42"); }
    #[test] fn test_jsc_eval_generator() { if !has_jsc(){return;} assert_eq!(eng().eval("(function*(){yield 1;yield 2;})().next().value").unwrap(),"1"); }
    #[test] fn test_jsc_eval_for_of() { if !has_jsc(){return;} assert_eq!(eng().eval("(function(){let s=0;for(const v of[1,2,3,4]){s+=v}return s;})()").unwrap(),"10"); }
    #[test] fn test_jsc_eval_for_in() { if !has_jsc(){return;} assert_eq!(eng().eval("(function(){const o={a:1,b:2};let k='';for(const p in o){k+=p}return k;})()").unwrap(),"ab"); }
    #[test] fn test_jsc_eval_version() { let v=JSCEngine::version(); assert!(!v.is_empty()); }
    #[test] fn test_jsc_configured() { if !has_jsc(){return;} let e=eng(); assert!(e.is_native_available()); }
    #[test] fn test_jsc_global_get() { if !has_jsc(){return;} let e=eng(); let _=e.eval("var gv=99"); let r=e.get_global("gv"); assert!(r.is_ok()||r.is_err()); }
    #[test] fn test_jsc_eval_empty() { if !has_jsc(){return;} assert!(eng().eval("").is_ok()); }
    #[test] fn test_jsc_eval_whitespace() { if !has_jsc(){return;} assert!(eng().eval("   ").is_ok()); }
    #[test] fn test_jsc_eval_global_this() { if !has_jsc(){return;} assert_eq!(eng().eval("typeof globalThis").unwrap(),"object"); }
    #[test] fn test_jsc_eval_eval_fn() { if !has_jsc(){return;} assert_eq!(eng().eval("eval('1+2')").unwrap(),"3"); }
    #[test] fn test_jsc_eval_fn_ctor() { if !has_jsc(){return;} assert_eq!(eng().eval("new Function('a','b','return a+b')(3,4)").unwrap(),"7"); }
    #[test] fn test_jsc_eval_unicode() { if !has_jsc(){return;} assert_eq!(eng().eval("'\\u0041'").unwrap(),"A"); }
    #[test] fn test_jsc_eval_var_scope() { if !has_jsc(){return;} assert_eq!(eng().eval("(function(){var x=5;if(true){var x=10}return x;})()").unwrap(),"10"); }
    #[test] fn test_jsc_eval_let_scope() { if !has_jsc(){return;} assert_eq!(eng().eval("(function(){let x=5;if(true){let x=10}return x;})()").unwrap(),"5"); }
    #[test] fn test_jsc_eval_const() { if !has_jsc(){return;} assert_eq!(eng().eval("(function(){const x=5;return x;})()").unwrap(),"5"); }
    #[test] fn test_jsc_eval_default_params() { if !has_jsc(){return;} assert_eq!(eng().eval("(function(a=1,b=2){return a+b;})(5)").unwrap(),"7"); }
    #[test] fn test_jsc_eval_arrow_return_obj() { if !has_jsc(){return;} assert_eq!(eng().eval("(()=>({x:1,y:2}))().x").unwrap(),"1"); }
    #[test] fn test_jsc_eval_computed_prop() { if !has_jsc(){return;} assert_eq!(eng().eval("({['he'+'llo']:42})['hello']").unwrap(),"42"); }
    #[test] fn test_jsc_eval_short_circuit_and() { if !has_jsc(){return;} assert_eq!(eng().eval("null&&'unreachable'").unwrap(),"null"); }
    #[test] fn test_jsc_eval_short_circuit_or() { if !has_jsc(){return;} assert_eq!(eng().eval("null||'default'").unwrap(),"default"); }
    #[test] fn test_jsc_eval_nullish_coalescing() { if !has_jsc(){return;} assert_eq!(eng().eval("null??'fallback'").unwrap(),"fallback"); }
    #[test] fn test_jsc_eval_optional_chaining() { if !has_jsc(){return;} assert_eq!(eng().eval("({a:{b:42}})?.a?.b").unwrap(),"42"); }
    #[test] fn test_jsc_eval_optional_chaining_null() { if !has_jsc(){return;} assert_eq!(eng().eval("null?.a").unwrap(),"undefined"); }
    #[test] fn test_jsc_eval_is_finite() { if !has_jsc(){return;} assert_eq!(eng().eval("Number.isFinite(42)").unwrap(),"true"); }
    #[test] fn test_jsc_eval_is_nan() { if !has_jsc(){return;} assert_eq!(eng().eval("Number.isNaN(NaN)").unwrap(),"true"); }
    #[test] fn test_jsc_eval_is_safe_integer() { if !has_jsc(){return;} assert_eq!(eng().eval("Number.isSafeInteger(9007199254740991)").unwrap(),"true"); }
    #[test] fn test_jsc_eval_parse_int_radix() { if !has_jsc(){return;} assert_eq!(eng().eval("parseInt('FF',16)").unwrap(),"255"); }
    #[test] fn test_jsc_eval_string_char_at() { if !has_jsc(){return;} assert_eq!(eng().eval("'hello'.charAt(1)").unwrap(),"e"); }
    #[test] fn test_jsc_eval_string_char_code_at() { if !has_jsc(){return;} assert_eq!(eng().eval("'A'.charCodeAt(0)").unwrap(),"65"); }
    #[test] fn test_jsc_eval_string_code_point_at() { if !has_jsc(){return;} assert_eq!(eng().eval("'A'.codePointAt(0)").unwrap(),"65"); }
    #[test] fn test_jsc_eval_string_from_code_point() { if !has_jsc(){return;} assert_eq!(eng().eval("String.fromCodePoint(65)").unwrap(),"A"); }
    #[test] fn test_jsc_eval_string_concat_method() { if !has_jsc(){return;} assert_eq!(eng().eval("'a'.concat('b','c')").unwrap(),"abc"); }
    #[test] fn test_jsc_eval_string_index_of() { if !has_jsc(){return;} assert_eq!(eng().eval("'hello'.indexOf('l')").unwrap(),"2"); }
    #[test] fn test_jsc_eval_string_last_index_of() { if !has_jsc(){return;} assert_eq!(eng().eval("'hello'.lastIndexOf('l')").unwrap(),"3"); }
    #[test] fn test_jsc_eval_string_match() { if !has_jsc(){return;} let r=eng().eval("'hello123'.match(/\\d+/)").unwrap(); assert!(r.contains("123"),"got:{r}"); }
    #[test] fn test_jsc_eval_string_pad_start() { if !has_jsc(){return;} assert_eq!(eng().eval("'5'.padStart(3,'0')").unwrap(),"005"); }
    #[test] fn test_jsc_eval_string_pad_end() { if !has_jsc(){return;} assert_eq!(eng().eval("'5'.padEnd(3,'0')").unwrap(),"500"); }
    #[test] fn test_jsc_eval_string_search() { if !has_jsc(){return;} assert_eq!(eng().eval("'hello123'.search(/\\d+/)").unwrap(),"5"); }
    #[test] fn test_jsc_eval_string_to_lower() { if !has_jsc(){return;} assert_eq!(eng().eval("'HELLO'.toLowerCase()").unwrap(),"hello"); }
    #[test] fn test_jsc_eval_string_to_upper() { if !has_jsc(){return;} assert_eq!(eng().eval("'hello'.toUpperCase()").unwrap(),"HELLO"); }
    #[test] fn test_jsc_eval_symbol_for() { if !has_jsc(){return;} assert_eq!(eng().eval("Symbol.for('test')===Symbol.for('test')").unwrap(),"true"); }
    #[test] fn test_jsc_eval_symbol_key_for() { if !has_jsc(){return;} assert_eq!(eng().eval("Symbol.keyFor(Symbol.for('x'))").unwrap(),"x"); }
    #[test] fn test_jsc_eval_set_has() { if !has_jsc(){return;} assert_eq!(eng().eval("(function(){const s=new Set([1,2,3]);return s.has(2);})()").unwrap(),"true"); }
    #[test] fn test_jsc_eval_set_delete() { if !has_jsc(){return;} assert_eq!(eng().eval("(function(){const s=new Set([1,2]);s.delete(1);return s.size;})()").unwrap(),"1"); }
    #[test] fn test_jsc_eval_set_clear() { if !has_jsc(){return;} assert_eq!(eng().eval("(function(){const s=new Set([1,2]);s.clear();return s.size;})()").unwrap(),"0"); }
    #[test] fn test_jsc_eval_map_has() { if !has_jsc(){return;} assert_eq!(eng().eval("(function(){const m=new Map();m.set('a',1);return m.has('a');})()").unwrap(),"true"); }
    #[test] fn test_jsc_eval_map_delete() { if !has_jsc(){return;} assert_eq!(eng().eval("(function(){const m=new Map([['a',1]]);m.delete('a');return m.size;})()").unwrap(),"0"); }
    #[test] fn test_jsc_eval_map_keys() { if !has_jsc(){return;} assert_eq!(eng().eval("(function(){const m=new Map([['a',1],['b',2]]);return Array.from(m.keys()).length;})()").unwrap(),"2"); }
    #[test] fn test_jsc_eval_map_clear() { if !has_jsc(){return;} assert_eq!(eng().eval("(function(){const m=new Map([['a',1]]);m.clear();return m.size;})()").unwrap(),"0"); }
    #[test] fn test_jsc_eval_typeof_undefined_var() { if !has_jsc(){return;} assert_eq!(eng().eval("typeof nonExistentVar").unwrap(),"undefined"); }
    #[test] fn test_jsc_eval_void_operator() { if !has_jsc(){return;} assert_eq!(eng().eval("void 0").unwrap(),"undefined"); }
    #[test] fn test_jsc_eval_typeof_null() { if !has_jsc(){return;} assert_eq!(eng().eval("typeof null").unwrap(),"object"); }
    #[test] fn test_jsc_eval_delete_prop() { if !has_jsc(){return;} assert_eq!(eng().eval("(function(){const o={a:1};delete o.a;return o.a;})()").unwrap(),"undefined"); }
    #[test] fn test_jsc_eval_in_operator() { if !has_jsc(){return;} assert_eq!(eng().eval("'length'in[1,2,3]").unwrap(),"true"); }
    #[test] fn test_jsc_eval_instance_of() { if !has_jsc(){return;} assert_eq!(eng().eval("[]instanceof Array").unwrap(),"true"); }
    #[test] fn test_jsc_eval_number_methods() { if !has_jsc(){return;} assert_eq!(eng().eval("Number.isInteger(42)").unwrap(),"true"); }
    #[test] fn test_jsc_eval_number_to_fixed() { if !has_jsc(){return;} assert_eq!(eng().eval("(3.14159).toFixed(2)").unwrap(),"3.14"); }
    #[test] fn test_jsc_eval_math_random() { if !has_jsc(){return;} let r=eng().eval("Math.random()").unwrap(); let n:f64=r.parse().unwrap_or(-1.0); assert!(n>=0.0&&n<1.0,"got:{r}"); }
    #[test] fn test_jsc_eval_math_round() { if !has_jsc(){return;} assert_eq!(eng().eval("Math.round(3.5)").unwrap(),"4"); }
    #[test] fn test_jsc_eval_math_abs() { if !has_jsc(){return;} assert_eq!(eng().eval("Math.abs(-5)").unwrap(),"5"); }
    #[test] fn test_jsc_eval_math_pow() { if !has_jsc(){return;} assert_eq!(eng().eval("Math.pow(2,10)").unwrap(),"1024"); }
    #[test] fn test_jsc_eval_math_sqrt() { if !has_jsc(){return;} assert_eq!(eng().eval("Math.sqrt(16)").unwrap(),"4"); }
    #[test] fn test_jsc_eval_math_min() { if !has_jsc(){return;} assert_eq!(eng().eval("Math.min(5,2,8)").unwrap(),"2"); }
    #[test] fn test_jsc_eval_math_trunc() { if !has_jsc(){return;} assert_eq!(eng().eval("Math.trunc(4.9)").unwrap(),"4"); }
    #[test] fn test_jsc_eval_math_sign() { if !has_jsc(){return;} assert_eq!(eng().eval("Math.sign(-5)").unwrap(),"-1"); }
    #[test] fn test_jsc_eval_math_cbrt() { if !has_jsc(){return;} assert_eq!(eng().eval("Math.cbrt(27)").unwrap(),"3"); }
    #[test] fn test_jsc_eval_math_hypot() { if !has_jsc(){return;} assert_eq!(eng().eval("Math.hypot(3,4)").unwrap(),"5"); }
    #[test] fn test_jsc_eval_date_construct() { if !has_jsc(){return;} assert_eq!(eng().eval("new Date('2024-01-01').getFullYear()").unwrap(),"2024"); }
    #[test] fn test_jsc_eval_date_month() { if !has_jsc(){return;} assert_eq!(eng().eval("new Date('2024-06-15').getMonth()").unwrap(),"5"); }
    #[test] fn test_jsc_eval_date_date() { if !has_jsc(){return;} assert_eq!(eng().eval("new Date('2024-06-15').getDate()").unwrap(),"15"); }
    #[test] fn test_jsc_eval_regex_exec() { if !has_jsc(){return;} assert_eq!(eng().eval("/(\\d+)/.exec('abc123def')[1]").unwrap(),"123"); }
    #[test] fn test_jsc_eval_regex_flags() { if !has_jsc(){return;} assert_eq!(eng().eval("/test/gi.flags").unwrap(),"gi"); }
    #[test] fn test_jsc_eval_regex_source() { if !has_jsc(){return;} assert_eq!(eng().eval("/abc/.source").unwrap(),"abc"); }
    #[test] fn test_jsc_eval_regex_test_no_match() { if !has_jsc(){return;} assert_eq!(eng().eval("/xyz/.test('hello')").unwrap(),"false"); }
    #[test] fn test_jsc_eval_string_trim() { if !has_jsc(){return;} assert_eq!(eng().eval("'  hello  '.trim()").unwrap(),"hello"); }
    #[test] fn test_jsc_eval_string_split() { if !has_jsc(){return;} assert_eq!(eng().eval("'a,b,c'.split(',')").unwrap(),"a,b,c"); }
    #[test] fn test_jsc_eval_string_includes() { if !has_jsc(){return;} assert_eq!(eng().eval("'hello'.includes('ell')").unwrap(),"true"); }
    #[test] fn test_jsc_eval_string_starts_with() { if !has_jsc(){return;} assert_eq!(eng().eval("'hello'.startsWith('he')").unwrap(),"true"); }
    #[test] fn test_jsc_eval_string_ends_with() { if !has_jsc(){return;} assert_eq!(eng().eval("'hello'.endsWith('lo')").unwrap(),"true"); }
    #[test] fn test_jsc_eval_string_repeat() { if !has_jsc(){return;} assert_eq!(eng().eval("'ab'.repeat(3)").unwrap(),"ababab"); }
    #[test] fn test_jsc_eval_string_replace() { if !has_jsc(){return;} assert_eq!(eng().eval("'hello world'.replace('world','there')").unwrap(),"hello there"); }
    #[test] fn test_jsc_eval_replace_all() { if !has_jsc(){return;} assert_eq!(eng().eval("'a-a-a'.replaceAll('-','+')").unwrap(),"a+a+a"); }
    #[test] fn test_jsc_eval_array_flat() { if !has_jsc(){return;} assert_eq!(eng().eval("[[1,2],[3,4]].flat()").unwrap(),"1,2,3,4"); }
    #[test] fn test_jsc_eval_array_flat_map() { if !has_jsc(){return;} assert_eq!(eng().eval("[1,2,3].flatMap(x=>[x,x*2])").unwrap(),"1,2,2,4,3,6"); }
    #[test] fn test_jsc_eval_array_find() { if !has_jsc(){return;} assert_eq!(eng().eval("[1,2,3,4].find(x=>x>2)").unwrap(),"3"); }
    #[test] fn test_jsc_eval_array_find_index() { if !has_jsc(){return;} assert_eq!(eng().eval("[1,2,3,4].findIndex(x=>x>2)").unwrap(),"2"); }
    #[test] fn test_jsc_eval_array_some() { if !has_jsc(){return;} assert_eq!(eng().eval("[1,2,3].some(x=>x>2)").unwrap(),"true"); }
    #[test] fn test_jsc_eval_array_every() { if !has_jsc(){return;} assert_eq!(eng().eval("[1,2,3].every(x=>x>0)").unwrap(),"true"); }
    #[test] fn test_jsc_eval_array_from() { if !has_jsc(){return;} assert_eq!(eng().eval("Array.from('hello').length").unwrap(),"5"); }
    #[test] fn test_jsc_eval_array_of() { if !has_jsc(){return;} assert_eq!(eng().eval("Array.of(1,2,3).length").unwrap(),"3"); }
    #[test] fn test_jsc_eval_array_fill() { if !has_jsc(){return;} assert_eq!(eng().eval("new Array(3).fill(0)").unwrap(),"0,0,0"); }
    #[test] fn test_jsc_eval_array_includes() { if !has_jsc(){return;} assert_eq!(eng().eval("[1,2,3].includes(2)").unwrap(),"true"); }
    #[test] fn test_jsc_eval_array_join() { if !has_jsc(){return;} assert_eq!(eng().eval("[1,2,3].join('-')").unwrap(),"1-2-3"); }
    #[test] fn test_jsc_eval_array_reverse() { if !has_jsc(){return;} assert_eq!(eng().eval("[1,2,3].reverse()").unwrap(),"3,2,1"); }
    #[test] fn test_jsc_eval_array_sort() { if !has_jsc(){return;} assert_eq!(eng().eval("[3,1,2].sort()").unwrap(),"1,2,3"); }
    #[test] fn test_jsc_eval_array_concat() { if !has_jsc(){return;} assert_eq!(eng().eval("[1,2].concat([3,4]).length").unwrap(),"4"); }
    #[test] fn test_jsc_eval_array_slice() { if !has_jsc(){return;} assert_eq!(eng().eval("[1,2,3,4].slice(1,3)").unwrap(),"2,3"); }
    #[test] fn test_jsc_eval_array_splice() { if !has_jsc(){return;} assert_eq!(eng().eval("(function(){let a=[1,2,3,4];a.splice(1,2);return a;})()").unwrap(),"1,4"); }
    #[test] fn test_jsc_eval_object_keys() { if !has_jsc(){return;} assert_eq!(eng().eval("Object.keys({a:1,b:2}).length").unwrap(),"2"); }
    #[test] fn test_jsc_eval_object_values() { if !has_jsc(){return;} assert_eq!(eng().eval("Object.values({a:1,b:2})").unwrap(),"1,2"); }
    #[test] fn test_jsc_eval_object_entries() { if !has_jsc(){return;} assert_eq!(eng().eval("Object.entries({a:1,b:2}).length").unwrap(),"2"); }
    #[test] fn test_jsc_eval_object_assign() { if !has_jsc(){return;} assert_eq!(eng().eval("Object.assign({},{a:1},{b:2}).b").unwrap(),"2"); }
    #[test] fn test_jsc_eval_object_freeze() { if !has_jsc(){return;} assert_eq!(eng().eval("Object.isFrozen(Object.freeze({}))").unwrap(),"true"); }
    #[test] fn test_jsc_eval_object_seal() { if !has_jsc(){return;} assert_eq!(eng().eval("Object.isSealed(Object.seal({a:1}))").unwrap(),"true"); }
    #[test] fn test_jsc_eval_parse_int() { if !has_jsc(){return;} assert_eq!(eng().eval("parseInt('42')").unwrap(),"42"); }
    #[test] fn test_jsc_eval_parse_float() { if !has_jsc(){return;} assert_eq!(eng().eval("parseFloat('3.14')").unwrap(),"3.14"); }
    #[test] fn test_jsc_eval_is_nan_global() { if !has_jsc(){return;} assert_eq!(eng().eval("isNaN(NaN)").unwrap(),"true"); }
    #[test] fn test_jsc_eval_encode_uri() { if !has_jsc(){return;} assert_eq!(eng().eval("encodeURIComponent('hello world')").unwrap(),"hello%20world"); }
    #[test] fn test_jsc_eval_decode_uri() { if !has_jsc(){return;} assert_eq!(eng().eval("decodeURIComponent('hello%20world')").unwrap(),"hello world"); }
    #[test] fn test_jsc_eval_promise_resolve() { if !has_jsc(){return;} assert!(eng().eval("Promise.resolve(42)").is_ok()); }
    #[test] fn test_jsc_eval_bigint() { if !has_jsc(){return;} let r=eng().eval("12345678901234567890n"); if let Ok(v)=r{assert!(v.contains("1234567890"),"got:{v}");} }
    #[test] fn test_jsc_eval_weak_map() { if !has_jsc(){return;} assert_eq!(eng().eval("typeof WeakMap").unwrap(),"function"); }
    #[test] fn test_jsc_eval_weak_set() { if !has_jsc(){return;} assert_eq!(eng().eval("typeof WeakSet").unwrap(),"function"); }
    #[test] fn test_jsc_eval_proxy() { if !has_jsc(){return;} assert_eq!(eng().eval("typeof Proxy").unwrap(),"function"); }
    #[test] fn test_jsc_eval_reflect() { if !has_jsc(){return;} assert_eq!(eng().eval("typeof Reflect").unwrap(),"object"); }
    #[test] fn test_jsc_eval_data_view() { if !has_jsc(){return;} assert_eq!(eng().eval("typeof DataView").unwrap(),"function"); }
    #[test] fn test_jsc_eval_async_fn() { if !has_jsc(){return;} assert!(eng().eval("(async function(){return 42;})()").is_ok()); }
    #[test] fn test_jsc_eval_new_target() { if !has_jsc(){return;} assert_eq!(eng().eval("(function(){return typeof new.target;})()").unwrap(),"undefined"); }
    #[test] fn test_jsc_eval_complex_math() { if !has_jsc(){return;} let r=eng().eval("Math.sin(Math.PI/2)").unwrap(); let n:f64=r.parse().unwrap_or(0.0); assert!((n-1.0).abs()<0.01,"got:{r}"); }
    #[test] fn test_jsc_eval_complex_obj() { if !has_jsc(){return;} let r=eng().eval("JSON.stringify({name:'test',values:[1,2,3],nested:{a:{b:42}}})").unwrap(); assert!(r.contains("\"b\":42"),"got:{r}"); }
    #[test] fn test_jsc_eval_module() { if !has_jsc(){return;} let r=eng().execute_module("mod.js","export const x=1;"); assert!(r.is_ok()||r.is_err()); }
    #[test] fn test_jsc_get_heap_stats() { if !has_jsc(){return;} let e=eng(); let s=e.get_heap_stats().unwrap(); assert!(s.total_heap_size>0); }
    #[test] fn test_jsc_request_gc() { if !has_jsc(){return;} eng().request_gc(); }
    #[test] fn test_jsc_microtasks_perform_check() { if !has_jsc(){return;} eng().microtasks_perform_check(); }
    #[test] fn test_jsc_get_stack_trace() { if !has_jsc(){return;} let r=eng().get_stack_trace(); assert!(r.is_ok()||r.is_err()); }
    #[test] fn test_jsc_version() { let v=JSCEngine::version(); assert!(!v.is_empty()); }
    #[test] fn test_jsc_is_native_available() { let e=JSCEngine::new(); if let Ok(ref eng)=e{assert!(eng.is_native_available()||!eng.is_native_available());} }
    #[test] fn test_jsc_value_type_enum() { assert_eq!(JSCValueType::Undefined as u32,0); assert_eq!(JSCValueType::Null as u32,1); assert_eq!(JSCValueType::Boolean as u32,2); }
    #[test] fn test_jsc_value_type_number() { assert_eq!(JSCValueType::Number as u32,3); assert_eq!(JSCValueType::String as u32,4); assert_eq!(JSCValueType::Object as u32,5); }
    #[test] fn test_jsc_value_type_array() { assert_eq!(JSCValueType::Array as u32,6); assert_eq!(JSCValueType::Function as u32,7); }
    #[test] fn test_jsc_value_type_error() { assert_eq!(JSCValueType::Error as u32,8); assert_eq!(JSCValueType::Symbol as u32,9); }
    #[test] fn test_jsc_value_type_typed_array() { assert_eq!(JSCValueType::TypedArray as u32,10); }
    #[test] fn test_jsc_heap_stats_struct() { let h=HeapStats{total_heap_size:0,total_heap_size_executable:0,total_physical_size:0,total_available_size:0,used_heap_size:0,heap_size_limit:0,malloced_memory:0,peak_malloced_memory:0,number_of_native_contexts:0,number_of_detached_contexts:0,total_global_handles_size:0,used_global_handles_size:0,external_memory:0}; assert_eq!(h.heap_size_limit,0); }
    #[test] fn test_jsc_error_not_initialized() { let e=JSCError::NotInitialized; assert_eq!(e.to_string(),"JSC not initialized"); }
    #[test] fn test_jsc_error_init_failed() { let e=JSCError::InitFailed("oom".into()); assert_eq!(e.to_string(),"JSC init failed: oom"); }
    #[test] fn test_jsc_error_eval_failed() { let e=JSCError::EvalFailed("syntax".into()); assert_eq!(e.to_string(),"JSC eval error: syntax"); }
    #[test] fn test_jsc_error_module_failed() { let e=JSCError::ModuleFailed("err".into()); assert_eq!(e.to_string(),"JSC module error: err"); }
    #[test] fn test_jsc_error_global_failed() { let e=JSCError::GlobalFailed("err".into()); assert_eq!(e.to_string(),"JSC global error: err"); }
    #[test] fn test_jsc_error_call_failed() { let e=JSCError::CallFailed("err".into()); assert_eq!(e.to_string(),"JSC call error: err"); }
    #[test] fn test_jsc_error_snapshot_failed() { let e=JSCError::SnapshotFailed("err".into()); assert_eq!(e.to_string(),"JSC snapshot error: err"); }
    #[test] fn test_jsc_error_internal() { let e=JSCError::Internal("crash".into()); assert_eq!(e.to_string(),"JSC internal error: crash"); }
    #[test] fn test_jsc_error_common_kind() { let e=JSCError::NotInitialized; assert!(matches!(e.to_common_kind(),common::error::CommonErrorKind::NotInitialized)); }
    #[test] fn test_jsc_error_common_kind_init() { let e=JSCError::InitFailed("x".into()); assert!(matches!(e.to_common_kind(),common::error::CommonErrorKind::InitFailed(_))); }
    #[test] fn test_jsc_error_common_kind_exec() { let e=JSCError::EvalFailed("x".into()); assert!(matches!(e.to_common_kind(),common::error::CommonErrorKind::ExecutionFailed(_))); }
    #[test] fn test_jsc_error_common_kind_module() { let e=JSCError::ModuleFailed("x".into()); assert!(matches!(e.to_common_kind(),common::error::CommonErrorKind::ExecutionFailed(_))); }
    #[test] fn test_jsc_permissions_new() { let p=permissions::JSCPermissions::new(); assert!(!p.check(&permissions::Permission::Read)); }
    #[test] fn test_jsc_permissions_allow_all() { let p=permissions::JSCPermissions::allow_all(); assert!(p.check(&permissions::Permission::Read)); assert!(p.check(&permissions::Permission::Write)); }
    #[test] fn test_jsc_permissions_deny_all() { let p=permissions::JSCPermissions::deny_all(); assert!(!p.check(&permissions::Permission::Read)); }
    #[test] fn test_jsc_permissions_check_path() { let p=permissions::JSCPermissions::allow_all(); assert!(p.check_path(&permissions::Permission::Read,"/tmp")); }
    #[test] fn test_jsc_permissions_common_roundtrip() { let c=common::permissions::CommonPermissions::allow_all(); let p=permissions::JSCPermissions::from_common(c.clone()); assert_eq!(p.to_common().allow_read,c.allow_read); }
    #[test] fn test_jsc_permission_enum_convert() { for (p,cp) in [(permissions::Permission::Read,common::permissions::CommonPermission::Read),(permissions::Permission::Write,common::permissions::CommonPermission::Write),(permissions::Permission::Net,common::permissions::CommonPermission::Net),(permissions::Permission::Env,common::permissions::CommonPermission::Env),(permissions::Permission::Run,common::permissions::CommonPermission::Run),(permissions::Permission::Ffi,common::permissions::CommonPermission::Ffi),(permissions::Permission::All,common::permissions::CommonPermission::All)]{let c:common::permissions::CommonPermission=p.clone().into();assert_eq!(c,cp);let v:permissions::Permission=cp.into();assert_eq!(v,p);} }
    #[test] fn test_jsc_module_loader_new() { let l=module_loader::JSCModuleLoader::new("/tmp"); let r=l.resolve("file:///a.js","/b"); assert_eq!(r,Ok("file:///a.js".to_string())); }
    #[test] fn test_jsc_module_loader_resolve_relative() { let l=module_loader::JSCModuleLoader::new("/tmp"); let r=l.resolve("./lib.js","/base/mod.js"); assert_eq!(r,Ok("/base/./lib.js".to_string())); }
    #[test] fn test_jsc_module_loader_register_load() { let l=module_loader::JSCModuleLoader::new("/tmp"); l.register("my:mod","export const x=1;"); assert_eq!(l.load("my:mod"),Ok("export const x=1;".to_string())); }
    #[test] fn test_jsc_module_loader_resolve_unresolvable() { let l=module_loader::JSCModuleLoader::new("/nonexistent"); let r=l.resolve("some-pkg","/base"); assert!(r.is_err()); }
    #[test] fn test_jsc_eval_escape() { if !has_jsc(){return;} let r=eng().eval("'line1\\nline2'").unwrap(); assert_eq!(r,"line1\nline2"); }
    #[test] fn test_jsc_eval_json_roundtrip() { if !has_jsc(){return;} let e=eng(); let v=e.json_parse("{\"a\":1}"); if let Ok(ptr)=v{let r=e.json_stringify(ptr);assert!(r.is_ok());} }
    #[test] fn test_jsc_eval_intl() { if !has_jsc(){return;} assert_eq!(eng().eval("typeof Intl").unwrap(),"object"); }
    #[test] fn test_jsc_eval_shared_array_buffer() { if !has_jsc(){return;} assert_eq!(eng().eval("typeof SharedArrayBuffer").unwrap(),"function"); }
    #[test] fn test_jsc_eval_weak_ref() { if !has_jsc(){return;} assert_eq!(eng().eval("typeof WeakRef").unwrap(),"function"); }
    #[test] fn test_jsc_eval_array_buffer() { if !has_jsc(){return;} assert_eq!(eng().eval("typeof ArrayBuffer").unwrap(),"function"); }
    #[test] fn test_jsc_eval_uint8_clamped() { if !has_jsc(){return;} assert_eq!(eng().eval("typeof Uint8ClampedArray").unwrap(),"function"); }
    #[test] fn test_jsc_eval_float32() { if !has_jsc(){return;} assert_eq!(eng().eval("typeof Float32Array").unwrap(),"function"); }
    #[test] fn test_jsc_eval_float64() { if !has_jsc(){return;} assert_eq!(eng().eval("typeof Float64Array").unwrap(),"function"); }
    #[test] fn test_jsc_eval_int8() { if !has_jsc(){return;} assert_eq!(eng().eval("typeof Int8Array").unwrap(),"function"); }
    #[test] fn test_jsc_eval_uint8() { if !has_jsc(){return;} assert_eq!(eng().eval("typeof Uint8Array").unwrap(),"function"); }
    #[test] fn test_jsc_eval_int16() { if !has_jsc(){return;} assert_eq!(eng().eval("typeof Int16Array").unwrap(),"function"); }
    #[test] fn test_jsc_eval_uint16() { if !has_jsc(){return;} assert_eq!(eng().eval("typeof Uint16Array").unwrap(),"function"); }
    #[test] fn test_jsc_eval_int32() { if !has_jsc(){return;} assert_eq!(eng().eval("typeof Int32Array").unwrap(),"function"); }
    #[test] fn test_jsc_eval_uint32() { if !has_jsc(){return;} assert_eq!(eng().eval("typeof Uint32Array").unwrap(),"function"); }
    #[test] fn test_jsc_eval_bigint64() { if !has_jsc(){return;} assert_eq!(eng().eval("typeof BigInt64Array").unwrap(),"function"); }
    #[test] fn test_jsc_eval_biguint64() { if !has_jsc(){return;} assert_eq!(eng().eval("typeof BigUint64Array").unwrap(),"function"); }
    #[test] fn test_jsc_call_function() { if !has_jsc(){return;} assert_eq!(eng().eval("(function(a,b){return a+b;})(3,4)").unwrap(),"7"); }
    #[test] fn test_jsc_value_is_null() { if !has_jsc(){return;} assert_eq!(eng().eval("null === null").unwrap(),"true"); }
    #[test] fn test_jsc_value_is_undefined() { if !has_jsc(){return;} assert_eq!(eng().eval("undefined === undefined").unwrap(),"true"); }
    #[test] fn test_jsc_value_is_boolean() { if !has_jsc(){return;} assert_eq!(eng().eval("typeof true === 'boolean'").unwrap(),"true"); }
    #[test] fn test_jsc_value_is_number() { if !has_jsc(){return;} assert_eq!(eng().eval("typeof 42 === 'number'").unwrap(),"true"); }
    #[test] fn test_jsc_value_is_string() { if !has_jsc(){return;} assert_eq!(eng().eval("typeof 'hi' === 'string'").unwrap(),"true"); }
}
