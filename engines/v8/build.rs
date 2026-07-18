fn main() {
    #[cfg(feature = "native")]
    {
        let source_dir = std::path::Path::new("cpp/source");
        println!("cargo:rerun-if-changed=include/klyron_v8.h");
        println!("cargo:rerun-if-changed=cpp/impl/types.h");
        println!("cargo:rerun-if-changed=cpp/impl/internal.h");

        // Rebuild if any .cpp file changes
        for entry in std::fs::read_dir(source_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "cpp") {
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }

        let v8_dir = std::env::var("V8_DIR").ok();
        let v8_include = if let Some(ref dir) = v8_dir {
            format!("{}/include", dir)
        } else {
            "/usr/include/nodejs/deps/v8/include".to_string()
        };

        let v8_lib = if let Some(ref dir) = v8_dir {
            format!("{}/lib", dir)
        } else if std::path::Path::new("/usr/lib/x86_64-linux-gnu/libv8.so").exists() {
            "/usr/lib/x86_64-linux-gnu".to_string()
        } else {
            let candidates = vec!["/usr/lib", "/usr/local/lib", "/opt/v8/lib"];
            candidates
                .iter()
                .find(|d| std::path::Path::new(d).join("libv8.so").exists())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "/usr/lib".to_string())
        };

        let mut builder = cc::Build::new();
        builder
            .cpp(true)
            .include(&v8_include)
            .include("include")
            .include(".")
            .flag("-std=c++20")
            .flag_if_supported("-Wno-deprecated-declarations")
            .flag_if_supported("-Wno-unused-parameter")
            .flag_if_supported("-Wno-unknown-pragmas");

        // Add all .cpp files from cpp/source/
        for entry in std::fs::read_dir(source_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "cpp") {
                builder.file(&path);
            }
        }

        if let Ok(extra_flags) = std::env::var("V8_CXXFLAGS") {
            for flag in extra_flags.split(' ') {
                let flag = flag.trim();
                if !flag.is_empty() {
                    builder.flag(flag);
                }
            }
        }

        builder.compile("libklyron_v8.a");

        println!("cargo:rustc-link-search=native={}", v8_lib);
        println!("cargo:rustc-link-lib=v8");
        println!("cargo:rustc-link-lib=v8_libplatform");
        println!("cargo:rustc-link-lib=v8_libbase");

        let icu = format!("{}/libicui18n.so", v8_lib);
        if std::path::Path::new(&icu).exists() {
            println!("cargo:rustc-link-lib=icui18n");
            println!("cargo:rustc-link-lib=icuuc");
            println!("cargo:rustc-link-lib=icudata");
        }

        if let Ok(extra_libs) = std::env::var("V8_EXTRA_LIBS") {
            for lib in extra_libs.split(',') {
                let lib = lib.trim();
                if !lib.is_empty() {
                    println!("cargo:rustc-link-lib={}", lib);
                }
            }
        }

        #[cfg(target_os = "linux")]
        println!("cargo:rustc-link-lib=stdc++");
        #[cfg(target_os = "macos")]
        println!("cargo:rustc-link-lib=c++");
    }
}
