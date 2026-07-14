use clap::Subcommand;
use klyron_napi::NapiLoader;

#[derive(Subcommand)]
pub enum NapiAction {
    Build,
    Generate,
    Test,
    List,
}

pub fn run_napi(action: NapiAction) -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    match action {
        NapiAction::Build => {
            println!("🔧 Building N-API native module...");
            let napi_rs = dir.join("Cargo.toml");
            if napi_rs.exists() {
                crate::run_cmd("cargo", &["build", "--release"], &dir)
            } else {
                anyhow::bail!("No Cargo.toml found. Run `klyron napi generate` first.")
            }
        }
        NapiAction::Generate => {
            println!("📦 Generating N-API bindings...");
            std::fs::create_dir_all(dir.join("src"))?;

            let cargo_toml = r#"[package]
name = "native-module"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
napi = { version = "2", features = ["napi4"] }
napi-derive = "2"

[build-dependencies]
napi-build = "2"
"#;
            std::fs::write(dir.join("Cargo.toml"), cargo_toml)?;

            let build_rs = r#"extern crate napi_build;
fn main() { napi_build::setup(); }
"#;
            std::fs::write(dir.join("build.rs"), build_rs)?;

            let lib_rs = r#"use napi_derive::napi;

#[napi]
fn hello() -> String {
    "Hello from N-API!".to_string()
}

#[napi]
fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#;
            std::fs::write(dir.join("src").join("lib.rs"), lib_rs)?;
            println!("✅ N-API module scaffold created");
            println!("  Build with: klyron napi build");
            println!("  Test with: klyron napi test");
            Ok(())
        }
        NapiAction::Test => {
            println!("🧪 Testing N-API native module...");
            // Try to load and test the native module using klyron_napi
            let _loader = NapiLoader::new();
            let node_files = find_node_files(&dir);
            if !node_files.is_empty() {
                for node_file in &node_files {
                    println!("  Found .node file: {}", node_file.display());
                    match NapiLoader::is_napi_module(node_file) {
                        true => println!("    ✓ Valid N-API module"),
                        false => println!("    ✗ Not an N-API module"),
                    }
                }
                Ok(())
            } else {
                println!("  No .node files found in current directory");
                println!("  Trying via Node.js require...");
                crate::run_cmd("node", &["-e", "const m = require('./'); console.log(m.hello ? m.hello() : 'no hello export');"], &dir)
            }
        }
        NapiAction::List => {
            println!("📋 N-API modules in current directory:");
            let node_files = find_node_files(&dir);
            if node_files.is_empty() {
                println!("  No .node files found");
            } else {
                for node_file in &node_files {
                    let valid = NapiLoader::is_napi_module(node_file);
                    println!("  {} [{}]", node_file.display(), if valid { "✓" } else { "?" });
                }
            }
            Ok(())
        }
    }
}

fn find_node_files(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "node") {
                files.push(path.clone());
            }
            // Check build/Release subdirectory
            if path.is_dir() && path.file_name().map_or(false, |n| n == "build") {
                let release = path.join("Release");
                if release.exists() {
                    if let Ok(sub_entries) = std::fs::read_dir(&release) {
                        for sub in sub_entries.flatten() {
                            let sub_path = sub.path();
                            if sub_path.extension().map_or(false, |e| e == "node") {
                                files.push(sub_path);
                            }
                        }
                    }
                }
            }
        }
    }
    files
}
