#![no_main]

use libfuzzer_sys::fuzz_target;
use klyron_pm::KlyronLockfile;

fuzz_target!(|data: &[u8]| {
    if data.len() < 4 {
        return;
    }

    if data.len() > 1024 * 1024 {
        return;
    }

    let _ = KlyronLockfile::from_bytes(data);

    if let Ok(s) = std::str::from_utf8(data) {
        let _ = KlyronLockfile::from_json(s);
    }
});
