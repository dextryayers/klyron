use std::fs;
use std::path::Path;

#[no_mangle]
pub extern "C" fn on_after_test(ctx_ptr: i32, ctx_len: i32) -> i32 {
    let context = unsafe {
        let slice = std::slice::from_raw_parts(ctx_ptr as *const u8, ctx_len as usize);
        String::from_utf8_lossy(slice).to_string()
    };

    let result = generate_test_report(&context);
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

fn generate_test_report(json_input: &str) -> Result<String, String> {
    let report_dir = Path::new(".klyron/test-reports");
    fs::create_dir_all(report_dir)
        .map_err(|e| format!("Cannot create report directory: {}", e))?;

    let parsed: serde_json::Value = serde_json::from_str(json_input)
        .map_err(|e| format!("Invalid JSON input: {}", e))?;

    let total = parsed["total"].as_u64().unwrap_or(0);
    let passed = parsed["passed"].as_u64().unwrap_or(0);
    let failed = parsed["failed"].as_u64().unwrap_or(0);
    let skipped = parsed["skipped"].as_u64().unwrap_or(0);
    let duration_ms = parsed["duration_ms"].as_u64().unwrap_or(0);

    let suites = parsed["suites"].as_array().map(|a| a.clone()).unwrap_or_default();

    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
    html.push_str("<meta charset=\"UTF-8\">\n");
    html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
    html.push_str("<title>Klyron Test Report</title>\n");
    html.push_str("<style>\n");
    html.push_str("* { margin: 0; padding: 0; box-sizing: border-box; }\n");
    html.push_str("body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #f5f5f5; color: #333; padding: 20px; }\n");
    html.push_str(".container { max-width: 960px; margin: 0 auto; }\n");
    html.push_str("h1 { font-size: 24px; margin-bottom: 20px; }\n");
    html.push_str(".summary { display: flex; gap: 16px; margin-bottom: 24px; }\n");
    html.push_str(".stat { background: white; border-radius: 8px; padding: 16px 24px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); flex: 1; text-align: center; }\n");
    html.push_str(".stat-value { font-size: 32px; font-weight: bold; }\n");
    html.push_str(".stat-label { font-size: 12px; color: #666; text-transform: uppercase; letter-spacing: 1px; }\n");
    html.push_str(".pass { color: #22c55e; } .fail { color: #ef4444; } .skip { color: #f59e0b; } .total { color: #3b82f6; }\n");
    html.push_str(".suite { background: white; border-radius: 8px; margin-bottom: 12px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); overflow: hidden; }\n");
    html.push_str(".suite-header { padding: 12px 16px; cursor: pointer; display: flex; justify-content: space-between; align-items: center; }\n");
    html.push_str(".suite-header:hover { background: #f8f8f8; }\n");
    html.push_str(".suite-name { font-weight: 600; }\n");
    html.push_str(".suite-badge { font-size: 12px; padding: 2px 8px; border-radius: 4px; }\n");
    html.push_str(".suite-badge.pass { background: #dcfce7; color: #166534; }\n");
    html.push_str(".suite-badge.fail { background: #fef2f2; color: #991b1b; }\n");
    html.push_str(".test-case { padding: 8px 16px 8px 32px; border-top: 1px solid #eee; display: flex; justify-content: space-between; }\n");
    html.push_str(".test-case.pass { background: #fafafa; } .test-case.fail { background: #fef2f2; }\n");
    html.push_str(".test-name { font-size: 14px; } .test-status { font-size: 12px; font-weight: 600; }\n");
    html.push_str(".test-status.pass { color: #22c55e; } .test-status.fail { color: #ef4444; }\n");
    html.push_str(".error-msg { color: #ef4444; font-size: 12px; margin-top: 4px; }\n");
    html.push_str(".footer { text-align: center; color: #999; font-size: 12px; margin-top: 32px; }\n");
    html.push_str("</style>\n</head>\n<body>\n");
    html.push_str("<div class=\"container\">\n");
    html.push_str(&format!("<h1>Klyron Test Report</h1>\n"));
    html.push_str("<div class=\"summary\">\n");
    html.push_str(&format!("<div class=\"stat\"><div class=\"stat-value total\">{}</div><div class=\"stat-label\">Total</div></div>\n", total));
    html.push_str(&format!("<div class=\"stat\"><div class=\"stat-value pass\">{}</div><div class=\"stat-label\">Passed</div></div>\n", passed));
    html.push_str(&format!("<div class=\"stat\"><div class=\"stat-value fail\">{}</div><div class=\"stat-label\">Failed</div></div>\n", failed));
    html.push_str(&format!("<div class=\"stat\"><div class=\"stat-value skip\">{}</div><div class=\"stat-label\">Skipped</div></div>\n", skipped));
    html.push_str("</div>\n");

    for suite in &suites {
        let suite_name = suite["name"].as_str().unwrap_or("Unnamed Suite");
        let suite_passed = suite["passed"].as_u64().unwrap_or(0);
        let suite_failed = suite["failed"].as_u64().unwrap_or(0);
        let tests = suite["tests"].as_array().map(|a| a.clone()).unwrap_or_default();

        let badge_class = if suite_failed > 0 { "fail" } else { "pass" };
        let badge_text = if suite_failed > 0 {
            format!("{} failed", suite_failed)
        } else {
            format!("{} passed", suite_passed)
        };

        html.push_str("<div class=\"suite\">\n");
        html.push_str(&format!(
            "<div class=\"suite-header\"><span class=\"suite-name\">{}</span><span class=\"suite-badge {}\">{}</span></div>\n",
            suite_name, badge_class, badge_text
        ));

        for test in &tests {
            let test_name = test["name"].as_str().unwrap_or("Unnamed Test");
            let test_status = test["status"].as_str().unwrap_or("unknown");
            let test_error = test["error"].as_str();

            let status_class = if test_status == "passed" { "pass" } else { "fail" };
            html.push_str(&format!(
                "<div class=\"test-case {}\"><span class=\"test-name\">{}</span><span class=\"test-status {}\">{}</span></div>\n",
                status_class, test_name, status_class, test_status
            ));

            if let Some(error) = test_error {
                if !error.is_empty() {
                    html.push_str(&format!("<div class=\"error-msg\" style=\"padding: 0 16px 8px 32px;\">{}</div>\n", error));
                }
            }
        }

        html.push_str("</div>\n");
    }

    html.push_str(&format!(
        "<div class=\"footer\">Generated by Klyron Test Reporter | Duration: {}ms | {}</div>\n",
        duration_ms,
        chrono_now()
    ));
    html.push_str("</div>\n</body>\n</html>\n");

    let report_path = report_dir.join("test-report.html");
    fs::write(&report_path, &html)
        .map_err(|e| format!("Cannot write report: {}", e))?;

    Ok(format!(
        "Test report generated: {} - {} passed, {} failed, {} skipped ({}ms)",
        report_path.display(),
        passed,
        failed,
        skipped,
        duration_ms
    ))
}

fn chrono_now() -> String {
    "2026-07-15T00:00:00Z".to_string()
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
