use crate::{
    config::WatchConfig,
    errors::BodoError,
    graph::{Graph, NodeKind},
    plugin::{Plugin, PluginConfig},
    Result,
};
use globset::{Glob, GlobSet, GlobSetBuilder};
use log::{debug, error, warn};
use notify::{Config as NotifyConfig, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    any::Any,
    collections::HashSet,
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver},
    time::{Duration, Instant},
};

pub struct WatchPlugin {
    watch_entries: Vec<WatchEntry>,
    watch_mode: bool,
    stop_on_fail: bool,
    // We'll store a pointer to whether we need to re-run the entire pipeline. In a real setup
    // you might want a reference to the GraphManager or some approach to re-run tasks.
    // Here we'll keep it simpler and just store a flag we can read in on_after_run.
}

#[derive(Debug)]
pub struct WatchEntry {
    #[allow(dead_code)]
    pub task_name: String,
    pub glob_set: GlobSet,
    pub ignore_set: Option<GlobSet>,
    pub directories_to_watch: HashSet<PathBuf>,
    pub debounce_ms: u64,
}

impl WatchPlugin {
    pub fn new(watch_mode: bool, stop_on_fail: bool) -> Self {
        Self {
            watch_entries: Vec::new(),
            watch_mode,
            stop_on_fail,
        }
    }

    pub fn get_watch_entry_count(&self) -> usize {
        self.watch_entries.len()
    }

    pub fn is_watch_mode(&self) -> bool {
        self.watch_mode
    }

    // We'll store a pointer to whether we need to re-run the entire pipeline. In a real setup
    // you might want a reference to the GraphManager or some approach to re-run tasks.
    // Here we'll keep it simpler and just store a flag we can read in on_after_run.

    fn create_watcher() -> Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
        debug!("Creating file watcher with 1s poll interval");
        let (tx, rx) = mpsc::channel();
        let watcher = RecommendedWatcher::new(
            move |res| {
                let _ = tx.send(res);
            },
            NotifyConfig::default().with_poll_interval(Duration::from_secs(1)),
        )
        .map_err(|e| BodoError::PluginError(format!("Failed to create watcher: {}", e)))?;
        Ok((watcher, rx))
    }

    // This function is intended for testing purposes.
    pub fn create_watcher_test() -> Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
        Self::create_watcher()
    }

    pub fn filter_changed_paths(
        &self,
        changed_paths: &[PathBuf],
        entry: &WatchEntry,
    ) -> Vec<PathBuf> {
        let mut matched = vec![];

        let cwd = match std::env::current_dir() {
            Ok(path) => path,
            Err(e) => {
                warn!("Failed to get current directory: {}", e);
                return vec![];
            }
        };

        for changed_path in changed_paths {
            let changed_abs = match changed_path.canonicalize() {
                Ok(p) => p,
                Err(e) => {
                    warn!(
                        "Failed to canonicalize path {}: {}",
                        changed_path.display(),
                        e
                    );
                    continue;
                }
            };
            let mut is_under_watch_dir = false;
            for watch_dir in &entry.directories_to_watch {
                let watch_abs = match watch_dir.canonicalize() {
                    Ok(p) => p,
                    Err(e) => {
                        warn!(
                            "Failed to canonicalize watch dir {}: {}",
                            watch_dir.display(),
                            e
                        );
                        continue;
                    }
                };
                if changed_abs.starts_with(&watch_abs) {
                    is_under_watch_dir = true;
                    break;
                }
            }
            if !is_under_watch_dir {
                continue;
            }
            let rel_path = match changed_abs.strip_prefix(&cwd) {
                Ok(p) => p,
                Err(_) => continue,
            };
            let rel_str = rel_path.to_string_lossy().replace('\\', "/");
            if !entry.glob_set.is_match(&rel_str) {
                continue;
            }
            if let Some(ignore) = &entry.ignore_set {
                if ignore.is_match(&rel_str) {
                    continue;
                }
            }
            matched.push(changed_path.clone());
        }
        matched
    }

    pub fn find_base_directory(patt: &str) -> Option<PathBuf> {
        let path = Path::new(patt);
        if patt.starts_with("**/") {
            return Some(PathBuf::from("."));
        }
        let components = path.components().collect::<Vec<_>>();
        let first_wildcard = components
            .iter()
            .position(|c| c.as_os_str().to_string_lossy().contains('*'));
        let base = if let Some(idx) = first_wildcard {
            if idx == 0 {
                PathBuf::from(".")
            } else {
                PathBuf::from_iter(&components[..idx])
            }
        } else if path.is_dir() {
            path.to_path_buf()
        } else {
            path.parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf()
        };
        if base.as_os_str().is_empty() {
            Some(PathBuf::from("."))
        } else {
            Some(base)
        }
    }

    // Rest of the implementation...
    // ... (Rest of the methods remain unchanged)
}

