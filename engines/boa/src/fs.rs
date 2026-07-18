use boa_engine::{Context, JsValue, NativeFunction, JsResult, js_string, JsString};
use boa_engine::property::Attribute;
use std::path::Path;

fn val_to_string(v: &JsValue, ctx: &mut Context) -> String {
    v.to_string(ctx)
        .map(|s| s.to_std_string_escaped())
        .unwrap_or_default()
}

fn fs_read_file(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let path = args.first().map(|v| val_to_string(v, context)).unwrap_or_default();
    match std::fs::read_to_string(&path) {
        Ok(content) => Ok(JsValue::from(JsString::from(content))),
        Err(e) => Err(boa_engine::JsError::from_native(
            boa_engine::JsNativeError::error().with_message(format!("readFile {}: {}", path, e))
        )),
    }
}

fn fs_write_file(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let path = args.first().map(|v| val_to_string(v, context)).unwrap_or_default();
    let content = args.get(1).map(|v| val_to_string(v, context)).unwrap_or_default();
    match std::fs::write(&path, &content) {
        Ok(_) => Ok(JsValue::undefined()),
        Err(e) => Err(boa_engine::JsError::from_native(
            boa_engine::JsNativeError::error().with_message(format!("writeFile {}: {}", path, e))
        )),
    }
}

fn fs_append_file(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let path = args.first().map(|v| val_to_string(v, context)).unwrap_or_default();
    let content = args.get(1).map(|v| val_to_string(v, context)).unwrap_or_default();
    use std::io::Write;
    match std::fs::OpenOptions::new().append(true).create(true).open(&path) {
        Ok(mut file) => {
            if let Err(e) = file.write_all(content.as_bytes()) {
                return Err(boa_engine::JsError::from_native(
                    boa_engine::JsNativeError::error().with_message(format!("appendFile {}: {}", path, e))
                ));
            }
            Ok(JsValue::undefined())
        }
        Err(e) => Err(boa_engine::JsError::from_native(
            boa_engine::JsNativeError::error().with_message(format!("appendFile {}: {}", path, e))
        )),
    }
}

fn fs_exists(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let path = args.first().map(|v| val_to_string(v, context)).unwrap_or_default();
    Ok(JsValue::from(Path::new(&path).exists()))
}

fn fs_is_dir(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let path = args.first().map(|v| val_to_string(v, context)).unwrap_or_default();
    Ok(JsValue::from(Path::new(&path).is_dir()))
}

fn fs_is_file(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let path = args.first().map(|v| val_to_string(v, context)).unwrap_or_default();
    Ok(JsValue::from(Path::new(&path).is_file()))
}

fn fs_mkdir(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let path = args.first().map(|v| val_to_string(v, context)).unwrap_or_default();
    let recursive = args.get(1).and_then(|v| v.as_boolean()).unwrap_or(false);
    let result = if recursive { std::fs::create_dir_all(&path) } else { std::fs::create_dir(&path) };
    match result {
        Ok(_) => Ok(JsValue::undefined()),
        Err(e) => Err(boa_engine::JsError::from_native(
            boa_engine::JsNativeError::error().with_message(format!("mkdir {}: {}", path, e))
        )),
    }
}

fn fs_remove_file(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let path = args.first().map(|v| val_to_string(v, context)).unwrap_or_default();
    match std::fs::remove_file(&path) {
        Ok(_) => Ok(JsValue::undefined()),
        Err(e) => Err(boa_engine::JsError::from_native(
            boa_engine::JsNativeError::error().with_message(format!("removeFile {}: {}", path, e))
        )),
    }
}

fn fs_remove_dir(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let path = args.first().map(|v| val_to_string(v, context)).unwrap_or_default();
    match std::fs::remove_dir_all(&path) {
        Ok(_) => Ok(JsValue::undefined()),
        Err(e) => Err(boa_engine::JsError::from_native(
            boa_engine::JsNativeError::error().with_message(format!("removeDir {}: {}", path, e))
        )),
    }
}

