#![no_main]

use libfuzzer_sys::fuzz_target;
use klyron_loader::ModuleResolver;
use std::path::Path;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() || data.len() > 4096 {
        return;
    }

    if let Ok(specifier) = std::str::from_utf8(data) {
        let resolver = ModuleResolver::new();
        let base = Path::new("/tmp/fuzz_base");
        let _ = resolver.resolve(specifier, base);
    }
});
