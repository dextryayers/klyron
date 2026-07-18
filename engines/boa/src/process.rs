use boa_engine::{Context, JsValue, NativeFunction, js_string, JsString};
use boa_engine::object::builtins::JsArray;
use boa_engine::property::Attribute;

pub struct Process;

impl Process {
    pub fn env_var(key: &str) -> Option<String> { std::env::var(key).ok() }
    pub fn args() -> Vec<String> { std::env::args().collect() }
    pub fn pid() -> u32 { std::process::id() }
    pub fn cwd() -> String { std::env::current_dir().map(|p| p.to_string_lossy().to_string()).unwrap_or_default() }
    pub fn chdir(dir: &str) -> Result<(), String> { std::env::set_current_dir(dir).map_err(|e| format!("chdir failed: {}", e)) }
    pub fn memory_usage() -> u64 {
        #[cfg(target_os = "linux")]
        if let Ok(content) = std::fs::read_to_string("/proc/self/status") {
            for line in content.lines() {
                if line.starts_with("VmRSS:") {
                    if let Some(val) = line.split_whitespace().nth(1) {
                        if let Ok(kb) = val.parse::<u64>() { return kb * 1024; }
                    }
                }
            }
        }
        0
    }

    pub fn register(context: &mut Context) -> Result<(), crate::BoaError> {
        let mut builder = boa_engine::object::ObjectInitializer::new(context);
        builder.function(NativeFunction::from_fn_ptr(|_this, args, _ctx| {
            let key = args.first().and_then(|v| v.as_string()).map(|s| s.to_std_string_escaped()).unwrap_or_default();
            Ok(JsValue::from(JsString::from(Self::env_var(&key).unwrap_or_default())))
        }), js_string!("getenv"), 1usize)
        .function(NativeFunction::from_fn_ptr(|_this, args, _ctx| {
            let key = args.first().and_then(|v| v.as_string()).map(|s| s.to_std_string_escaped()).unwrap_or_default();
            let val = args.get(1).and_then(|v| v.as_string()).map(|s| s.to_std_string_escaped()).unwrap_or_default();
            unsafe { std::env::set_var(&key, &val); }
            Ok(JsValue::undefined())
        }), js_string!("setenv"), 2usize)
        .function(NativeFunction::from_fn_ptr(|_this, _args, context| {
            let arr = JsArray::new(context);
            for arg in Self::args() {
                let _ = arr.push(JsValue::from(JsString::from(arg)), context);
            }
            Ok(JsValue::from(arr))
        }), js_string!("argv"), 0usize)
        .function(NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
            Ok(JsValue::from(Self::pid() as f64))
        }), js_string!("pid"), 0usize)
        .function(NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
            Ok(JsValue::from(JsString::from(Self::cwd())))
        }), js_string!("cwd"), 0usize)
        .function(NativeFunction::from_fn_ptr(|_this, args, _ctx| {
            let dir = args.first().and_then(|v| v.as_string()).map(|s| s.to_std_string_escaped()).unwrap_or_default();
            match Self::chdir(&dir) {
                Ok(_) => Ok(JsValue::undefined()),
                Err(e) => Err(boa_engine::JsError::from_native(boa_engine::JsNativeError::error().with_message(e))),
            }
        }), js_string!("chdir"), 1usize)
        .function(NativeFunction::from_fn_ptr(|_this, args, _ctx| {
            let code = args.first().and_then(|v| v.as_number()).unwrap_or(0.0) as i32;
            Err(boa_engine::JsError::from_native(
                boa_engine::JsNativeError::error().with_message(format!("process.exit({}): use throw instead", code))
            ))
        }), js_string!("exit"), 1usize)
        .function(NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
            Ok(JsValue::from(Self::memory_usage() as f64))
        }), js_string!("memoryUsage"), 0usize);
        let process_obj = builder.build();

        context.register_global_property(
            js_string!("process"),
            process_obj,
            Attribute::WRITABLE | Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
        ).map_err(|e| crate::BoaError::from_js_error(&e))?;

        Ok(())
    }
}
