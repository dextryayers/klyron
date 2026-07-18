use boa_engine::{Context, JsValue, NativeFunction, js_string, JsString};
use boa_engine::property::Attribute;
use std::path::Path;

fn to_str(v: &JsValue, ctx: &mut Context) -> String {
    v.to_string(ctx).map(|s| s.to_std_string_escaped()).unwrap_or_default()
}

pub struct PathUtils;

impl PathUtils {
    pub fn join(parts: &[String]) -> String {
        if parts.is_empty() { return String::new(); }
        let mut path = std::path::PathBuf::from(&parts[0]);
        for p in &parts[1..] { path = path.join(p); }
        path.to_string_lossy().to_string()
    }

    pub fn dirname(path: &str) -> String {
        Path::new(path).parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|| ".".to_string())
    }

    pub fn basename(path: &str) -> String {
        Path::new(path).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()
    }

    pub fn extname(path: &str) -> String {
        Path::new(path).extension().map(|e| e.to_string_lossy().to_string()).unwrap_or_default()
    }

    pub fn is_absolute(path: &str) -> bool { Path::new(path).is_absolute() }

    pub fn normalize(path: &str) -> String {
        let mut parts: Vec<&str> = Vec::new();
        for component in Path::new(path).components() {
            match component {
                std::path::Component::Normal(c) => parts.push(c.to_str().unwrap_or("")),
                std::path::Component::ParentDir => { parts.pop(); }
                _ => {}
            }
        }
        let result = parts.join("/");
        if path.starts_with('/') { format!("/{}", result) } else { result }
    }

    pub fn resolve(base: &str, relative: &str) -> String {
        let base_path = Path::new(base);
        if base_path.is_absolute() {
            base_path.join(relative).to_string_lossy().to_string()
        } else {
            let cwd = std::env::current_dir().unwrap_or_default();
            cwd.join(base).join(relative).to_string_lossy().to_string()
        }
    }

    pub fn relative(from: &str, to: &str) -> String {
        let from_path = Path::new(from);
        let to_path = Path::new(to);
        let mut from_comp: Vec<_> = from_path.components().collect();
        let mut to_comp: Vec<_> = to_path.components().collect();
        while !from_comp.is_empty() && !to_comp.is_empty() && from_comp[0] == to_comp[0] {
            from_comp.remove(0);
            to_comp.remove(0);
        }
        let mut result: Vec<&str> = Vec::new();
        for _ in &from_comp { result.push(".."); }
        for c in &to_comp { result.push(c.as_os_str().to_str().unwrap_or("")); }
        if result.is_empty() { return ".".to_string(); }
        result.join("/")
    }

    pub fn register(context: &mut Context) -> Result<(), crate::BoaError> {
        let mut builder = boa_engine::object::ObjectInitializer::new(context);
        builder.function(NativeFunction::from_fn_ptr(|_this, args, ctx| {
            let parts: Vec<String> = args.iter().map(|a| to_str(a, ctx)).collect();
            Ok(JsValue::from(JsString::from(Self::join(&parts))))
        }), js_string!("join"), 2usize)
        .function(NativeFunction::from_fn_ptr(|_this, args, ctx| {
            let p = args.first().map(|v| to_str(v, ctx)).unwrap_or_default();
            Ok(JsValue::from(JsString::from(Self::dirname(&p))))
        }), js_string!("dirname"), 1usize)
        .function(NativeFunction::from_fn_ptr(|_this, args, ctx| {
            let p = args.first().map(|v| to_str(v, ctx)).unwrap_or_default();
            Ok(JsValue::from(JsString::from(Self::basename(&p))))
        }), js_string!("basename"), 1usize)
        .function(NativeFunction::from_fn_ptr(|_this, args, ctx| {
            let p = args.first().map(|v| to_str(v, ctx)).unwrap_or_default();
            Ok(JsValue::from(JsString::from(Self::extname(&p))))
        }), js_string!("extname"), 1usize)
        .function(NativeFunction::from_fn_ptr(|_this, args, ctx| {
            let p = args.first().map(|v| to_str(v, ctx)).unwrap_or_default();
            Ok(JsValue::from(JsString::from(Self::normalize(&p))))
        }), js_string!("normalize"), 1usize)
        .function(NativeFunction::from_fn_ptr(|_this, args, ctx| {
            let p = args.first().map(|v| to_str(v, ctx)).unwrap_or_default();
            Ok(JsValue::from(Self::is_absolute(&p)))
        }), js_string!("isAbsolute"), 1usize)
        .function(NativeFunction::from_fn_ptr(|_this, args, ctx| {
            let base = args.first().map(|v| to_str(v, ctx)).unwrap_or_default();
            let rel = args.get(1).map(|v| to_str(v, ctx)).unwrap_or_default();
            Ok(JsValue::from(JsString::from(Self::resolve(&base, &rel))))
        }), js_string!("resolve"), 2usize)
        .function(NativeFunction::from_fn_ptr(|_this, args, ctx| {
            let from = args.first().map(|v| to_str(v, ctx)).unwrap_or_default();
            let to = args.get(1).map(|v| to_str(v, ctx)).unwrap_or_default();
            Ok(JsValue::from(JsString::from(Self::relative(&from, &to))))
        }), js_string!("relative"), 2usize);
        let obj = builder.build();

        context.register_global_property(
            js_string!("path"),
            obj,
            Attribute::WRITABLE | Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
        ).map_err(|e| crate::BoaError::from_js_error(&e))?;
        Ok(())
    }
}
