use crate::{
    config::WatchConfig,
    errors::BodoError,
    graph::{Graph, NodeKind},
    plugin::{Plugin, PluginConfig},
    Result,
};
use async_trait::async_trait;
use globset::{Glob, GlobSet, GlobSetBuilder};
use log::{debug, warn};
use notify::{Config as NotifyConfig, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    any::Any,
    collections::HashSet,
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver},
    time::{Duration, Instant},
};

/// Plugin that watches for file changes and re-runs only tasks whose WatchConfig matched something.
pub struct WatchPlugin {
    watch_entries: Vec<WatchEntry>,
    watch_mode: bool,
}

#[derive(Debug)]
struct WatchEntry {
    task_name: String,
    glob_set: GlobSet,
    ignore_set: Option<GlobSet>,
    directories_to_watch: HashSet<PathBuf>,
    debounce_ms: u64,
}

impl WatchPlugin {
    pub fn new() -> Self {
        Self {
            watch_entries: Vec::new(),
            watch_mode: false,
        }
    }

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

        debug!("File watcher created successfully");
        Ok((watcher, rx))
    }

    /// For each changed file, see if it matches our glob/ignore sets.
    /// We only accept it if it's under one of the watched directories,
    /// then strip that directory prefix so the remaining path can be tested
    /// against a pattern like "tests/**/*.rs".
    fn filter_changed_paths(&self, changed_paths: &[PathBuf], entry: &WatchEntry) -> Vec<PathBuf> {
        debug!(
            "Filtering {} changed paths for task '{}'",
            changed_paths.len(),
            entry.task_name
        );
        let mut matched = vec![];

        let cwd = match std::env::current_dir() {
            Ok(path) => path,
            Err(e) => {
                warn!("Failed to get current directory: {}", e);
                return vec![];
            }
        };

        for changed_path in changed_paths {
            debug!("Processing changed path: {}", changed_path.display());
            let changed_abs = match changed_path.canonicalize() {
                Ok(p) => {
                    debug!("Canonicalized path: {}", p.display());
                    p
                }
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
                debug!("Checking against watch dir: {}", watch_dir.display());
                let watch_abs = match watch_dir.canonicalize() {
                    Ok(p) => {
                        debug!("Canonicalized watch dir: {}", p.display());
                        p
                    }
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
                debug!("Path is not under any watch directory");
                continue;
            }

            let rel_path = match changed_abs.strip_prefix(&cwd) {
                Ok(p) => p,
                Err(e) => {
                    warn!("Failed to strip project prefix: {}", e);
                    continue;
                }
            };

            let rel_str = rel_path.to_string_lossy().replace('\\', "/");
            debug!("Testing relative path against globs: {}", rel_str);

            if !entry.glob_set.is_match(&rel_str) {
                debug!("Path did not match include patterns");
                continue;
            }
            debug!("Path matched include patterns");

            if let Some(ignore) = &entry.ignore_set {
                if ignore.is_match(&rel_str) {
                    debug!("Path matched ignore patterns, skipping");
                    continue;
                }
                debug!("Path did not match ignore patterns");
            }

            matched.push(changed_path.clone());
        }

        debug!(
            "Found {} matching paths for task '{}'",
            matched.len(),
            entry.task_name
        );
        matched
    }

    /// Re-run a single task. For a bigger system, you'd call the same manager logic
    /// your ExecutionPlugin uses, but here we just spawn a shell command for illustration.
    async fn rerun_task(&self, graph: &mut Graph, task_name: &str) -> Result<()> {
        debug!("Attempting to rerun task: '{}'", task_name);
        if let Some(&node_id) = graph.task_registry.get(task_name) {
            debug!("Found task node_id: {}", node_id);
            if let NodeKind::Task(task_data) = &graph.nodes[node_id as usize].kind {
                if let Some(cmd) = &task_data.command {
                    debug!("Executing command: {}", cmd);
                    debug!("Running task: '{}'", task_data.name);
                    let status = std::process::Command::new("sh").arg("-c").arg(cmd).status();
                    match status {
                        Ok(s) if !s.success() => {
                            debug!("Task failed with exit code: {:?}", s.code());
                            return Err(BodoError::PluginError(format!(
                                "Task '{}' failed (exit={:?})",
                                task_data.name,
                                s.code()
                            )));
                        }
                        Ok(_) => {
                            debug!("Task completed successfully");
                        }
                        Err(e) => {
                            debug!("Failed to spawn task: {}", e);
                            return Err(BodoError::PluginError(format!(
                                "Error spawning '{}': {}",
                                task_data.name, e
                            )));
                        }
                    }
                } else {
                    debug!("Task has no command defined");
                }
            }
        } else {
            debug!("Task '{}' not found in registry", task_name);
            return Err(BodoError::PluginError(format!(
                "Task '{}' not found for watch re-run",
                task_name
            )));
        }
        Ok(())
    }
}

