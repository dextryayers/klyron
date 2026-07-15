use clap::Subcommand;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum ConfigAction {
    Get { key: String },
    Set { key: String, value: String },
    List,
    Delete { key: String },
}

fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("klyron")
        .join("config.json")
}

fn load_config() -> BTreeMap<String, serde_json::Value> {
    let path = config_path();
    if path.exists() {
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<BTreeMap<String, serde_json::Value>>(&s).ok())
            .unwrap_or_default()
    } else {
        BTreeMap::new()
    }
}

fn save_config(config: &BTreeMap<String, serde_json::Value>) -> anyhow::Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(config)?;
    std::fs::write(&path, content)?;
    Ok(())
}

pub fn run_config_action(action: ConfigAction) -> anyhow::Result<()> {
    match action {
        ConfigAction::Get { key } => {
            let config = load_config();
            match config.get(&key) {
                Some(val) => {
                    let s = match val {
                        serde_json::Value::String(s) => s.clone(),
                        other => serde_json::to_string_pretty(other)?,
                    };
                    println!("{s}");
                }
                None => {
                    eprintln!("Config key '{}' not found", key);
                }
            }
            Ok(())
        }
        ConfigAction::Set { key, value } => {
            let mut config = load_config();
            let parsed = serde_json::from_str::<serde_json::Value>(&value);
            match parsed {
                Ok(v) => {
                    config.insert(key.clone(), v);
                }
                Err(_) => {
                    config.insert(key.clone(), serde_json::Value::String(value.clone()));
                }
            }
            save_config(&config)?;
            eprintln!(
                "{} Config set: {} = {}",
                crate::Color::GREEN.paint("\u{2713}"),
                key,
                value,
            );
            Ok(())
        }
        ConfigAction::List => {
            let config = load_config();
            if config.is_empty() {
                println!("No configuration values set.");
                println!("Config file: {}", config_path().display());
                return Ok(());
            }
            println!("Configuration ({}):", config_path().display());
            for (key, val) in &config {
                let display = match val {
                    serde_json::Value::String(s) => s.clone(),
                    other => serde_json::to_string_pretty(other).unwrap_or_default(),
                };
                println!("  {:<20} = {}", key, display);
            }
            Ok(())
        }
        ConfigAction::Delete { key } => {
            let mut config = load_config();
            if config.remove(&key).is_some() {
                save_config(&config)?;
                eprintln!(
                    "{} Config deleted: {}",
                    crate::Color::GREEN.paint("\u{2713}"),
                    key,
                );
            } else {
                eprintln!("Config key '{}' not found", key);
            }
            Ok(())
        }
    }
}
