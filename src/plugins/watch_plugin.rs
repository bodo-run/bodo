use async_trait::async_trait;
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde_json::from_str;
use std::{
    any::Any,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tokio::sync::{mpsc, Mutex};

use crate::{
    config::WatchConfig,
    errors::BodoError,
    graph::{Graph, NodeKind},
    plugin::Plugin,
    Result,
};

pub struct WatchPlugin {
    watchers: Arc<Mutex<Vec<RecommendedWatcher>>>,
    is_watching: Arc<AtomicBool>,
}

impl WatchPlugin {
    pub fn new() -> Self {
        Self {
            watchers: Arc::new(Mutex::new(Vec::new())),
            is_watching: Arc::new(AtomicBool::new(false)),
        }
    }

    async fn setup_watcher(
        &self,
        node_id: u64,
        config: WatchConfig,
        graph: Arc<Mutex<Graph>>,
    ) -> Result<()> {
        let (tx, mut rx) = mpsc::channel(32);
        let mut watcher = RecommendedWatcher::new(
            move |res| {
                if let Ok(event) = res {
                    let _ = tx.blocking_send(event);
                }
            },
            Config::default(),
        )
        .map_err(|e| BodoError::PluginError(format!("Failed to create watcher: {}", e)))?;

        // Watch all patterns
        for pattern in config.patterns {
            let path = PathBuf::from(pattern);
            watcher
                .watch(&path, RecursiveMode::Recursive)
                .map_err(|e| {
                    BodoError::PluginError(format!("Failed to watch {}: {}", path.display(), e))
                })?;
        }

        // Store watcher
        self.watchers.lock().await.push(watcher);

        // Start watch loop
        let is_watching = self.is_watching.clone();
        let debounce_ms = config.debounce_ms;
        let ignore_patterns = config.ignore_patterns;

        tokio::spawn(async move {
            let mut last_event = std::time::Instant::now();
            while is_watching.load(Ordering::SeqCst) {
                if let Some(event) = rx.recv().await {
                    // Check if path matches ignore patterns
                    if let Some(paths) = event.paths.first() {
                        let path_str = paths.to_string_lossy();
                        if ignore_patterns.iter().any(|p| path_str.contains(p)) {
                            continue;
                        }
                    }

                    // Only handle create/modify/remove events
                    match event.kind {
                        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                            // Debounce
                            let now = std::time::Instant::now();
                            if now.duration_since(last_event).as_millis() < debounce_ms as u128 {
                                continue;
                            }
                            last_event = now;

                            // Trigger task re-run
                            let mut graph = graph.lock().await;
                            if let Some(node) = graph.nodes.get_mut(node_id as usize) {
                                if let NodeKind::Task(_) = &node.kind {
                                    println!(
                                        "Triggering re-run of task {} due to file change",
                                        node.id
                                    );
                                }
                            }
                        }
                        _ => continue,
                    }
                }
            }
        });

        Ok(())
    }
}

#[async_trait]
impl Plugin for WatchPlugin {
    fn name(&self) -> &'static str {
        "WatchPlugin"
    }

    fn priority(&self) -> i32 {
        70 // Before execution plugins
    }

    async fn on_graph_build(&mut self, graph: &mut Graph) -> Result<()> {
        self.is_watching.store(true, Ordering::SeqCst);

        // Collect watch configs first to avoid borrow issues
        let mut watch_configs = Vec::new();
        for node in &graph.nodes {
            if let NodeKind::Task(_) = &node.kind {
                if let Some(watch_str) = node.metadata.get("watch") {
                    let config: WatchConfig = from_str(watch_str).map_err(|e| {
                        BodoError::PluginError(format!("Invalid watch config: {}", e))
                    })?;
                    watch_configs.push((node.id, config));
                }
            }
        }

        // Setup watchers
        let graph = Arc::new(Mutex::new(graph.clone()));
        for (node_id, config) in watch_configs {
            self.setup_watcher(node_id, config, graph.clone()).await?;
        }

        Ok(())
    }

    async fn on_after_run(&mut self, _graph: &mut Graph) -> Result<()> {
        // Stop all watchers
        self.is_watching.store(false, Ordering::SeqCst);
        self.watchers.lock().await.clear();
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
