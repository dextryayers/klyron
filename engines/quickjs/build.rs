use std::path::Path;

fn main() {
    let vendor = Path::new("vendor/quickjs");
    let out_dir = std::env::var("OUT_DIR").unwrap();

    let source_files = [
        "quickjs.c",
        "libregexp.c",
        "libunicode.c",
        "dtoa.c",
    ];

    let mut builder = cc::Build::new();
    builder
        .include("vendor/quickjs")
        .extra_warnings(false)
        .flag_if_supported("-Wno-implicit-const-int-float-conversion")
        .flag_if_supported("-Wno-array-bounds")
        .flag_if_supported("-Wno-format-truncation")
        .define("_GNU_SOURCE", None);

    for src in &source_files {
        let src_path = vendor.join(src);
        println!("cargo:rerun-if-changed={}", src_path.display());
        builder.file(src_path);
    }

    builder
        .include("c")
        .file("c/quickjs_wrapper.c");
    println!("cargo:rerun-if-changed=c/quickjs_wrapper.c");
    println!("cargo:rerun-if-changed=c/quickjs_wrapper.h");

    builder.compile("libquickjs.a");

    println!("cargo:root={}", std::env::var("CARGO_MANIFEST_DIR").unwrap());
    println!("cargo:rustc-link-lib=static=quickjs");
    println!("cargo:rustc-link-search=native={}", out_dir);
}
