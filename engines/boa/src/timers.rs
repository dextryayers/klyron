use boa_engine::{Context, JsValue, NativeFunction, JsResult, js_string};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

#[allow(dead_code)]
struct TimerData {
    id: u64,
    interval_ms: u64,
    repeats: bool,
}

static TIMERS: Mutex<Option<HashMap<u64, TimerData>>> = Mutex::new(None);

fn ensure_timers() -> std::sync::MutexGuard<'static, Option<HashMap<u64, TimerData>>> {
    let mut guard = TIMERS.lock().unwrap();
    if guard.is_none() {
        *guard = Some(HashMap::new());
    }
    guard
}

fn next_id() -> u64 {
    NEXT_ID.fetch_add(1, Ordering::SeqCst)
}

fn set_timeout_fn(_this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let delay = args.get(1).and_then(|v| v.as_number()).unwrap_or(0.0) as u64;
    let id = next_id();
    let mut timers = ensure_timers();
    if let Some(ref mut map) = *timers {
        map.insert(id, TimerData { id, interval_ms: delay, repeats: false });
    }
    Ok(JsValue::from(id as f64))
}

fn set_interval_fn(_this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let delay = args.get(1).and_then(|v| v.as_number()).unwrap_or(0.0) as u64;
    let id = next_id();
    let mut timers = ensure_timers();
    if let Some(ref mut map) = *timers {
        map.insert(id, TimerData { id, interval_ms: delay, repeats: true });
    }
    Ok(JsValue::from(id as f64))
}

fn clear_timer_fn(_this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    if let Some(id_val) = args.first().and_then(|v| v.as_number()) {
        let mut timers = ensure_timers();
        if let Some(ref mut map) = *timers {
            map.remove(&(id_val as u64));
        }
    }
    Ok(JsValue::undefined())
}

fn set_immediate_fn(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let id = next_id();
    let mut timers = ensure_timers();
    if let Some(ref mut map) = *timers {
        map.insert(id, TimerData { id, interval_ms: 0, repeats: false });
    }
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

    pub fn process_timers(_context: &mut Context) {
        let mut timers = ensure_timers();
        if let Some(ref mut map) = *timers {
            let now = std::time::Instant::now();
            let _ = now;
            map.retain(|_id, data| {
                if data.interval_ms == 0 {
                    false
                } else if data.repeats {
                    true
                } else {
                    false
                }
            });
        }
    }

    pub fn clear_all() {
        let mut timers = ensure_timers();
        if let Some(ref mut map) = *timers {
            map.clear();
        }
    }
}
