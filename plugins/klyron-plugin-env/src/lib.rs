use std::env;

#[no_mangle]
pub extern "C" fn on_before_serve(ctx_ptr: i32, ctx_len: i32) -> i32 {
    let _context = unsafe {
        let slice = std::slice::from_raw_parts(ctx_ptr as *const u8, ctx_len as usize);
        String::from_utf8_lossy(slice).to_string()
    };

    let result = validate_environment();
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

fn validate_environment() -> Result<String, String> {
    let required_vars = vec![
        "NODE_ENV",
        "PORT",
        "HOST",
    ];

    let recommended_vars = vec![
        "DATABASE_URL",
        "API_KEY",
    ];

    let mut missing_required = Vec::new();
    let mut missing_recommended = Vec::new();
    let mut present = Vec::new();

    for var in &required_vars {
        match env::var(var) {
            Ok(val) => {
                if val.is_empty() {
                    missing_required.push(format!("{} (empty)", var));
                } else {
                    present.push(var.to_string());
                }
            }
            Err(_) => {
                missing_required.push(var.to_string());
            }
        }
    }

    for var in &recommended_vars {
        if env::var(var).is_err() {
            missing_recommended.push(var.to_string());
        } else {
            present.push(var.to_string());
        }
    }

    if !missing_required.is_empty() {
        return Err(format!(
            "Missing required environment variables: {}. Present: {}.",
            missing_required.join(", "),
            present.join(", ")
        ));
    }

    let mut report = format!(
        "Environment validation passed. {} variable(s) checked.",
        present.len()
    );

    if !missing_recommended.is_empty() {
        report.push_str(&format!(
            " Warning: recommended variables missing: {}.",
            missing_recommended.join(", ")
        ));
    }

    report.push_str(&format!(" Present: {}.", present.join(", ")));
    Ok(report)
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
