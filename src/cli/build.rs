use std::path::Path;
use std::process::Command;

fn compile_engine(name: &str, src: &Path, bin: &Path, compiler: &[&str], flags: &[&str]) -> bool {
    if bin.exists() {
        // Recompile if source is newer
        let src_mtime = std::fs::metadata(src).and_then(|m| m.modified()).ok();
        let bin_mtime = std::fs::metadata(bin).and_then(|m| m.modified()).ok();
        if let (Some(s), Some(b)) = (src_mtime, bin_mtime) {
            if s <= b {
                return true;
            }
        }
    }

    let mut cmd = Command::new(compiler[0]);
    cmd.args(&compiler[1..]);
    cmd.arg("-o").arg(bin);
    cmd.arg(src);
    for flag in flags {
        cmd.arg(flag);
    }

    let status = cmd.status();
    match status {
        Ok(s) if s.success() => {
            println!("cargo:warning=Compiled {} engine", name);
            true
        }
        Ok(s) => {
            println!("cargo:warning={} engine compilation failed (exit: {})", name, s);
            false
        }
        Err(e) => {
            println!("cargo:warning={} engine compiler not found: {}", name, e);
            false
        }
    }
}

fn main() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let engines_dir = manifest_dir.join("engines");
    let out_dir_str = std::env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir_str);

    // ─── C engine ───────────────────────────────────────────────────────
    let c_src = engines_dir.join("c/engine.c");
    let c_bin = out_dir.join("klyron-engine-c");
    if c_src.exists() {
        compile_engine("C", &c_src, &c_bin, &["cc", "-x", "c"], &["-lm", "-Wall", "-Wextra", "-Werror", "-O2", "-pthread"]);
        println!("cargo:rerun-if-changed={}", c_src.display());
    }

    // ─── C++ engine ─────────────────────────────────────────────────────
    let cpp_src = engines_dir.join("cpp/engine.cpp");
    let cpp_bin = out_dir.join("klyron-engine-cpp");
    if cpp_src.exists() {
        let mut compiled = false;
        for compiler in &[&["g++", "-std=c++20"], &["clang++", "-std=c++20"], &["c++", "-std=c++20"]] {
            if compile_engine("C++", &cpp_src, &cpp_bin, &[compiler[0], compiler[1]],
                             &["-lm", "-pthread", "-Wall", "-Wextra", "-Werror", "-O2"]) {
                compiled = true;
                break;
            }
        }
        if !compiled {
            println!("cargo:warning=C++ engine compilation failed (tried g++, clang++, c++)");
        }
        println!("cargo:rerun-if-changed={}", cpp_src.display());
    }

    // ─── Go engine (optional) ───────────────────────────────────────────
    let go_src = engines_dir.join("go/engine.go");
    let go_bin = out_dir.join("klyron-engine-go");
    if go_src.exists() {
        let status = Command::new("go")
            .args(["build", "-o", go_bin.to_str().unwrap(), go_src.to_str().unwrap()])
            .status();
        if let Ok(s) = status {
            if s.success() {
                println!("cargo:warning=Compiled Go engine");
            } else {
                println!("cargo:warning=Go engine compilation failed");
            }
        } else {
            println!("cargo:warning=Go not installed, skipping Go engine");
        }
        println!("cargo:rerun-if-changed={}", go_src.display());
    }

    // ─── Rust engine (optional) ─────────────────────────────────────────
    let rs_dir = engines_dir.join("rs");
    let rs_bin = out_dir.join("klyron-engine-rust");
    if rs_dir.join("Cargo.toml").exists() {
        let status = Command::new("cargo")
            .args(["build", "--release", "--manifest-path", rs_dir.join("Cargo.toml").to_str().unwrap()])
            .status();
        if let Ok(s) = status {
            if s.success() {
                let rel_bin = rs_dir.join("target/release/klyron-engine-rust");
                if rel_bin.exists() {
                    std::fs::copy(&rel_bin, &rs_bin).ok();
                    println!("cargo:warning=Compiled Rust engine");
                }
            } else {
                println!("cargo:warning=Rust engine compilation failed");
            }
        }
        println!("cargo:rerun-if-changed={}", rs_dir.join("src").display());
    }

    // ─── Zig engine (optional) ──────────────────────────────────────────
    let zig_src = engines_dir.join("zig/engine.zig");
    let zig_bin = out_dir.join("klyron-engine-zig");
    if zig_src.exists() {
        let status = Command::new("zig")
            .args(["build-exe", zig_src.to_str().unwrap(), "--name", "klyron-engine-zig",
                    "-O", "ReleaseFast", "--cache-dir", "/tmp/zig-cache"])
            .status();
        if let Ok(s) = status {
            if s.success() {
                let zig_out = zig_src.parent().unwrap().join("klyron-engine-zig");
                if zig_out.exists() {
                    std::fs::copy(&zig_out, &zig_bin).ok();
                    std::fs::remove_file(&zig_out).ok();
                    println!("cargo:warning=Compiled Zig engine");
                }
            } else {
                println!("cargo:warning=Zig engine compilation failed");
            }
        } else {
            println!("cargo:warning=Zig not installed, skipping Zig engine");
        }
        println!("cargo:rerun-if-changed={}", zig_src.display());
    }

    // Re-run if any engine changes
    println!("cargo:rerun-if-changed=build.rs");
}
