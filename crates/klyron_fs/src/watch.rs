use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use crate::FsEvent;

pub struct FileWatcherBuilder {
    paths: Vec<PathBuf>,
    recursive: bool,
    poll_interval: Duration,
    filter: Option<Vec<String>>,
}

impl FileWatcherBuilder {
    pub fn new() -> Self {
        Self {
            paths: Vec::new(),
            recursive: true,
            poll_interval: Duration::from_millis(500),
            filter: None,
        }
    }

    pub fn watch(mut self, path: &Path) -> Self {
        self.paths.push(path.to_path_buf());
        self
    }

    pub fn recursive(mut self, recursive: bool) -> Self {
        self.recursive = recursive;
        self
    }

    pub fn poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }

    pub fn extensions(mut self, exts: &[&str]) -> Self {
        self.filter = Some(exts.iter().map(|e| e.to_string()).collect());
        self
    }

    pub fn build(self) -> anyhow::Result<FileWatcher> {
        let (tx, rx) = mpsc::channel();

        let mut watcher = RecommendedWatcher::new(
            move |res: notify::Result<Event>| {
                if let Ok(event) = res {
                    let fs_event = match event.kind {
                        EventKind::Create(_) => FsEvent::Created(event.paths[0].clone()),
                        EventKind::Modify(_) => FsEvent::Modified(event.paths[0].clone()),
                        EventKind::Remove(_) => FsEvent::Removed(event.paths[0].clone()),
                        _ => return,
                    };
                    let _ = tx.send(fs_event);
                }
            },
            Config::default().with_poll_interval(self.poll_interval),
        )?;

        for path in &self.paths {
            if !path.exists() {
                anyhow::bail!("Watch path does not exist: {}", path.display());
            }
            watcher.watch(
                path,
                if self.recursive {
                    RecursiveMode::Recursive
                } else {
                    RecursiveMode::NonRecursive
                },
            )?;
        }

        Ok(FileWatcher {
            _watcher: Arc::new(Mutex::new(Some(watcher))),
            rx,
        })
    }
}

impl Default for FileWatcherBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct FileWatcher {
    _watcher: Arc<Mutex<Option<RecommendedWatcher>>>,
    rx: mpsc::Receiver<FsEvent>,
}

impl FileWatcher {
    pub fn receiver(&self) -> &mpsc::Receiver<FsEvent> {
        &self.rx
    }

    pub fn try_recv(&self) -> Result<FsEvent, mpsc::TryRecvError> {
        self.rx.try_recv()
    }

    pub fn iter(&self) -> mpsc::Iter<'_, FsEvent> {
        self.rx.iter()
    }
}

impl Drop for FileWatcher {
    fn drop(&mut self) {
        if let Ok(mut watcher) = self._watcher.lock() {
            if let Some(mut w) = watcher.take() {
                let _ = w.unwatch(Path::new("/"));
            }
        }
    }
}
