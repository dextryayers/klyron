use std::fs;
use std::path::Path;

#[no_mangle]
pub extern "C" fn on_after_build(ctx_ptr: i32, ctx_len: i32) -> i32 {
    let build_dir = unsafe {
        let slice = std::slice::from_raw_parts(ctx_ptr as *const u8, ctx_len as usize);
        if slice.is_empty() { "." } else { std::str::from_utf8_unchecked(slice) }
    };

    let result = minify_assets(build_dir);
    match result {
        Ok(output) => {
            let bytes = output.as_bytes();
            let ptr = bytes.as_ptr() as i32;
            std::mem::forget(bytes);
            ptr
        }
        Err(e) => {
            let bytes = e.as_bytes();
            let ptr = bytes.as_ptr() as i32;
            std::mem::forget(bytes);
            ptr
        }
    }
}

fn minify_assets(dir: &str) -> Result<String, String> {
    let dir_path = Path::new(dir);
    if !dir_path.exists() {
        return Err(format!("Build directory not found: {}", dir));
    }

    let mut js_count = 0u32;
    let mut css_count = 0u32;
    let mut saved_bytes: u64 = 0;

    visit_files(dir_path, &mut |path| {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        match ext {
            "js" | "mjs" => {
                let content = fs::read_to_string(path)
                    .map_err(|e| format!("Cannot read {:?}: {}", path, e))?;
                let original_size = content.len() as u64;
                let minified = minify_js(&content);
                fs::write(path, &minified)
                    .map_err(|e| format!("Cannot write {:?}: {}", path, e))?;
                saved_bytes += original_size - minified.len() as u64;
                js_count += 1;
            }
            "css" => {
                let content = fs::read_to_string(path)
                    .map_err(|e| format!("Cannot read {:?}: {}", path, e))?;
                let original_size = content.len() as u64;
                let minified = minify_css(&content);
                fs::write(path, &minified)
                    .map_err(|e| format!("Cannot write {:?}: {}", path, e))?;
                saved_bytes += original_size - minified.len() as u64;
                css_count += 1;
            }
            _ => {}
        }
        Ok(())
    })?;

    Ok(format!(
        "Minified {} JS and {} CSS file(s). Saved {} byte(s).",
        js_count, css_count, saved_bytes
    ))
}

fn minify_js(source: &str) -> String {
    let mut output = String::with_capacity(source.len());
    let chars: Vec<char> = source.chars().collect();
    let len = chars.len();
    let mut i = 0;

    let mut in_string = false;
    let mut string_char = '"';
    let mut in_block_comment = false;
    let mut in_line_comment = false;
    let mut prev_char = ' ';

    while i < len {
        if in_block_comment {
            if i + 1 < len && chars[i] == '*' && chars[i + 1] == '/' {
                in_block_comment = false;
                i += 2;
            } else {
                i += 1;
            }
            continue;
        }

        if in_line_comment {
            if chars[i] == '\n' {
                in_line_comment = false;
                output.push('\n');
                i += 1;
            } else {
                i += 1;
            }
            continue;
        }

        if !in_string {
            if i + 1 < len && chars[i] == '/' && chars[i + 1] == '/' {
                in_line_comment = true;
                i += 2;
                continue;
            }
            if i + 1 < len && chars[i] == '/' && chars[i + 1] == '*' {
                in_block_comment = true;
                i += 2;
                continue;
            }
        }

        if !in_string && (chars[i] == '"' || chars[i] == '\'' || chars[i] == '`') {
            in_string = !in_string;
            string_char = chars[i];
            output.push(chars[i]);
            i += 1;
            continue;
        }

        if in_string {
            if chars[i] == '\\' && i + 1 < len {
                output.push(chars[i]);
                output.push(chars[i + 1]);
                i += 2;
                continue;
            }
            if chars[i] == string_char {
                in_string = false;
            }
            output.push(chars[i]);
            i += 1;
            continue;
        }

        if chars[i] == ' ' || chars[i] == '\t' || chars[i] == '\n' || chars[i] == '\r' {
            let is_significant = {
                let next_non_ws = chars[i..].iter().position(|&c| !c.is_ascii_whitespace());
                match next_non_ws {
                    Some(n) if i + n < len => {
                        let next = chars[i + n];
                        prev_char.is_alphanumeric() && next.is_alphanumeric()
                            || prev_char == ')' && next == '('
                    }
                    _ => false,
                }
            };
            if is_significant {
                output.push(' ');
            }
            i += 1;
            while i < len && (chars[i] == ' ' || chars[i] == '\t' || chars[i] == '\n' || chars[i] == '\r') {
                i += 1;
            }
            continue;
        }

        output.push(chars[i]);
        prev_char = chars[i];
        i += 1;
    }

    output
}

fn minify_css(source: &str) -> String {
    let mut output = String::with_capacity(source.len());
    let chars: Vec<char> = source.chars().collect();
    let len = chars.len();
    let mut i = 0;

    let mut in_comment = false;
    let mut in_string = false;
    let mut string_char = '"';

    while i < len {
        if in_comment {
            if i + 1 < len && chars[i] == '*' && chars[i + 1] == '/' {
                in_comment = false;
                i += 2;
            } else {
                i += 1;
            }
            continue;
        }

        if !in_string && i + 1 < len && chars[i] == '/' && chars[i + 1] == '*' {
            in_comment = true;
            i += 2;
            continue;
        }

        if !in_string && (chars[i] == '"' || chars[i] == '\'') {
            in_string = !in_string;
            string_char = chars[i];
            output.push(chars[i]);
            i += 1;
            continue;
        }

        if in_string {
            if chars[i] == string_char {
                in_string = false;
            }
            output.push(chars[i]);
            i += 1;
            continue;
        }

        if chars[i] == ' ' || chars[i] == '\t' || chars[i] == '\n' || chars[i] == '\r' {
            i += 1;
            continue;
        }

        if chars[i] == ':' {
            output.push(':');
            i += 1;
            while i < len && (chars[i] == ' ' || chars[i] == '\t') {
                i += 1;
            }
            continue;
        }

        if chars[i] == ';' || chars[i] == ',' || chars[i] == '{' || chars[i] == '}' {
            output.push(chars[i]);
            i += 1;
            continue;
        }

        output.push(chars[i]);
        i += 1;
    }

    output
}

fn visit_files<F>(dir: &Path, f: &mut F) -> Result<(), String>
where
    F: FnMut(&Path) -> Result<(), String>,
{
    if dir.is_dir() {
        for entry in fs::read_dir(dir).map_err(|e| format!("Cannot read dir {:?}: {}", dir, e))? {
            let entry = entry.map_err(|e| format!("Dir entry error: {}", e))?;
            let path = entry.path();
            if path.is_dir() {
                if !path.file_name().map_or(false, |n| n == "node_modules" || n.starts_with('.')) {
                    visit_files(&path, f)?;
                }
            } else if path.is_file() {
                f(&path)?;
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
