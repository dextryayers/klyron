use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=cpp/source/");
    println!("cargo:rerun-if-changed=cpp/impl/");
    println!("cargo:rerun-if-changed=include/klyron_jsc.h");

    if std::env::var("CARGO_FEATURE_NATIVE").is_err() {
        return;
    }

    /* Try system pkg-config first, then fallback to extracted headers */
    let lib = pkg_config::Config::new()
        .statik(false)
        .probe("javascriptcoregtk-4.1")
        .or_else(|_| pkg_config::Config::new().probe("javascriptcoregtk-6.0"))
        .or_else(|_| pkg_config::Config::new().probe("javascriptcoregtk-4.0"));

    let mut builder = cc::Build::new();
    builder
        .cpp(true)
        .flag("-std=c++17")
        .flag_if_supported("-Wno-deprecated-declarations");

    if let Ok(ref lib) = lib {
        for path in &lib.include_paths {
            builder.include(path);
        }
    } else {
        /* Fallback: use extracted headers from dev package */
        let extracted = "/tmp/opencode/jsc-dev/usr/include/webkitgtk-4.1";
        if Path::new(extracted).exists() {
            builder.include(extracted);
            /* No .so symlink exists — link directly to .so.0 */
            let so_path = "/usr/lib/x86_64-linux-gnu/libjavascriptcoregtk-4.1.so.0";
            if Path::new(so_path).exists() {
                /*
                 * Link directly to the versioned .so.0 via -l:filename
                 * so it appears AFTER the static lib in the link order,
                 * avoiding --as-needed discarding the library.
                 */
                println!("cargo:rustc-link-arg=-l:libjavascriptcoregtk-4.1.so.0");
                println!("cargo:rustc-link-arg=-L/usr/lib/x86_64-linux-gnu");
            }
            println!("cargo:warning=JSC: using extracted headers from dev package");
        } else {
            panic!("JavaScriptCore headers not found. Install libjavascriptcoregtk-4.1-dev or extract headers to /tmp/opencode/jsc-dev/");
        }
    }

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    builder
        .include(&manifest_dir)
        .include(std::path::Path::new(&manifest_dir).join("include"));

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
