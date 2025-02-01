use crate::{
    config::WatchConfig,
    errors::BodoError,
    graph::{Graph, NodeKind},
    plugin::{Plugin, PluginConfig},
    Result,
};
use async_trait::async_trait;
use globset::{Glob, GlobSet, GlobSetBuilder};
use notify::{Config as NotifyConfig, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    any::Any,
    collections::HashSet,
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver},
    time::{Duration, Instant},
};

/// Our updated WatchPlugin that supports glob patterns like "src/**/*.rs".
/// It watches each pattern's directory (recursively), then filters events
/// by matching them against the glob/ignore sets.
pub struct WatchPlugin {
    watch_entries: Vec<WatchEntry>,
    watch_mode: bool,
}

#[derive(Debug)]
struct WatchEntry {
    task_name: String,
    // We'll convert the user's patterns into a GlobSet for matching, and
    // also keep a set of directories to actually watch.
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

    /// Filters changed paths to those that match the watch patterns and do *not* match ignore globs.
    /// Returns a vector of all paths that survive filtering.
    fn filter_changed_paths(&self, changed_paths: &[PathBuf], entry: &WatchEntry) -> Vec<PathBuf> {
        changed_paths
            .iter()
            .filter_map(|p| {
                // If path is None or not valid UTF-8, skip
                let path_str = p.to_str()?;
                // Must match at least one "include" glob
                if !entry.glob_set.is_match(path_str) {
                    return None;
                }
                // If there's an ignore set, skip if matched
                if let Some(ref ignore_set) = entry.ignore_set {
                    if ignore_set.is_match(path_str) {
                        return None;
                    }
                }
                Some(p.clone())
            })
            .collect()
    }

    /// Reruns the specified task synchronously. For a full re-run with concurrency, you'd call
    /// the same logic used in ExecutionPlugin or the GraphManager. Here we just do a naive shell spawn.
    async fn rerun_task(&self, graph: &mut Graph, task_name: &str) -> Result<()> {
        if let Some(&node_id) = graph.task_registry.get(task_name) {
            if let NodeKind::Task(task_data) = &graph.nodes[node_id as usize].kind {
                if let Some(cmd) = &task_data.command {
                    println!("WatchPlugin: Running task: '{t}'", t = task_data.name);
                    let status = std::process::Command::new("sh").arg("-c").arg(cmd).status();
                    match status {
                        Ok(s) if !s.success() => {
                            return Err(BodoError::PluginError(format!(
                                "Task '{}' failed (exit={:?})",
                                task_data.name,
                                s.code()
                            )));
                        }
                        Err(e) => {
                            return Err(BodoError::PluginError(format!(
                                "Error spawning '{}': {}",
                                task_data.name, e
                            )));
                        }
                        _ => {}
                    }
                }
            }
        } else {
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
        90 // Ensure it runs after concurrency transforms and before final execution
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    async fn on_init(&mut self, config: &PluginConfig) -> Result<()> {
        self.watch_mode = config.watch;
        Ok(())
    }

    /// Collect all tasks that have WatchConfig, build globsets and figure out which directories
    /// to actually watch. We'll do the real watching in on_after_run.
    async fn on_graph_build(&mut self, graph: &mut Graph) -> Result<()> {
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
                    let mut builder = GlobSetBuilder::new();
                    for patt in patterns {
                        // Build the glob
                        let glob = Glob::new(patt).map_err(|e| {
                            BodoError::PluginError(format!(
                                "Invalid glob pattern '{}': {}",
                                patt, e
                            ))
                        })?;
                        builder.add(glob);
                    }
                    let glob_set = builder.build().map_err(|e| {
                        BodoError::PluginError(format!("Could not build globset: {}", e))
                    })?;

                    // Build ignore set if any
                    let mut ignore_builder = GlobSetBuilder::new();
                    let mut have_ignores = false;
                    for ignore in ignore_patterns {
                        let ig = Glob::new(ignore).map_err(|e| {
                            BodoError::PluginError(format!(
                                "Invalid ignore pattern '{}': {}",
                                ignore, e
                            ))
                        })?;
                        ignore_builder.add(ig);
                        have_ignores = true;
                    }
                    let ignore_set = if have_ignores {
                        Some(ignore_builder.build().map_err(|e| {
                            BodoError::PluginError(format!("Could not build ignore globset: {}", e))
                        })?)
                    } else {
                        None
                    };

                    // From each pattern, extract a directory to watch. We watch recursively,
                    // then filter inside the event. For example, "src/**/*.rs" => watch "src".
                    let mut dirs_to_watch = HashSet::new();
                    for patt in patterns {
                        if let Some(dir) = find_base_directory(patt) {
                            dirs_to_watch.insert(dir);
                        }
                    }

                    // Collect this watch entry
                    let entry = WatchEntry {
                        task_name: task_data.name.clone(),
                        glob_set,
                        ignore_set,
                        directories_to_watch: dirs_to_watch,
                        debounce_ms: *debounce_ms,
                    };
                    self.watch_entries.push(entry);
                }
            }
        }

        // Print some debug info if desired
        for entry in &self.watch_entries {
            println!(
                "WatchPlugin: Will watch task '{}' with directories: {:?}",
                entry.task_name, entry.directories_to_watch
            );
        }

