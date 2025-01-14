use async_trait::async_trait;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::any::Any;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{channel, Sender};

use crate::errors::Result;
use crate::graph::Graph;
use crate::plugin::{Plugin, PluginConfig};
use std::collections::HashSet;

pub struct WatchPlugin {
    watched_paths: Arc<Mutex<HashSet<PathBuf>>>,
    run_count: Arc<Mutex<u32>>,
    watcher: Option<RecommendedWatcher>,
    event_tx: Option<Sender<()>>,
}

impl WatchPlugin {
    pub fn new() -> Self {
        Self {
            watched_paths: Arc::new(Mutex::new(HashSet::new())),
            run_count: Arc::new(Mutex::new(0)),
            watcher: None,
            event_tx: None,
        }
    }

    pub fn watch_file(&mut self, path: &Path) {
        if let Ok(mut paths) = self.watched_paths.lock() {
            paths.insert(path.to_path_buf());
            if let Some(watcher) = &mut self.watcher {
                let _ = watcher.watch(path, RecursiveMode::NonRecursive);
            }
        }
    }

    pub fn get_run_count(&self) -> u32 {
        *self.run_count.lock().unwrap()
    }

    fn increment_run_count(&self) {
        if let Ok(mut count) = self.run_count.lock() {
            *count += 1;
        }
    }
}

#[async_trait]
impl Plugin for WatchPlugin {
    fn name(&self) -> &'static str {
        "watch"
    }

    async fn on_init(&mut self, _config: &PluginConfig) -> Result<()> {
        let (event_tx, mut event_rx) = channel(100);
        self.event_tx = Some(event_tx.clone());

        let watched_paths = Arc::clone(&self.watched_paths);
        let run_count = Arc::clone(&self.run_count);

        let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            if let Ok(event) = res {
                if let Ok(paths) = watched_paths.lock() {
                    for path_buf in event.paths {
                        if paths.contains(&path_buf) {
                            let _ = event_tx.blocking_send(());
                            break;
                        }
                    }
                }
            }
        })?;

        // Watch any existing paths
        if let Ok(paths) = self.watched_paths.lock() {
            for path in paths.iter() {
                let _ = watcher.watch(path, RecursiveMode::NonRecursive);
            }
        }

        self.watcher = Some(watcher);

        // Spawn a task to handle file change events
        tokio::spawn(async move {
            while let Some(_) = event_rx.recv().await {
                if let Ok(mut count) = run_count.lock() {
                    *count += 1;
                }
            }
        });

        Ok(())
    }

    async fn on_graph_build(&mut self, _graph: &mut Graph) -> Result<()> {
        Ok(())
    }

    fn on_task_start(&mut self) {
        self.increment_run_count();
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