impl Default for WatchPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for WatchPlugin {
    fn name(&self) -> &'static str {
        "WatchPlugin"
    }

    fn priority(&self) -> i32 {
        90 // after concurrency transforms, before final execution
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    async fn on_init(&mut self, config: &PluginConfig) -> Result<()> {
        self.watch_mode = config.watch;
        Ok(())
    }

    /// Gather tasks that have watch configs. Build GlobSets and figure out which directories to watch.
    async fn on_graph_build(&mut self, graph: &mut Graph) -> Result<()> {
        debug!("Starting graph build in watch mode: {}", self.watch_mode);
        if !self.watch_mode {
            return Ok(());
        }

        for node in &graph.nodes {
            if let NodeKind::Task(task_data) = &node.kind {
                if let Some(WatchConfig {
                    patterns,
                    debounce_ms,
                    ignore_patterns,
                }) = &task_data.watch
                {
                    debug!("Processing watch config for task: '{}'", task_data.name);
                    debug!("Watch patterns: {:?}", patterns);
                    debug!("Ignore patterns: {:?}", ignore_patterns);

                    let mut gbuilder = GlobSetBuilder::new();
                    for patt in patterns {
                        debug!("Building glob for pattern: {}", patt);
                        let glob = Glob::new(patt).map_err(|e| {
                            debug!("Invalid watch pattern: {}", e);
                            BodoError::PluginError(format!(
                                "Invalid watch pattern '{}': {}",
                                patt, e
                            ))
                        })?;
                        gbuilder.add(glob);
                    }
                    let glob_set = gbuilder.build().map_err(|e| {
                        debug!("Failed to build globset: {}", e);
                        BodoError::PluginError(format!("Could not build globset: {}", e))
                    })?;
                    debug!("Successfully built include globset");

                    let mut ignore_builder = GlobSetBuilder::new();
                    let mut have_ignores = false;
                    for ig in ignore_patterns {
                        debug!("Building ignore glob for pattern: {}", ig);
                        let g = Glob::new(ig).map_err(|e| {
                            debug!("Invalid ignore pattern: {}", e);
                            BodoError::PluginError(format!(
                                "Invalid ignore pattern '{}': {}",
                                ig, e
                            ))
                        })?;
                        ignore_builder.add(g);
                        have_ignores = true;
                    }
                    let ignore_set = if have_ignores {
                        debug!("Building ignore globset");
                        Some(ignore_builder.build().map_err(|e| {
                            BodoError::PluginError(format!("Failed building ignore globset: {}", e))
                        })?)
                    } else {
                        debug!("No ignore patterns to build");
                        None
                    };

                    let mut dirs = HashSet::new();
                    for patt in patterns {
                        debug!("Finding base directory for pattern: {}", patt);
                        if let Some(dir) = find_base_directory(patt) {
                            debug!("Found base directory: {}", dir.display());
                            dirs.insert(dir);
                        } else {
                            debug!("No base directory found for pattern");
                        }
                    }

                    debug!("Adding watch entry for task '{}'", task_data.name);
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

        for entry in &self.watch_entries {
            debug!(
                "Will watch task '{}' over dirs: {:?}",
                entry.task_name, entry.directories_to_watch
            );
        }
        debug!(
            "Graph build complete with {} watch entries",
            self.watch_entries.len()
        );

        Ok(())
    }

    /// After initial run, start the watch loop: re-run only tasks whose watchers matched something.
    async fn on_after_run(&mut self, graph: &mut Graph) -> Result<()> {
        debug!("Starting after_run in watch mode: {}", self.watch_mode);
        if !self.watch_mode || self.watch_entries.is_empty() {
            debug!("Watch mode disabled or no entries, exiting");
            return Ok(());
        }

        let (mut watcher, rx) = Self::create_watcher()?;
        let mut all_dirs = HashSet::new();
        let mut max_debounce = 500;

        for entry in &self.watch_entries {
            debug!("Processing watch entry for task: '{}'", entry.task_name);
            max_debounce = max_debounce.max(entry.debounce_ms);
            debug!("Updated max_debounce to: {}ms", max_debounce);
            all_dirs.extend(entry.directories_to_watch.iter().cloned());
        }

        for d in &all_dirs {
            debug!("Setting up watch for directory: {}", d.display());
            if d.is_dir() {
                if let Err(e) = watcher.watch(d, RecursiveMode::Recursive) {
                    warn!("WatchPlugin: Failed to watch '{}': {}", d.display(), e);
                } else {
                    debug!("Successfully watching directory: {}", d.display());
                }
            } else {
                debug!(
                    "WatchPlugin: '{}' not found or not a directory.",
                    d.display()
                );
            }
        }

        println!("Watching for file changes. Press Ctrl-C to stop...");

        debug!("Watch setup complete, entering main loop");
        let mut last_run = Instant::now();

        loop {
            let event = match rx.recv() {
                Ok(e) => e,
                Err(_) => {
                    debug!("WatchPlugin: Watcher channel closed. Exiting watch loop.");
                    break;
                }
            };
            let event = match event {
                Ok(ev) => {
                    debug!("Received file system event: {:?}", ev.kind);
                    ev
                }
                Err(err) => {
                    warn!("WatchPlugin: Watch error: {}", err);
                    continue;
                }
            };

            let now = Instant::now();
            let since_last = now.duration_since(last_run);
            debug!("Time since last run: {}ms", since_last.as_millis());
            if since_last < Duration::from_millis(max_debounce) {
                debug!("Debouncing event (too soon after last run)");
                continue;
            }
            last_run = now;

            let changed_paths = event.paths;
            if changed_paths.is_empty() {
                debug!("Event contained no paths, skipping");
                continue;
            }
            debug!("Processing {} changed paths", changed_paths.len());

            for entry in &self.watch_entries {
                debug!("Checking changes for task: '{}'", entry.task_name);
                let matched = self.filter_changed_paths(&changed_paths, entry);
                if !matched.is_empty() {
                    debug!("Found {} matching paths", matched.len());
                    if matched.len() < 6 {
                        debug!("Changes for '{}':", entry.task_name);
                        for p in &matched {
                            debug!("  -> {}", p.display());
                        }
                    } else {
                        debug!(
                            "{} changes for '{}', showing first 5:",
                            matched.len(),
                            entry.task_name
                        );
                        for p in matched.iter().take(5) {
                            debug!("  -> {}", p.display());
                        }
                    }

                    debug!("Triggering rerun for task: '{}'", entry.task_name);
                    if let Err(e) = self.rerun_task(graph, &entry.task_name).await {
                        warn!("WatchPlugin: Task '{}' failed: {}", entry.task_name, e);
                    }
                } else {
                    debug!("No matching paths for task: '{}'", entry.task_name);
                }
            }
        }

        debug!("Watch loop terminated");
        Ok(())
    }
}

/// Extract a top-level directory (or `.`) to watch for a pattern like "tests/**/*.rs"
fn find_base_directory(patt: &str) -> Option<PathBuf> {
    debug!("Finding base directory for pattern: {}", patt);
    let path = Path::new(patt);

    if patt.starts_with("**/") {
        debug!("Pattern starts with '**/', using '.' as base");
        return Some(PathBuf::from("."));
    }

    let components = path.components().collect::<Vec<_>>();
    debug!("Path components: {:?}", components);
    let first_wildcard = components
        .iter()
        .position(|c| c.as_os_str().to_string_lossy().contains('*'));

    let base = if let Some(idx) = first_wildcard {
        debug!("Found wildcard at component index: {}", idx);
        if idx == 0 {
            debug!("Wildcard at start, using '.' as base");
            PathBuf::from(".")
        } else {
            debug!("Using components before wildcard as base");
            PathBuf::from_iter(&components[..idx])
        }
    } else {
        debug!("No wildcard found in pattern");
        if path.is_dir() {
            debug!("Pattern is an existing directory");
            path.to_path_buf()
        } else {
            debug!("Using parent directory as base");
            path.parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf()
        }
    };

    if base.as_os_str().is_empty() {
        debug!("Empty base path, using '.' instead");
        Some(PathBuf::from("."))
    } else {
        debug!("Using base path: {}", base.display());
        Some(base)
    }
}
