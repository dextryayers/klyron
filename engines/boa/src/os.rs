use boa_engine::{Context, JsValue, NativeFunction, js_string, JsString};
use boa_engine::property::Attribute;

pub struct OsInfo;

impl OsInfo {
    pub fn platform() -> String { std::env::consts::OS.to_string() }
    pub fn arch() -> String { std::env::consts::ARCH.to_string() }
    pub fn hostname() -> String {
        std::env::var("HOSTNAME").or_else(|_| std::env::var("HOST")).unwrap_or_else(|_| "localhost".to_string())
    }
    pub fn uptime_secs() -> u64 {
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()
    }
    pub fn total_memory() -> u64 {
        #[cfg(target_os = "linux")]
        if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
            for line in content.lines() {
                if line.starts_with("MemTotal:") {
                    if let Some(val) = line.split_whitespace().nth(1) {
                        if let Ok(kb) = val.parse::<u64>() { return kb * 1024; }
                    }
                }
            }
        }
        0
    }
    pub fn cpus() -> usize {
        std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1)
    }
    pub fn endianness() -> String {
        if cfg!(target_endian = "little") { "LE".to_string() } else { "BE".to_string() }
    }
    pub fn tmpdir() -> String { std::env::temp_dir().to_string_lossy().to_string() }
    pub fn homedir() -> String {
        std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).unwrap_or_else(|_| "/".to_string())
    }

    pub fn register(context: &mut Context) -> Result<(), crate::BoaError> {
        let mut builder = boa_engine::object::ObjectInitializer::new(context);
        builder.function(NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
            Ok(JsValue::from(JsString::from(Self::platform())))
        }), js_string!("platform"), 0usize)
        .function(NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
            Ok(JsValue::from(JsString::from(Self::arch())))
        }), js_string!("arch"), 0usize)
        .function(NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
            Ok(JsValue::from(JsString::from(Self::hostname())))
        }), js_string!("hostname"), 0usize)
        .function(NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
            Ok(JsValue::from(Self::uptime_secs() as f64))
        }), js_string!("uptime"), 0usize)
        .function(NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
            Ok(JsValue::from(Self::cpus() as f64))
        }), js_string!("cpus"), 0usize)
        .function(NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
            Ok(JsValue::from(Self::total_memory() as f64))
        }), js_string!("totalMemory"), 0usize)
        .function(NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
            Ok(JsValue::from(JsString::from(Self::endianness())))
        }), js_string!("endianness"), 0usize)
        .function(NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
            Ok(JsValue::from(JsString::from(Self::tmpdir())))
        }), js_string!("tmpdir"), 0usize)
        .function(NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
            Ok(JsValue::from(JsString::from(Self::homedir())))
        }), js_string!("homedir"), 0usize);
        let obj = builder.build();

        context.register_global_property(
            js_string!("os"),
            obj,
            Attribute::WRITABLE | Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
        ).map_err(|e| crate::BoaError::from_js_error(&e))?;
        Ok(())
    }
}
