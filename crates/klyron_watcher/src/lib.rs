use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, SystemTime};

pub struct FileWatcher {
    path: String,
    interval: Duration,
}

impl FileWatcher {
    pub fn new(path: &str) -> Self {
        Self { path: path.to_string(), interval: Duration::from_millis(500) }
    }

    pub fn with_interval(mut self, ms: u64) -> Self {
        self.interval = Duration::from_millis(ms);
        self
    }

    pub fn start<F>(self, mut callback: F) -> anyhow::Result<()>
    where
        F: FnMut(String) + Send + 'static,
    {
        let path = self.path.clone();
        let interval = self.interval;
        let (tx, rx) = mpsc::channel::<String>();

        let tx_clone = tx.clone();
        thread::spawn(move || {
            let mut last_modified = SystemTime::now();
            loop {
                if let Ok(meta) = std::fs::metadata(&path) {
                    if let Ok(modified) = meta.modified() {
                        if modified > last_modified {
                            last_modified = modified;
                            let _ = tx_clone.send(path.clone());
                        }
                    }
                }
                thread::sleep(interval);
            }
        });

        while let Ok(path) = rx.recv() {
            callback(path);
        }
        Ok(())
    }
}

pub fn watch<F>(path: &Path, on_change: F) -> anyhow::Result<()>
where
    F: Fn() + Send + 'static,
{
    let watcher = FileWatcher::new(path.to_str().unwrap_or("."));
    watcher.start(move |_| on_change())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_file_watcher_new() {
        let w = FileWatcher::new("/tmp/test.txt");
        assert_eq!(w.path, "/tmp/test.txt");
    }
}
