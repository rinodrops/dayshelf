use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};

use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};

pub struct FolderWatch {
    _watcher: RecommendedWatcher,
    rx: Receiver<Result<Event, notify::Error>>,
}

impl FolderWatch {
    pub fn open(dir: PathBuf) -> Result<Self, notify::Error> {
        let (tx, rx) = mpsc::channel();
        let mut watcher = notify::recommended_watcher(tx)?;
        watcher.watch(&dir, RecursiveMode::NonRecursive)?;
        Ok(Self {
            _watcher: watcher,
            rx,
        })
    }

    pub fn poll(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        while let Ok(event) = self.rx.try_recv() {
            let Ok(event) = event else {
                continue;
            };
            paths.extend(event.paths);
        }
        paths
    }
}
