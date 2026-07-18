use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use crossbeam_channel::{unbounded, Receiver, Sender};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tracing::{info, warn};

use crate::loader::{find_config, load_config};
use crate::KlyronConfig;

#[derive(Debug, Clone)]
pub enum ConfigEvent {
    Changed(PathBuf, KlyronConfig),
    Error(String),
    Reloaded(PathBuf, KlyronConfig),
}

pub struct ConfigWatcher {
    config_path: Option<PathBuf>,
    watcher_tx: Option<Sender<ConfigEvent>>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl ConfigWatcher {
    pub fn new() -> Self {
        Self {
            config_path: None,
            watcher_tx: None,
            handle: None,
        }
    }

    pub fn watch(dir: &Path) -> anyhow::Result<(Receiver<ConfigEvent>, ConfigWatcher)> {
        let (tx, rx) = unbounded::<ConfigEvent>();

        let config_path = find_config(dir);
        let config_dir = config_path.as_ref().and_then(|p| p.parent()).map(|p| p.to_path_buf());

        if let Some(ref path) = config_path {
            info!("Watching config file: {}", path.display());

            let tx_clone = tx.clone();
            let watch_path = config_dir.clone().unwrap_or_else(|| dir.to_path_buf());
            let cfg_clone = path.clone();

            let mut watcher = RecommendedWatcher::new(
                move |res: Result<Event, notify::Error>| {
                    if let Ok(event) = res {
                        if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                            let event_paths: Vec<PathBuf> = event.paths.iter()
                                .filter(|p| p.ends_with(cfg_clone.file_name().unwrap_or_default()))
                                .cloned()
                                .collect();
                            if !event_paths.is_empty() {
                                match load_config(cfg_clone.parent().unwrap_or(Path::new("."))) {
                                    Ok(config) => {
                                        let _ = tx_clone.send(ConfigEvent::Reloaded(cfg_clone.clone(), config));
                                    }
                                    Err(e) => {
                                        let _ = tx_clone.send(ConfigEvent::Error(format!("Reload failed: {e}")));
                                    }
                                }
                            }
                        }
                    }
                },
                Config::default(),
            ).map_err(|e| anyhow::anyhow!("Failed to create file watcher: {e}"))?;

            watcher.watch(&watch_path, RecursiveMode::NonRecursive)
                .map_err(|e| anyhow::anyhow!("Failed to watch directory: {e}"))?;

            let watcher_handle = std::thread::spawn(move || {
                loop {
                    std::thread::sleep(Duration::from_secs(1));
                }
            });

            let config_watcher = ConfigWatcher {
                config_path,
                watcher_tx: Some(tx),
                handle: Some(watcher_handle),
            };

            Ok((rx, config_watcher))
        } else {
            warn!("No config file found to watch");
            let _ = tx.send(ConfigEvent::Error("No config file found".into()));
            Ok((rx, ConfigWatcher::new()))
        }
    }

    pub fn stop(&mut self) {
        if let Some(_handle) = self.handle.take() {
            info!("Stopping config watcher");
        }
    }

    pub fn is_watching(&self) -> bool {
        self.config_path.is_some() && self.handle.is_some()
    }

    pub fn watched_path(&self) -> Option<&Path> {
        self.config_path.as_deref()
    }
}

impl Drop for ConfigWatcher {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_config_watcher_creation() {
        let watcher = ConfigWatcher::new();
        assert!(!watcher.is_watching());
        assert!(watcher.watched_path().is_none());
    }

    #[test]
    fn test_config_watcher_no_config() {
        let dir = std::env::temp_dir().join("klyron_test_watch_none");
        let _ = fs::create_dir_all(&dir);
        let (rx, watcher) = ConfigWatcher::watch(&dir).unwrap();
        assert!(!watcher.is_watching());
        if let Ok(event) = rx.recv_timeout(Duration::from_millis(100)) {
            match event {
                ConfigEvent::Error(msg) => assert!(msg.contains("No config file")),
                _ => panic!("Expected error event"),
            }
        }
    }

    #[test]
    fn test_config_watcher_with_config() {
        let dir = std::env::temp_dir().join("klyron_test_watch_config");
        let _ = fs::create_dir_all(&dir);
        let config_path = dir.join("klyron.toml");
        fs::write(&config_path, r#"[project]
name = "test-app"
version = "1.0.0"
"#).unwrap();

        let (_rx, watcher) = ConfigWatcher::watch(&dir).unwrap();
        assert!(watcher.is_watching());
        assert_eq!(watcher.watched_path(), Some(config_path.as_path()));
    }

    #[test]
    fn test_config_event_types() {
        let event1 = ConfigEvent::Changed(
            PathBuf::from("klyron.toml"),
            KlyronConfig::default(),
        );
        let event2 = ConfigEvent::Error("test error".into());
        let event3 = ConfigEvent::Reloaded(
            PathBuf::from("klyron.toml"),
            KlyronConfig::default(),
        );

        match event1 {
            ConfigEvent::Changed(p, _) => assert_eq!(p, PathBuf::from("klyron.toml")),
            _ => panic!("Wrong variant"),
        }
        match event2 {
            ConfigEvent::Error(msg) => assert_eq!(msg, "test error"),
            _ => panic!("Wrong variant"),
        }
        match event3 {
            ConfigEvent::Reloaded(p, _) => assert_eq!(p, PathBuf::from("klyron.toml")),
            _ => panic!("Wrong variant"),
        }
    }
}
