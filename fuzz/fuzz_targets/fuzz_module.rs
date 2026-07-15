use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use std::path::Path;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() || data.len() > 16384 {
        return;
    }

    let input = String::from_utf8_lossy(data);

    let path = Path::new(&input.trim());

    let _ = path.extension();
    let _ = path.file_name();
    let _ = path.parent();
    let _ = path.is_absolute();

    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        match ext {
            "js" | "ts" | "jsx" | "tsx" | "mjs" | "cjs" | "json" | "node" => {}
            _ => {}
        }
    }

    let _ = input.trim().to_lowercase();
});
