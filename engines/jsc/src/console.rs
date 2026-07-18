use std::collections::HashMap;
use std::sync::Mutex;

use crate::JSCEngine;

pub struct JSCConsole {
    counts: Mutex<HashMap<String, u64>>,
    timers: Mutex<HashMap<String, std::time::Instant>>,
}

impl JSCConsole {
    pub fn new() -> Self {
        Self {
            counts: Mutex::new(HashMap::new()),
            timers: Mutex::new(HashMap::new()),
        }
    }

    pub fn log(&self, _engine: &JSCEngine, _args: &[*const std::ffi::c_void]) {
        #[cfg(feature = "native")]
        crate::ffi::console_log(_engine.raw_handle(), _args);
        #[cfg(not(feature = "native"))]
        eprintln!("[log]");
    }

    pub fn warn(&self, _engine: &JSCEngine, _args: &[*const std::ffi::c_void]) {
        #[cfg(feature = "native")]
        crate::ffi::console_warn(_engine.raw_handle(), _args);
        #[cfg(not(feature = "native"))]
        eprintln!("[warn]");
    }

    pub fn error(&self, _engine: &JSCEngine, _args: &[*const std::ffi::c_void]) {
        #[cfg(feature = "native")]
        crate::ffi::console_error(_engine.raw_handle(), _args);
        #[cfg(not(feature = "native"))]
        eprintln!("[error]");
    }

    pub fn info(&self, _engine: &JSCEngine, _args: &[*const std::ffi::c_void]) {
        #[cfg(feature = "native")]
        crate::ffi::console_info(_engine.raw_handle(), _args);
        #[cfg(not(feature = "native"))]
        eprintln!("[info]");
    }

    pub fn debug(&self, _engine: &JSCEngine, _args: &[*const std::ffi::c_void]) {
        #[cfg(feature = "native")]
        crate::ffi::console_debug(_engine.raw_handle(), _args);
        #[cfg(not(feature = "native"))]
        eprintln!("[debug]");
    }

    pub fn table(&self, _engine: &JSCEngine, _data: *const std::ffi::c_void) {
        #[cfg(feature = "native")]
        crate::ffi::console_table(_engine.raw_handle(), _data as *mut crate::ffi::JSCValueHandle);
        #[cfg(not(feature = "native"))]
        eprintln!("[table]");
    }

    pub fn assert(&self, _engine: &JSCEngine, _condition: bool, _args: &[*const std::ffi::c_void]) {
        #[cfg(feature = "native")]
        crate::ffi::console_assert(_engine.raw_handle(), _condition, _args);
        #[cfg(not(feature = "native"))]
        {}
    }

    pub fn count(&self, _engine: &JSCEngine, _label: Option<&str>) {
        let lbl = _label.unwrap_or("default").to_string();
        let mut counts = self.counts.lock().unwrap();
        let entry = counts.entry(lbl.clone()).or_insert(0);
        *entry += 1;
        eprintln!("[count] {}: {}", lbl, *entry);
    }

    pub fn time(&self, _engine: &JSCEngine, label: Option<&str>) {
        let lbl = label.unwrap_or("default").to_string();
        let mut timers = self.timers.lock().unwrap();
        timers.insert(lbl, std::time::Instant::now());
    }

    pub fn time_end(&self, _engine: &JSCEngine, label: Option<&str>) {
        let lbl = label.unwrap_or("default").to_string();
        let mut timers = self.timers.lock().unwrap();
        if let Some(start) = timers.remove(&lbl) {
            let elapsed = start.elapsed();
            eprintln!("[timeEnd] {}: {:?}", lbl, elapsed);
        } else {
            eprintln!("[timeEnd] {}: timer not found", lbl);
        }
    }

    pub fn trace(&self, _engine: &JSCEngine) {
        let backtrace = std::backtrace::Backtrace::force_capture();
        eprintln!("[trace]\n{:?}", backtrace);
    }
}

impl Default for JSCConsole {
    fn default() -> Self {
        Self::new()
    }
}
