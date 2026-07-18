use boa_engine::{Context, JsValue, NativeFunction, JsResult, js_string};
use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// Wrapper to make JsValue Send/Sync. Boa is single-threaded so this is safe.
struct SendJsValue(JsValue);
unsafe impl Send for SendJsValue {}
unsafe impl Sync for SendJsValue {}

struct TimerData {
    #[allow(dead_code)]
    id: u64,
    callback: SendJsValue,
    interval_ms: u64,
    repeats: bool,
}

static TIMERS: LazyLock<Mutex<HashMap<u64, TimerData>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

fn next_id() -> u64 {
    NEXT_ID.fetch_add(1, Ordering::SeqCst)
}

fn set_timeout_fn(_this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let callback = args.first().cloned().unwrap_or(JsValue::undefined());
    let delay = args.get(1).and_then(|v| v.as_number()).unwrap_or(0.0) as u64;
    let id = next_id();
    let mut timers = TIMERS.lock().unwrap();
    timers.insert(id, TimerData { id, callback: SendJsValue(callback), interval_ms: delay, repeats: false });
    Ok(JsValue::from(id as f64))
}

fn set_interval_fn(_this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let callback = args.first().cloned().unwrap_or(JsValue::undefined());
    let delay = args.get(1).and_then(|v| v.as_number()).unwrap_or(0.0) as u64;
    let id = next_id();
    let mut timers = TIMERS.lock().unwrap();
    timers.insert(id, TimerData { id, callback: SendJsValue(callback), interval_ms: delay, repeats: true });
    Ok(JsValue::from(id as f64))
}

fn clear_timer_fn(_this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    if let Some(id_val) = args.first().and_then(|v| v.as_number()) {
        let mut timers = TIMERS.lock().unwrap();
        timers.remove(&(id_val as u64));
    }
    Ok(JsValue::undefined())
}

fn set_immediate_fn(_this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let callback = args.first().cloned().unwrap_or(JsValue::undefined());
    let id = next_id();
    let mut timers = TIMERS.lock().unwrap();
    timers.insert(id, TimerData { id, callback: SendJsValue(callback), interval_ms: 0, repeats: false });
    Ok(JsValue::from(id as f64))
}

pub struct Timers;

impl Timers {
    pub fn register(context: &mut Context) -> Result<(), crate::BoaError> {
        context.register_global_builtin_callable(
            js_string!("setTimeout"), 2usize, NativeFunction::from_fn_ptr(set_timeout_fn),
        ).map_err(|e| crate::BoaError::from_js_error(&e))?;

        context.register_global_builtin_callable(
            js_string!("setInterval"), 2usize, NativeFunction::from_fn_ptr(set_interval_fn),
        ).map_err(|e| crate::BoaError::from_js_error(&e))?;

        context.register_global_builtin_callable(
            js_string!("clearTimeout"), 1usize, NativeFunction::from_fn_ptr(clear_timer_fn),
        ).map_err(|e| crate::BoaError::from_js_error(&e))?;

        context.register_global_builtin_callable(
            js_string!("clearInterval"), 1usize, NativeFunction::from_fn_ptr(clear_timer_fn),
        ).map_err(|e| crate::BoaError::from_js_error(&e))?;

        context.register_global_builtin_callable(
            js_string!("setImmediate"), 1usize, NativeFunction::from_fn_ptr(set_immediate_fn),
        ).map_err(|e| crate::BoaError::from_js_error(&e))?;

        context.register_global_builtin_callable(
            js_string!("clearImmediate"), 1usize, NativeFunction::from_fn_ptr(clear_timer_fn),
        ).map_err(|e| crate::BoaError::from_js_error(&e))?;

        Ok(())
    }

    pub fn process_timers(context: &mut Context) {
        let to_process: Vec<u64> = {
            let timers = TIMERS.lock().unwrap();
            timers.iter()
                .filter(|(_, data)| data.interval_ms == 0)
                .map(|(id, _)| *id)
                .collect()
        };

        for id in &to_process {
            let callback = {
                let timers = TIMERS.lock().unwrap();
                timers.get(id).map(|data| data.callback.0.clone())
            };
            if let Some(cb) = callback {
                if let Some(obj) = cb.as_object() {
                    let _ = obj.call(&JsValue::undefined(), &[], context);
                }
            }
        }

        {
            let mut timers = TIMERS.lock().unwrap();
            timers.retain(|id, data| {
                if to_process.contains(id) && !data.repeats {
                    false
                } else {
                    true
                }
            });
        }
    }

    pub fn clear_all() {
        let mut timers = TIMERS.lock().unwrap();
        timers.clear();
    }
}
