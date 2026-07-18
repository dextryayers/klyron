use boa_engine::{Context, JsValue, NativeFunction, JsResult, js_string};
use boa_engine::property::Attribute;
use std::time::Instant;
use std::collections::HashMap;
use std::sync::Mutex;

static TIMERS: Mutex<Option<HashMap<String, Instant>>> = Mutex::new(None);

fn get_timers() -> std::sync::MutexGuard<'static, Option<HashMap<String, Instant>>> {
    let mut timers = TIMERS.lock().unwrap();
    if timers.is_none() {
        *timers = Some(HashMap::new());
    }
    timers
}

fn format_args(args: &[JsValue], context: &mut Context) -> String {
    args.iter()
        .map(|a| {
            a.to_string(context)
                .map(|s| s.to_std_string_escaped())
                .unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn console_log(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    println!("{}", format_args(args, context));
    Ok(JsValue::undefined())
}

fn console_error(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    eprintln!("{}", format_args(args, context));
    Ok(JsValue::undefined())
}

fn console_warn(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    println!("{}", format_args(args, context));
    Ok(JsValue::undefined())
}

fn console_info(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    println!("{}", format_args(args, context));
    Ok(JsValue::undefined())
}

fn console_debug(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    println!("{}", format_args(args, context));
    Ok(JsValue::undefined())
}

fn console_table(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    if let Some(data) = args.first() {
        let s = data.to_string(context)
            .map(|s| s.to_std_string_escaped())
            .unwrap_or_default();
        println!("{}", s);
    }
    Ok(JsValue::undefined())
}

fn console_time(_this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let label = args.first()
        .and_then(|v| v.as_string())
        .map(|s| s.to_std_string_escaped())
        .unwrap_or_else(|| "default".to_string());
    let mut timers = get_timers();
    if let Some(ref mut map) = *timers {
        map.insert(label, Instant::now());
    }
    Ok(JsValue::undefined())
}

fn console_time_end(_this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let label = args.first()
        .and_then(|v| v.as_string())
        .map(|s| s.to_std_string_escaped())
        .unwrap_or_else(|| "default".to_string());
    let mut timers = get_timers();
    if let Some(ref mut map) = *timers {
        if let Some(start) = map.remove(&label) {
            let elapsed = start.elapsed();
            println!("{}: {:.3}ms", label, elapsed.as_secs_f64() * 1000.0);
        }
    }
    Ok(JsValue::undefined())
}

fn console_trace(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let _ = _context;
    let code = "new Error().stack";
    if let Ok(stack) = _context.eval(boa_engine::Source::from_bytes(code)) {
        let s = stack.to_string(_context)
            .map(|s| s.to_std_string_escaped())
            .unwrap_or_default();
        println!("Console.trace:\n{}", s);
    }
    Ok(JsValue::undefined())
}

fn console_count(_this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    println!("count: {}", format_args(args, _context));
    Ok(JsValue::undefined())
}

fn console_group(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn console_group_end(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

pub struct Console;

impl Console {
    pub fn register(context: &mut Context) -> Result<(), crate::BoaError> {
        let mut builder = boa_engine::object::ObjectInitializer::new(context);
        builder.function(NativeFunction::from_fn_ptr(console_log), js_string!("log"), 1usize)
            .function(NativeFunction::from_fn_ptr(console_error), js_string!("error"), 1usize)
            .function(NativeFunction::from_fn_ptr(console_warn), js_string!("warn"), 1usize)
            .function(NativeFunction::from_fn_ptr(console_info), js_string!("info"), 1usize)
            .function(NativeFunction::from_fn_ptr(console_debug), js_string!("debug"), 1usize)
            .function(NativeFunction::from_fn_ptr(console_table), js_string!("table"), 1usize)
            .function(NativeFunction::from_fn_ptr(console_time), js_string!("time"), 1usize)
            .function(NativeFunction::from_fn_ptr(console_time_end), js_string!("timeEnd"), 1usize)
            .function(NativeFunction::from_fn_ptr(console_trace), js_string!("trace"), 0usize)
            .function(NativeFunction::from_fn_ptr(console_count), js_string!("count"), 1usize)
            .function(NativeFunction::from_fn_ptr(console_group), js_string!("group"), 1usize)
            .function(NativeFunction::from_fn_ptr(console_group_end), js_string!("groupEnd"), 0usize);
        let console_obj = builder.build();

        context.register_global_property(
            js_string!("console"),
            console_obj,
            Attribute::WRITABLE | Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
        ).map_err(|e| crate::BoaError::from_js_error(&e))?;

        Ok(())
    }
}
