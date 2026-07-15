use std::fs;
use std::path::Path;

const LIVERELOAD_SCRIPT: &str = r#"<script>
(function() {
  const evtSource = new EventSource('/__klyron_livereload');
  evtSource.onmessage = function(e) {
    if (e.data === 'reload') {
      console.log('[Klyron] Live reload triggered');
      window.location.reload();
    }
  };
  evtSource.onerror = function() {
    console.log('[Klyron] Live reload connection lost');
    evtSource.close();
  };
  console.log('[Klyron] Live reload active');
})();
</script>"#;

#[no_mangle]
pub extern "C" fn on_after_build(ctx_ptr: i32, ctx_len: i32) -> i32 {
    let build_dir = unsafe {
        let slice = std::slice::from_raw_parts(ctx_ptr as *const u8, ctx_len as usize);
        if slice.is_empty() { "." } else { std::str::from_utf8_unchecked(slice) }
    };

    let result = inject_livereload(build_dir);
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

fn inject_livereload(dir: &str) -> Result<String, String> {
    let dir_path = Path::new(dir);
    if !dir_path.exists() {
        return Err(format!("Build directory not found: {}", dir));
    }

    let mut injected_count = 0u32;

    visit_html_files(dir_path, &mut |path| {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Cannot read {:?}: {}", path, e))?;

        if content.contains("__klyron_livereload") {
            return Ok(());
        }

        let modified = if let Some(pos) = content.find("</body>") {
            let mut new_content = String::with_capacity(content.len() + LIVERELOAD_SCRIPT.len());
            new_content.push_str(&content[..pos]);
            new_content.push_str(LIVERELOAD_SCRIPT);
            new_content.push_str("\n</body>");
            new_content.push_str(&content[pos + 7..]);
            new_content
        } else if let Some(pos) = content.find("</html>") {
            let mut new_content = String::with_capacity(content.len() + LIVERELOAD_SCRIPT.len());
            new_content.push_str(&content[..pos]);
            new_content.push_str(LIVERELOAD_SCRIPT);
            new_content.push_str("\n</html>");
            new_content.push_str(&content[pos + 7..]);
            new_content
        } else {
            content.to_string() + LIVERELOAD_SCRIPT
        };

        fs::write(path, &modified)
            .map_err(|e| format!("Cannot write {:?}: {}", path, e))?;

        injected_count += 1;
        Ok(())
    })?;

    if injected_count == 0 {
        return Ok("No HTML files found to inject live-reload script.".to_string());
    }

    Ok(format!(
        "Injected live-reload script into {} HTML file(s).",
        injected_count
    ))
}

fn visit_html_files<F>(dir: &Path, f: &mut F) -> Result<(), String>
where
    F: FnMut(&Path) -> Result<(), String>,
{
    if dir.is_dir() {
        for entry in fs::read_dir(dir).map_err(|e| format!("Cannot read dir {:?}: {}", dir, e))? {
            let entry = entry.map_err(|e| format!("Dir entry error: {}", e))?;
            let path = entry.path();
            if path.is_dir() {
                if !path.file_name().map_or(false, |n| n == "node_modules" || n.starts_with('.')) {
                    visit_html_files(&path, f)?;
                }
            } else if path.is_file() {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if ext == "html" || ext == "htm" {
                    f(&path)?;
                }
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
