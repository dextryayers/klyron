use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use parking_lot::Mutex;

use crate::engine::JsEngineKind;
use crate::engine_pool::EnginePool;

const COMMON_POLYFILLS: &[(&str, &str)] = &[
    ("console", r#"var console={log:function(){},error:function(){},warn:function(){},info:function(){},debug:function(){}};"#),
    ("timers", r#"var setTimeout=function(f,n){if(typeof f==='function')f();};var clearTimeout=function(){};var setInterval=function(f,n){if(typeof f==='function')f();};var clearInterval=function(){};"#),
    ("json", r#"var JSON={parse:function(s){try{return JSON.parse(s)}catch(e){return null}},stringify:function(o){try{return JSON.stringify(o)}catch(e){return String(o)}}};"#),
    ("fetch_polyfill", r#"var fetch=function(url){return new Promise(function(resolve){resolve({ok:true,status:200,json:function(){return Promise.resolve({}),text:function(){return Promise.resolve('')}}});});};"#),
    ("promise", r#"if(typeof Promise==='undefined'){var Promise=function(f){var cbs=[];f(function(r){cbs.forEach(function(cb){cb(r);})},function(){})};}"#),
];

pub struct EnginePreWarmer {
    ready: Arc<AtomicBool>,
    pool: Arc<Mutex<Option<EnginePool>>>,
    engine_kind: JsEngineKind,
}

impl EnginePreWarmer {
    pub fn new(engine_kind: JsEngineKind) -> Self {
        Self {
            ready: Arc::new(AtomicBool::new(false)),
            pool: Arc::new(Mutex::new(None)),
            engine_kind,
        }
    }

    pub fn start_background(&self, pool_size: usize) {
        let ready = self.ready.clone();
        let pool_arc = self.pool.clone();
        let kind = self.engine_kind;

        thread::spawn(move || {
            let pool = EnginePool::new(kind, pool_size, pool_size * 2);
            pool.warmup(pool_size);
            pool.pre_compile_scripts(COMMON_POLYFILLS);
            *pool_arc.lock() = Some(pool);
            ready.store(true, Ordering::SeqCst);
        });
    }

    pub fn start_blocking(&self, pool_size: usize) -> EnginePool {
        let pool = EnginePool::new(self.engine_kind, pool_size, pool_size * 2);
        pool.warmup(pool_size);
        pool.pre_compile_scripts(COMMON_POLYFILLS);
        *self.pool.lock() = Some(pool.clone());
        self.ready.store(true, Ordering::SeqCst);
        pool
    }

    pub fn is_ready(&self) -> bool {
        self.ready.load(Ordering::SeqCst)
    }

    pub fn pool(&self) -> Option<EnginePool> {
        self.pool.lock().clone()
    }

    pub fn wait_ready(&self, timeout: std::time::Duration) -> bool {
        let start = std::time::Instant::now();
        while !self.is_ready() {
            if start.elapsed() > timeout {
                return false;
            }
            thread::sleep(std::time::Duration::from_millis(10));
        }
        true
    }
}

pub fn default_pre_warm_scripts() -> Vec<(&'static str, &'static str)> {
    COMMON_POLYFILLS.to_vec()
}
