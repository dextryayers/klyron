use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=cpp/source/");
    println!("cargo:rerun-if-changed=cpp/impl/");
    println!("cargo:rerun-if-changed=include/klyron_jsc.h");

    if std::env::var("CARGO_FEATURE_NATIVE").is_err() {
        return;
    }

    let lib = pkg_config::Config::new()
        .statik(false)
        .probe("javascriptcoregtk-4.1")
        .or_else(|_| pkg_config::Config::new().probe("javascriptcoregtk-6.0"))
        .or_else(|_| pkg_config::Config::new().probe("javascriptcoregtk-4.0"))
        .expect("Could not find JavaScriptCore via pkg-config. Install libjavascriptcoregtk-4.1-dev or set PKG_CONFIG_PATH");

    let mut builder = cc::Build::new();
    builder
        .cpp(true)
        .flag("-std=c++17")
        .flag_if_supported("-Wno-deprecated-declarations");

    for path in &lib.include_paths {
        builder.include(path);
    }

    builder
        .include("include")
        .include("cpp");

    let source_dir = Path::new("cpp/source");
    if source_dir.exists() {
        for entry in std::fs::read_dir(source_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "cpp") {
                builder.file(&path);
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
    }

    builder.compile("libklyron_jsc_wrapper.a");
}
