use clap::Subcommand;
use klyron_plugin::PluginManager;
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum PluginAction {
    Install { name: String },
    Remove { name: String },
    List,
    Update,
    Create { name: String },
}

fn plugins_dir() -> PathBuf {
    dirs::home_dir()
        .map(|p| p.join(".klyron/plugins"))
        .unwrap_or_else(|| PathBuf::from("/tmp/klyron-plugins"))
}

pub fn run_plugin(action: PluginAction) -> anyhow::Result<()> {
    let plugin_dir = plugins_dir();

    match action {
        PluginAction::Install { name } => {
            println!("🔌 Installing plugin: {}", name);
            std::fs::create_dir_all(&plugin_dir)?;
            let wasm_path = plugin_dir.join(format!("{}.wasm", name));

            // Try to download from plugin registry
            let url = format!("https://registry.klyron.dev/plugins/{name}/latest/download");
            match ureq::get(&url).call() {
                Ok(response) => {
                    let mut reader = response.into_reader();
                    let mut data = Vec::new();
                    std::io::Read::read_to_end(&mut reader, &mut data)?;
                    std::fs::write(&wasm_path, &data)?;
                    println!("  Downloaded to: {}", wasm_path.display());

                    // Load the plugin to verify it works
                    let mut pm = PluginManager::new()?;
                    match pm.load(wasm_path.to_str().unwrap()) {
                        Ok(plugin_name) => println!("  Loaded plugin: {plugin_name}"),
                        Err(e) => eprintln!("  Warning: plugin loaded but verification failed: {e}"),
                    }
                }
                Err(_) => {
                    // Fallback: create placeholder
                    println!("  Plugin registry not available, creating placeholder");
                    let placeholder = format!(
                        "(module\n  (func (export \"hello\") (result i32)\n    i32.const 42\n  )\n  (memory (export \"memory\") 1)\n)"
                    );
                    std::fs::write(&wasm_path, placeholder.as_bytes())?;
                    println!("  Placeholder created at: {}", wasm_path.display());
                }
            }
            Ok(())
        }
        PluginAction::Remove { name } => {
            println!("🔌 Removing plugin: {}", name);
            let wasm_path = plugin_dir.join(format!("{}.wasm", name));
            if wasm_path.exists() {
                std::fs::remove_file(&wasm_path)?;
                println!("  Removed: {}", wasm_path.display());
            } else {
                println!("  Plugin '{}' not found", name);
            }
            Ok(())
        }
        PluginAction::List => {
            println!("🔌 Installed plugins:");
            if plugin_dir.exists() {
                let mut count = 0u32;
                for entry in std::fs::read_dir(&plugin_dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.extension().map_or(false, |e| e == "wasm") {
                        let name = path.file_stem().unwrap_or_default().to_string_lossy();
                        println!("  {name}");

                        // Try to load and inspect
                        if let Ok(pm) = PluginManager::new() {
                            if let Some(manifest) = pm.get_manifest(name.as_ref()) {
                                println!("    v{} ({})", manifest.version, manifest.permissions.join(", "));
                            }
                        }
                        count += 1;
                    }
                }
                if count == 0 {
                    println!("  No plugins installed");
                }
            } else {
                println!("  Plugin directory not found at {}", plugin_dir.display());
                println!("  Install a plugin first with: klyron plugin install <name>");
            }
            Ok(())
        }
        PluginAction::Update => {
            println!("🔌 Updating all plugins...");
            if plugin_dir.exists() {
                for entry in std::fs::read_dir(&plugin_dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.extension().map_or(false, |e| e == "wasm") {
                        let name = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
                        println!("  Updating {name}...");
                        // Re-download
                        let url = format!("https://registry.klyron.dev/plugins/{name}/latest/download");
                        if let Ok(response) = ureq::get(&url).call() {
                            let mut reader = response.into_reader();
                            let mut data = Vec::new();
                            let _ = std::io::Read::read_to_end(&mut reader, &mut data);
                            let _ = std::fs::write(&path, &data);
                        }
                    }
                }
            }
            println!("  Update complete");
            Ok(())
        }
        PluginAction::Create { name } => {
            println!("🔌 Scaffolding new plugin: {}", name);
            let dir = std::env::current_dir()?.join(&name);
            std::fs::create_dir_all(&dir)?;
            std::fs::write(dir.join("klyron-plugin.toml"), r#"[plugin]
name = "NAME"
version = "0.1.0"
description = "My Klyron plugin"

[permissions]
allow_net = []
allow_fs = []
allow_env = []
"#.replace("NAME", &name))?;

            std::fs::create_dir_all(dir.join("src"))?;
            std::fs::write(dir.join("src").join("lib.rs"), r#"use klyron_sdk::prelude::*;

#[klyron_plugin]
fn hello() -> String {
    "Hello from plugin!".to_string()
}

#[klyron_plugin]
fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}
"#)?;

            std::fs::write(dir.join("Cargo.toml"), format!(r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
klyron-sdk = "0.1"

[profile.release]
opt-level = "s"
lto = true
"#))?;

            println!("✅ Plugin scaffold created in {}/", name);
            println!("  Build with: cd {name} && cargo build --target wasm32-unknown-unknown --release");
            println!("  Install with: klyron plugin install ./target/wasm32-unknown-unknown/release/{name}.wasm");
            Ok(())
        }
    }
}
