use clap::Subcommand;

#[derive(Subcommand)]
pub enum NapiAction {
    Build,
    Generate,
    Test,
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
"#;
            std::fs::write(dir.join("src").join("lib.rs"), lib_rs)?;
            println!("✅ N-API module scaffold created");
            println!("  Build with: klyron napi build");
            Ok(())
        }
        NapiAction::Test => {
            crate::run_cmd("node", &["-e", "const m = require('./'); console.log(m.hello());"], &dir)
        }
    }
}
