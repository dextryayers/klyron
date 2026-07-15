use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() || data.len() > 65536 {
        return;
    }

    let input = String::from_utf8_lossy(data);

    let _ = url::Url::parse(&input);

    if let Ok(url) = url::Url::parse(&input) {
        if url.has_host() {
            let host = url.host_str().unwrap_or("");
            let port = url.port().unwrap_or(80);
            let _ = format!("{}:{}", host, port);
        }
    }

    let parts: Vec<&str> = input.splitn(3, |c: char| c.is_whitespace()).collect();
    if parts.len() == 3 {
        let _method = parts[0];
        let _path = parts[1];
        let _version = parts[2];
    }

    for line in input.lines() {
        if let Some(idx) = line.find(':') {
            let _header_name = &line[..idx].trim();
            let _header_value = &line[idx + 1..].trim();
        }
    }
});
