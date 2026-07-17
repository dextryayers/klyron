fn main() {
    println!("cargo:rerun-if-changed=cpp/jsc_wrapper.cc");
    println!("cargo:rerun-if-changed=cpp/jsc_wrapper.h");

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
        .include("cpp")
        .file("cpp/jsc_wrapper.cc")
        .compile("libjsc_wrapper.a");
}
