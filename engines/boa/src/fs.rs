use boa_engine::{Context, JsValue, NativeFunction, JsResult, js_string, JsString};
use boa_engine::object::builtins::JsArray;
use boa_engine::property::Attribute;
use std::path::Path;

fn val_to_string(v: &JsValue, ctx: &mut Context) -> String {
    v.to_string(ctx)
        .map(|s| s.to_std_string_escaped())
        .unwrap_or_default()
}

fn sanitize_path(path: &str) -> Result<String, String> {
    if path.contains("..") {
        return Err("path traversal detected: '..' is not allowed".into());
    }
    Ok(path.to_string())
}

fn fs_read_file(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let path = args.first().map(|v| val_to_string(v, context)).unwrap_or_default();
    let path = sanitize_path(&path).map_err(|e| {
        boa_engine::JsNativeError::error().with_message(e)
    })?;
    match std::fs::read_to_string(&path) {
        Ok(content) => Ok(JsValue::from(JsString::from(content))),
        Err(e) => Err(boa_engine::JsError::from_native(
            boa_engine::JsNativeError::error().with_message(format!("readFile {}: {}", path, e))
        )),
    }
}

fn fs_write_file(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let path = args.first().map(|v| val_to_string(v, context)).unwrap_or_default();
    let path = sanitize_path(&path).map_err(|e| {
        boa_engine::JsNativeError::error().with_message(e)
    })?;
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
    let path = sanitize_path(&path).map_err(|e| {
        boa_engine::JsNativeError::error().with_message(e)
    })?;
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
    let path = sanitize_path(&path).map_err(|e| {
        boa_engine::JsNativeError::error().with_message(e)
    })?;
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
    let path = sanitize_path(&path).map_err(|e| {
        boa_engine::JsNativeError::error().with_message(e)
    })?;
    match std::fs::remove_file(&path) {
        Ok(_) => Ok(JsValue::undefined()),
        Err(e) => Err(boa_engine::JsError::from_native(
            boa_engine::JsNativeError::error().with_message(format!("removeFile {}: {}", path, e))
        )),
    }
}

fn fs_remove_dir(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let path = args.first().map(|v| val_to_string(v, context)).unwrap_or_default();
    let path = sanitize_path(&path).map_err(|e| {
        boa_engine::JsNativeError::error().with_message(e)
    })?;
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
            let arr = JsArray::new(context);
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                let _ = arr.push(JsValue::from(JsString::from(name)), context);
            }
            Ok(JsValue::from(arr))
        }
        Err(e) => Err(boa_engine::JsError::from_native(
            boa_engine::JsNativeError::error().with_message(format!("readDir {}: {}", path, e))
        )),
    }
}

fn fs_rename(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let from = args.first().map(|v| val_to_string(v, context)).unwrap_or_default();
    let from = sanitize_path(&from).map_err(|e| {
        boa_engine::JsNativeError::error().with_message(e)
    })?;
    let to = args.get(1).map(|v| val_to_string(v, context)).unwrap_or_default();
    let to = sanitize_path(&to).map_err(|e| {
        boa_engine::JsNativeError::error().with_message(e)
    })?;
    match std::fs::rename(&from, &to) {
        Ok(_) => Ok(JsValue::undefined()),
        Err(e) => Err(boa_engine::JsError::from_native(
            boa_engine::JsNativeError::error().with_message(format!("rename {} -> {}: {}", from, to, e))
        )),
    }
}

fn fs_copy(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let from = args.first().map(|v| val_to_string(v, context)).unwrap_or_default();
    let from = sanitize_path(&from).map_err(|e| {
        boa_engine::JsNativeError::error().with_message(e)
    })?;
    let to = args.get(1).map(|v| val_to_string(v, context)).unwrap_or_default();
    let to = sanitize_path(&to).map_err(|e| {
        boa_engine::JsNativeError::error().with_message(e)
    })?;
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

            let obj = boa_engine::object::ObjectInitializer::new(context)
                .property(js_string!("size"), meta.len(), Attribute::all())
                .property(js_string!("isFile"), meta.is_file(), Attribute::all())
                .property(js_string!("isDir"), meta.is_dir(), Attribute::all())
                .property(js_string!("created"), created, Attribute::all())
                .property(js_string!("modified"), modified, Attribute::all())
                .build();
            Ok(JsValue::from(obj))
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