// Function moved outside of impl block
// A small helper that just returns some BodoConfig with "scripts/" as script dirs
fn graph_manager_config_snapshot() -> Result<crate::config::BodoConfig> {
    Ok(crate::config::BodoConfig {
        scripts_dirs: Some(vec!["scripts/".into()]),
        ..Default::default()
    })
}

impl Default for WatchPlugin {
    fn default() -> Self {
        Self::new(false, false)
    }
}

impl Plugin for WatchPlugin {
    fn name(&self) -> &'static str {
        "WatchPlugin"
    }

    fn priority(&self) -> i32 {
        90
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn on_init(&mut self, config: &PluginConfig) -> Result<()> {
        // If --watch or --auto_watch was passed, watch_mode might be updated:
        if config.watch {
            self.watch_mode = true;
        }
        Ok(())
    }

    fn on_graph_build(&mut self, graph: &mut Graph) -> Result<()> {
        // First check if any task has auto_watch enabled
        if !self.watch_mode {
            for node in &graph.nodes {
                if let NodeKind::Task(task_data) = &node.kind {
                    if let Some(WatchConfig {
                        auto_watch: true, ..
                    }) = &task_data.watch
                    {
                        // Found auto_watch == true, enable watch mode only if BODO_NO_WATCH is not set
                        if std::env::var("BODO_NO_WATCH").is_err() {
                            self.watch_mode = true;
                            break;
                        }
                    }
                }
            }
        }

        // If watch_mode is still false, do nothing
        if !self.watch_mode {
            return Ok(());
        }

        // Process watch entries for all tasks that have watch configs
        for node in &graph.nodes {
            if let NodeKind::Task(task_data) = &node.kind {
                if let Some(WatchConfig {
                    patterns,
                    debounce_ms,
                    ignore_patterns,
                    ..
                }) = &task_data.watch
                {
                    let mut gbuilder = GlobSetBuilder::new();
                    for patt in patterns {
                        let glob = Glob::new(patt).map_err(|e| {
                            BodoError::PluginError(format!(
                                "Invalid watch pattern '{}': {}",
                                patt, e
                            ))
                        })?;
                        gbuilder.add(glob);
                    }
                    let glob_set = gbuilder.build().map_err(|e| {
                        BodoError::PluginError(format!("Could not build globset: {}", e))
                    })?;

                    let mut ignore_builder = GlobSetBuilder::new();
                    let mut have_ignores = false;
                    for ig in ignore_patterns {
                        let g = Glob::new(ig).map_err(|e| {
                            BodoError::PluginError(format!(
                                "Invalid ignore pattern '{}': {}",
                                ig, e
                            ))
                        })?;
                        ignore_builder.add(g);
                        have_ignores = true;
                    }
                    let ignore_set = if have_ignores {
                        Some(ignore_builder.build().map_err(|e| {
                            BodoError::PluginError(format!("Failed building ignore globset: {}", e))
                        })?)
                    } else {
                        None
                    };

                    let mut dirs = HashSet::new();
                    for patt in patterns {
                        if let Some(dir) = Self::find_base_directory(patt) {
                            dirs.insert(dir);
                        }
                    }
                    self.watch_entries.push(WatchEntry {
                        task_name: task_data.name.clone(),
                        glob_set,
                        ignore_set,
                        directories_to_watch: dirs,
                        debounce_ms: *debounce_ms,
                    });
                }
            }
        }
        Ok(())
    }

    fn on_after_run(&mut self, _graph: &mut Graph) -> Result<()> {
        if !self.watch_mode || self.watch_entries.is_empty() {
            return Ok(());
        }

        let (mut watcher, rx) = WatchPlugin::create_watcher()?;
        let mut all_dirs = HashSet::new();
        let mut max_debounce = 500;

        for entry in &self.watch_entries {
            max_debounce = max_debounce.max(entry.debounce_ms);
            all_dirs.extend(entry.directories_to_watch.iter().cloned());
        }

        for d in &all_dirs {
            if d.is_dir() {
                if let Err(e) = watcher.watch(d, RecursiveMode::Recursive) {
                    warn!("WatchPlugin: Failed to watch '{}': {}", d.display(), e);
                }
            }
        }

        println!("Watching for file changes. Press Ctrl-C to stop...");

        let mut last_run = Instant::now();

        // We block here until the user kills the process
        loop {
            let event = match rx.recv() {
                Ok(e) => e,
                Err(_) => {
                    debug!("WatchPlugin: Watcher channel closed. Exiting loop.");
                    break;
                }
            };
            let event = match event {
                Ok(ev) => ev,
                Err(err) => {
                    warn!("WatchPlugin: Watch error: {}", err);
                    continue;
                }
            };

            let now = Instant::now();
            let since_last = now.duration_since(last_run);
            if since_last < Duration::from_millis(max_debounce) {
                debug!("Debouncing event (too soon after last run)");
                continue;
            }
            last_run = now;

            let changed_paths = event.paths;
            if changed_paths.is_empty() {
                continue;
            }
            // For each watch entry, see if anything matched
            for entry in &self.watch_entries {
                let matched = self.filter_changed_paths(&changed_paths, entry);
                if !matched.is_empty() {
                    println!(
                        "Files changed for task '{}': re-running pipeline...",
                        entry.task_name
                    );
                    // Re-run the entire plugin pipeline if desired.
                    // We'll forcibly rebuild everything in a fresh manager or re-run the same manager.

                    // For demonstration, let's do a trivial approach:
                    // Re-run the entire pipeline from scratch.
                    // Usually you'd keep a reference to GraphManager:
                    let mut new_manager = crate::manager::GraphManager::new();
                    new_manager.build_graph(graph_manager_config_snapshot()?)?;
                    // Re-register the same plugins with updated watch mode, etc.
                    new_manager
                        .register_plugin(Box::new(crate::plugins::env_plugin::EnvPlugin::new()));
                    new_manager
                        .register_plugin(Box::new(crate::plugins::path_plugin::PathPlugin::new()));
                    new_manager.register_plugin(Box::new(
                        crate::plugins::concurrent_plugin::ConcurrentPlugin::new(),
                    ));
                    new_manager.register_plugin(Box::new(
                        crate::plugins::prefix_plugin::PrefixPlugin::new(),
                    ));
                    new_manager
                        .register_plugin(Box::new(WatchPlugin::new(true, self.stop_on_fail)));
                    new_manager.register_plugin(Box::new(
                        crate::plugins::execution_plugin::ExecutionPlugin::new(),
                    ));
                    new_manager.register_plugin(Box::new(
                        crate::plugins::timeout_plugin::TimeoutPlugin::new(),
                    ));

                    // If we had some way to remember which task triggered, we could pass that again.
                    // For demonstration, we pass the same 'entry.task_name':
                    let mut options = serde_json::Map::new();
                    options.insert(
                        "task".to_string(),
                        serde_json::Value::String(entry.task_name.clone()),
                    );
                    let plugin_config = PluginConfig {
                        fail_fast: true,
                        watch: true,
                        list: false,
                        dry_run: false,
                        enable_recovery: false,
                        max_retry_attempts: None,
                        initial_retry_backoff: None,
                        options: Some(options),
                    };
                    if let Err(e) = new_manager.run_plugins(Some(plugin_config)) {
                        error!("WatchPlugin: re-run failed: {}", e);
                        if self.stop_on_fail {
                            warn!("WatchPlugin: Stopping watch loop due to re-run failure");
                            return Ok(());
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
