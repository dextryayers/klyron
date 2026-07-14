//! Simple timestamped logger.
//!
//! Logs messages to stderr with ISO-8601 UTC timestamps and a level tag.
//!
//! ```
//! use klyron_rust::logger::{info, warn, error, debug};
//! info("server started");
//! warn("disk space low");
//! ```

use std::time::SystemTime;

fn format_timestamp(t: SystemTime) -> String {
    let dur = t
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();
    let millis = dur.subsec_millis();
    let (year, month, day) = days_to_date((secs / 86400) as i64);
    let hour = (secs % 86400) / 3600;
    let min = (secs % 3600) / 60;
    let sec = secs % 60;
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        year, month, day, hour, min, sec, millis
    )
}

/// Convert days since 1970-01-01 to a Gregorian (year, month, day).
fn days_to_date(days: i64) -> (i64, u32, u32) {
    let z = days + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m as u32, d as u32)
}

fn log(level: &str, msg: &str) {
    let ts = format_timestamp(SystemTime::now());
    eprintln!("[{}] {}: {}", ts, level, msg);
}

/// Log an informational message.
pub fn info(msg: &str) {
    log("INFO", msg);
}

/// Log a warning message.
pub fn warn(msg: &str) {
    log("WARN", msg);
}

/// Log an error message.
pub fn error(msg: &str) {
    log("ERROR", msg);
}

/// Log a debug message.
pub fn debug(msg: &str) {
    log("DEBUG", msg);
}