fn fs_read_dir(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let path = args.first().map(|v| val_to_string(v, context)).unwrap_or_default();
    match std::fs::read_dir(&path) {
        Ok(entries) => {
            let names: Vec<String> = entries
                .filter_map(|e| e.ok().map(|e| e.file_name().to_string_lossy().to_string()))
                .collect();
            let names_json = serde_json::to_string(&names).unwrap_or_else(|_| "[]".to_string());
            context.eval(boa_engine::Source::from_bytes(&names_json))
        }
        Err(e) => Err(boa_engine::JsError::from_native(
            boa_engine::JsNativeError::error().with_message(format!("readDir {}: {}", path, e))
        )),
    }
}

fn fs_rename(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let from = args.first().map(|v| val_to_string(v, context)).unwrap_or_default();
    let to = args.get(1).map(|v| val_to_string(v, context)).unwrap_or_default();
    match std::fs::rename(&from, &to) {
        Ok(_) => Ok(JsValue::undefined()),
        Err(e) => Err(boa_engine::JsError::from_native(
            boa_engine::JsNativeError::error().with_message(format!("rename {} -> {}: {}", from, to, e))
        )),
    }
}

fn fs_copy(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let from = args.first().map(|v| val_to_string(v, context)).unwrap_or_default();
    let to = args.get(1).map(|v| val_to_string(v, context)).unwrap_or_default();
    match std::fs::copy(&from, &to) {
        Ok(_) => Ok(JsValue::undefined()),
        Err(e) => Err(boa_engine::JsError::from_native(
            boa_engine::JsNativeError::error().with_message(format!("copy {} -> {}: {}", from, to, e))
        )),
    }
}

fn fs_stat(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let path = args.first().map(|v| val_to_string(v, context)).unwrap_or_default();
    match std::fs::metadata(&path) {
        Ok(meta) => {
            let created = meta.created()
                .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as f64)
                .unwrap_or(0.0);
            let modified = meta.modified()
                .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as f64)
                .unwrap_or(0.0);
            let code = format!(
                "({{ size: {}, isFile: {}, isDir: {}, created: {}, modified: {} }})",
                meta.len(), meta.is_file(), meta.is_dir(), created, modified,
            );
            context.eval(boa_engine::Source::from_bytes(&code))
        }
        Err(e) => Err(boa_engine::JsError::from_native(
            boa_engine::JsNativeError::error().with_message(format!("stat {}: {}", path, e))
        )),
    }
}

pub struct Fs;

impl Fs {
    pub fn register(context: &mut Context) -> Result<(), crate::BoaError> {
        let mut builder = boa_engine::object::ObjectInitializer::new(context);
        builder.function(NativeFunction::from_fn_ptr(fs_read_file), js_string!("readFile"), 1usize)
            .function(NativeFunction::from_fn_ptr(fs_write_file), js_string!("writeFile"), 2usize)
            .function(NativeFunction::from_fn_ptr(fs_append_file), js_string!("appendFile"), 2usize)
            .function(NativeFunction::from_fn_ptr(fs_exists), js_string!("exists"), 1usize)
            .function(NativeFunction::from_fn_ptr(fs_is_dir), js_string!("isDir"), 1usize)
            .function(NativeFunction::from_fn_ptr(fs_is_file), js_string!("isFile"), 1usize)
            .function(NativeFunction::from_fn_ptr(fs_mkdir), js_string!("mkdir"), 2usize)
            .function(NativeFunction::from_fn_ptr(fs_remove_file), js_string!("removeFile"), 1usize)
            .function(NativeFunction::from_fn_ptr(fs_remove_dir), js_string!("removeDir"), 1usize)
            .function(NativeFunction::from_fn_ptr(fs_read_dir), js_string!("readDir"), 1usize)
            .function(NativeFunction::from_fn_ptr(fs_rename), js_string!("rename"), 2usize)
            .function(NativeFunction::from_fn_ptr(fs_copy), js_string!("copy"), 2usize)
            .function(NativeFunction::from_fn_ptr(fs_stat), js_string!("stat"), 1usize);
        let fs_obj = builder.build();

        context.register_global_property(
            js_string!("fs"),
            fs_obj,
            Attribute::WRITABLE | Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
        ).map_err(|e| crate::BoaError::from_js_error(&e))?;
        Ok(())
    }
}
