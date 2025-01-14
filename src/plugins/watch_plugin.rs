use async_trait::async_trait;
use notify::RecommendedWatcher;
use std::any::Any;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::errors::Result;
use crate::graph::Graph;
use crate::plugin::{Plugin, PluginConfig};
use std::collections::HashSet;

pub struct WatchPlugin {
    watched_paths: Arc<Mutex<HashSet<PathBuf>>>,
    run_count: Arc<Mutex<u32>>,
    watcher: Option<RecommendedWatcher>,
}

impl WatchPlugin {
    pub fn new() -> Self {
        Self {
            watched_paths: Arc::new(Mutex::new(HashSet::new())),
            run_count: Arc::new(Mutex::new(0)),
            watcher: None,
        }
    }

    pub fn watch_file(&mut self, path: &Path) {
        if let Ok(mut paths) = self.watched_paths.lock() {
            paths.insert(path.to_path_buf());
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
