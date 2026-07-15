use std::fs;
use std::path::Path;

#[no_mangle]
pub extern "C" fn on_before_build(ctx_ptr: i32, ctx_len: i32) -> i32 {
    let context = unsafe {
        let slice = std::slice::from_raw_parts(ctx_ptr as *const u8, ctx_len as usize);
        String::from_utf8_lossy(slice).to_string()
    };

    let build_dir = if context.is_empty() { "." } else { &context };

    let result = transpile_typescript(build_dir);
    match result {
        Ok(output) => {
            let bytes = output.as_bytes();
            let len = bytes.len() as i32;
            let ptr = bytes.as_ptr() as i32;
            std::mem::forget(bytes);
            ptr
        }
        Err(e) => {
            let err_bytes = e.as_bytes();
            let ptr = err_bytes.as_ptr() as i32;
            std::mem::forget(err_bytes);
            ptr
        }
    }
}

fn transpile_typescript(dir: &str) -> Result<String, String> {
    let dir_path = Path::new(dir);
    if !dir_path.exists() {
        return Err(format!("Directory not found: {}", dir));
    }

    let mut transpiled_count = 0u32;
    let mut errors = Vec::new();

    visit_dirs(dir_path, &mut |entry| {
        if let Some(ext) = entry.extension() {
            if ext == "ts" || ext == "tsx" {
                let ts_path = entry.path();
                let content = fs::read_to_string(&ts_path)
                    .map_err(|e| format!("Cannot read {:?}: {}", ts_path, e))?;

                let js_output = simple_transpile(&content);

                let js_path = ts_path.with_extension("js");
                fs::write(&js_path, &js_output)
                    .map_err(|e| format!("Cannot write {:?}: {}", js_path, e))?;

                transpiled_count += 1;
            }
        }
        Ok(())
    });

    if transpiled_count == 0 {
        return Ok("No TypeScript files found.".to_string());
    }

    Ok(format!(
        "Transpiled {} TypeScript file(s). {} error(s).",
        transpiled_count,
        errors.len()
    ))
}

fn simple_transpile(source: &str) -> String {
    let mut output = String::new();
    let mut i = 0;
    let chars: Vec<char> = source.chars().collect();

    while i < chars.len() {
        if i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '/' {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            output.push('\n');
            i += 1;
            continue;
        }

        if i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '*' {
            i += 2;
            while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                i += 1;
            }
            i += 2;
            continue;
        }

        if chars[i] == ':' && i + 1 < chars.len() && chars[i + 1] == ':' {
            output.push_str(".");
            i += 2;
            continue;
        }

        if source[i..].starts_with("interface ") {
            while i < chars.len() && chars[i] != '{' {
                i += 1;
            }
            if i < chars.len() {
                let mut brace_count = 1;
                i += 1;
                while i < chars.len() && brace_count > 0 {
                    if chars[i] == '{' {
                        brace_count += 1;
                    } else if chars[i] == '}' {
                        brace_count -= 1;
                    }
                    i += 1;
                }
            }
            continue;
        }

        if source[i..].starts_with("type ") && !source[i..].starts_with("typeof") {
            while i < chars.len() && chars[i] != ';' && chars[i] != '\n' {
                i += 1;
            }
            if i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }

        if source[i..].starts_with(": ")
            || source[i..].starts_with(" :")
            || (chars[i] == ':' && i > 0 && chars[i - 1] != ':')
        {
            if !source[i..].starts_with("::") {
                let before = if i > 0 { chars[i - 1] } else { ' ' };
                if before != ':' {
                    let mut j = i + 1;
                    while j < chars.len() && chars[j] == ' ' {
                        j += 1;
                    }
                    if j < chars.len()
                        && (chars[j].is_alphanumeric()
                            || chars[j] == '\''
                            || chars[j] == '"')
                    {
                        while j < chars.len()
                            && chars[j] != ','
                            && chars[j] != ')'
                            && chars[j] != ';'
                            && chars[j] != '\n'
                            && chars[j] != '='
                            && chars[j] != '>'
                            && chars[j] != '{'
                        {
                            j += 1;
                        }
                        i = j;
                        continue;
                    }
                }
            }
        }

        if source[i..].starts_with("as ") || source[i..].starts_with(" as ") {
            let mut j = i;
            while j < chars.len() && chars[j] != ';' && chars[j] != '\n' && chars[j] != ',' {
                j += 1;
            }
            i = j;
            continue;
        }

        output.push(chars[i]);
        i += 1;
    }

    output
}

fn visit_dirs<F>(dir: &Path, f: &mut F) -> Result<(), String>
where
    F: FnMut(&fs::DirEntry) -> Result<(), String>,
{
    if dir.is_dir() {
        for entry in fs::read_dir(dir).map_err(|e| format!("Cannot read dir {:?}: {}", dir, e))? {
            let entry = entry.map_err(|e| format!("Dir entry error: {}", e))?;
            let path = entry.path();
            if path.is_dir() && !path.file_name().map_or(false, |n| n == "node_modules" || n.starts_with('.')) {
                visit_dirs(&path, f)?;
            } else if path.is_file() {
                f(&entry)?;
            }
        }
    }
    Ok(())
}

#[no_mangle]
pub extern "C" fn alloc(size: i32) -> i32 {
    let mut buf = Vec::with_capacity(size as usize);
    let ptr = buf.as_mut_ptr() as i32;
    std::mem::forget(buf);
    ptr
}

#[no_mangle]
pub extern "C" fn dealloc(ptr: i32, size: i32) {
    unsafe {
        let _ = Vec::from_raw_parts(ptr as *mut u8, size as usize, size as usize);
    }
}
