use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use std::collections::HashMap;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() || data.len() > 1024 * 1024 {
        return;
    }
    let input = String::from_utf8_lossy(data);

    let _ = serde_json::from_str::<HashMap<String, serde_json::Value>>(&input);
    let _ = toml::from_str::<toml::Value>(&input);

    if input.contains("version") || input.contains("dependencies") || input.contains("name") {
        let _ = klyron_pm::LockfileV3::from_npm_lockfile(&input);
    }

    let _ = klyron_pm::PackageManagerKind::detect(std::path::Path::new(&input.trim()));
});