        Ok(())
    }

    /// After everything runs once, we start the watch loop (if watch mode is on).
    async fn on_after_run(&mut self, graph: &mut Graph) -> Result<()> {
        if !self.watch_mode || self.watch_entries.is_empty() {
            return Ok(());
        }

        // We'll watch the union of all directories from all watch_entries.
        let (mut watcher, rx) = Self::create_watcher()?;
        let mut all_dirs = HashSet::new();
        let mut max_debounce = 500;
        for entry in &self.watch_entries {
            max_debounce = max_debounce.max(entry.debounce_ms);
            for d in &entry.directories_to_watch {
                all_dirs.insert(d.clone());
            }
        }

        for dir in &all_dirs {
            // If the user references ".", "src", or any folder that exists, watch recursively.
            if dir.exists() && dir.is_dir() {
                if let Err(e) = watcher.watch(dir, RecursiveMode::Recursive) {
                    eprintln!(
                        "WatchPlugin: Failed to watch directory '{}': {}",
                        dir.display(),
                        e
                    );
                }
            } else {
                eprintln!(
                    "WatchPlugin: Directory '{}' does not exist or is not a directory.",
                    dir.display()
                );
            }
        }

        println!("WatchPlugin: Initial watch setup complete. Listening for changes...");
        let mut last_run = Instant::now();

        // Main watch loop
        loop {
            let event = match rx.recv() {
                Ok(e) => e,
                Err(_) => {
                    eprintln!("WatchPlugin: Watcher channel closed. Exiting watch loop.");
                    break;
                }
            };
            let event = match event {
                Ok(ev) => ev,
                Err(err) => {
                    eprintln!("WatchPlugin: Watch error: {}", err);
                    continue;
                }
            };

            let now = Instant::now();
            if now.duration_since(last_run) < Duration::from_millis(max_debounce) {
                // skip consecutive events in a short window
                continue;
            }
            last_run = now;

            let changed_paths = event.paths;
            if changed_paths.is_empty() {
                continue;
            }

            // For each watch entry, see if anything matched
            let mut any_match = false;
            for entry in &self.watch_entries {
                let matched = self.filter_changed_paths(&changed_paths, entry);
                if !matched.is_empty() {
                    any_match = true;
                }
            }

            if any_match {
                println!(
                    "WatchPlugin: Detected changes in at least one watched pattern. Re-running..."
                );
                // We re-run *all* watch-enabled tasks. Another approach:
                // re-run only the tasks whose watchers matched something.
                for entry in &self.watch_entries {
                    let matched = self.filter_changed_paths(&changed_paths, entry);
                    if !matched.is_empty() {
                        if matched.len() < 6 {
                            println!("WatchPlugin: Files changed for '{}':", entry.task_name);
                            for p in &matched {
                                println!("   -> {}", p.display());
                            }
                        } else {
                            println!(
                                "WatchPlugin: {} changes for task '{}', showing first 5:",
                                matched.len(),
                                entry.task_name
                            );
                            for p in matched.iter().take(5) {
                                println!("   -> {}", p.display());
                            }
                        }
                        if let Err(err) = self.rerun_task(graph, &entry.task_name).await {
                            eprintln!(
                                "WatchPlugin: Task '{}' failed with error: {:?}",
                                entry.task_name, err
                            );
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

/// Attempt to figure out which directory to watch for the given glob pattern.
/// Example:
///   "src/**/*.rs" -> "src"
///   "./tests/**/*.rs" -> "tests"
///   "myfile.rs" -> "."
fn find_base_directory(patt: &str) -> Option<PathBuf> {
    // If there's a directory part, extract it. Otherwise default to "."
    // For example, "src/**/*.rs" => "src"
    // Use Path::parent logic after removing trailing wildcard segments.
    let path = Path::new(patt);

    // If pattern is something like "**/*.rs", we watch "."
    if patt.contains("**/") && !patt.contains('/') {
        return Some(PathBuf::from("."));
    }

    // If there's at least one slash, let's try everything before the first wildcard
    // or the slash nearest the wildcard. Or we can do a simpler approach:
    // look at the path up to the first wildcard component.
    let components = path.components().collect::<Vec<_>>();
    let first_wildcard = components
        .iter()
        .position(|c| c.as_os_str().to_string_lossy().contains('*'));

    let base = if let Some(wc_idx) = first_wildcard {
        // Join everything before wc_idx
        if wc_idx == 0 {
            // pattern starts with wildcard => watch "."
            PathBuf::from(".")
        } else {
            PathBuf::from_iter(&components[..wc_idx])
        }
    } else {
        // No wildcard => watch the parent directory if possible
        if !path.is_dir() {
            // e.g. "Cargo.toml" => watch "."
            path.parent()
                .map(|p| p.to_path_buf())
                .or_else(|| Some(PathBuf::from(".")))?
        } else {
            // It's an actual directory with no wildcard
            path.to_path_buf()
        }
    };

    // If the resulting path is empty, default to "."
    if base.as_os_str().is_empty() {
        Some(PathBuf::from("."))
    } else {
        Some(base)
    }
}
