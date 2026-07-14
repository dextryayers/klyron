use clap::Subcommand;

#[derive(Subcommand)]
pub enum PluginAction {
    Install { name: String },
    Remove { name: String },
    List,
    Update,
    Create { name: String },
}

pub fn run_plugin(action: PluginAction) -> anyhow::Result<()> {
    match action {
        PluginAction::Install { name } => {
            println!("🔌 Installing plugin: {}", name);
            let plugin_dir = dirs::home_dir().map(|p| p.join(".klyron/plugins"));
            if let Some(dir) = plugin_dir {
                std::fs::create_dir_all(&dir)?;
                let plugin_path = dir.join(format!("{}.wasm", name));
                println!("  Plugin would be installed to: {}", plugin_path.display());
            }
            println!("  (Plugin system coming in Phase 8 — crate not yet implemented)");
            Ok(())
        }
        PluginAction::Remove { name } => {
            println!("🔌 Removing plugin: {}", name);
            Ok(())
        }
        PluginAction::List => {
            println!("🔌 Installed plugins:");
            let plugin_dir = dirs::home_dir().map(|p| p.join(".klyron/plugins"));
            if let Some(dir) = plugin_dir {
                if dir.exists() {
                    for entry in std::fs::read_dir(&dir)? {
                        let entry = entry?;
                        println!("  {}", entry.file_name().to_string_lossy());
                    }
                } else {
                    println!("  No plugins installed");
                }
            }
            Ok(())
        }
        PluginAction::Update => {
            println!("🔌 Updating all plugins...");
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
            std::fs::write(dir.join("src").join("lib.rs"), r#"use klyron_sdk::prelude::*;

#[klyron_plugin]
fn hello() -> String {
    "Hello from plugin!".to_string()
}
"#)?;
            println!("✅ Plugin scaffold created in {}/", name);
            Ok(())
        }
    }
}
