use crate::{
    config::WatchConfig,
    errors::BodoError,
    graph::{Graph, NodeKind, TaskData},
    plugin::{Plugin, PluginConfig},
    Result,
};
use async_trait::async_trait;
use notify::{
    event::ModifyKind, Config as NotifyConfig, Event, EventKind, RecommendedWatcher, RecursiveMode,
    Watcher,
};
use std::{
    any::Any,
    collections::HashSet,
    path::PathBuf,
    sync::mpsc::{self, Receiver},
    time::{Duration, Instant},
};

/// Our updated WatchPlugin that actually watches files, then re-runs tasks.
pub struct WatchPlugin {
    // internal storage of patterns to watch
    watch_entries: Vec<WatchEntry>,
    // The CLI "watch" flag, or any other data we need to decide if watch mode is active
    watch_mode: bool,
}

/// Each task that has watch config can add multiple patterns
#[derive(Debug)]
struct WatchEntry {
    task_name: String,
    patterns: Vec<String>,
    ignore_patterns: Vec<String>,
    debounce_ms: u64,
}

impl WatchPlugin {
    pub fn new() -> Self {
        Self {
            watch_entries: Vec::new(),
            watch_mode: false,
        }
    }

    /// Create the watcher + channel. Using the non-async `mpsc` approach for simplicity.
    fn create_watcher() -> Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
        let (tx, rx) = mpsc::channel();
        let watcher = RecommendedWatcher::new(
            move |res| {
                tx.send(res).ok();
            },
            // If needed, tweak the config
            NotifyConfig::default().with_poll_interval(Duration::from_secs(1)),
        )
        .map_err(|e| BodoError::PluginError(format!("Failed to create watcher: {}", e)))?;

        Ok((watcher, rx))
    }

    /// Extract changed paths from an event. For rename or remove, also handle old path etc.
    fn extract_changed_paths(event: &Event) -> Vec<PathBuf> {
        let mut paths = vec![];
        if !event.paths.is_empty() {
            for p in &event.paths {
                paths.push(p.clone());
            }
        }
        paths
    }

    /// Print the changed files. If too many, summarize.
    fn print_changed_paths_summary(paths: &[PathBuf]) {
        let len = paths.len();
        if len == 0 {
            return;
        }
        const MAX_SHOW: usize = 5;
        if len <= MAX_SHOW {
            println!("WatchPlugin: Detected file changes:");
            for p in paths {
                println!("   => {}", p.display());
            }
        } else {
            println!(
                "WatchPlugin: Detected {} changed files. Showing first {}:",
                len, MAX_SHOW
            );
            for p in paths.iter().take(MAX_SHOW) {
                println!("   => {}", p.display());
            }
        }
    }

    /// Actually re-run a task. We can re-use manager or however you do it.
    async fn rerun_task(&self, graph: &mut Graph, task_name: &str) -> Result<()> {
        if let Some(node_id) = graph.task_registry.get(task_name) {
            let node = &graph.nodes[*node_id as usize];
            if let NodeKind::Task(task_data) = &node.kind {
                println!("WatchPlugin: Running task: '{}'", task_data.name);
                // For a direct approach, run a single command or concurrency.
                // But typically you'd rely on the ExecutionPlugin or manager.
                if let Some(cmd) = &task_data.command {
                    // run a synchronous shell command ignoring errors
                    // or you can spawn it via ProcessManager
                    let status = std::process::Command::new("sh").arg("-c").arg(cmd).status();

                    // If it fails, return error, but that won't kill our watch loop
                    if let Ok(s) = status {
                        if !s.success() {
                            return Err(BodoError::PluginError(format!(
                                "Task '{}' failed (exit={:?})",
                                task_data.name,
                                s.code()
                            )));
                        }
                    } else if let Err(e) = status {
                        return Err(BodoError::PluginError(format!(
                            "Error spawning '{}': {}",
                            task_data.name, e
                        )));
                    }
                }
            }
        } else {
            // Possibly a concurrency group or command node, or just not found
            return Err(BodoError::TaskNotFound(task_name.to_string()));
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
        90 // Lower than ExecutionPlugin (95)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    /// In on_init, we can detect if watch mode was requested (e.g., from config.options).
    async fn on_init(&mut self, config: &PluginConfig) -> Result<()> {
        self.watch_mode = config.watch;
        Ok(())
    }

    /// In on_graph_build, collect tasks that have watch configs.
    /// We store them in `watch_entries` so we can set up watchers in on_after_run.
    async fn on_graph_build(&mut self, graph: &mut Graph) -> Result<()> {
        if !self.watch_mode {
            return Ok(());
        }

        // Collect watch info from tasks
        for node in &graph.nodes {
            if let NodeKind::Task(task_data) = &node.kind {
                if let Some(watch_config) = &task_data.watch {
                    // Save these patterns for when we actually set up watchers
                    let entry = WatchEntry {
                        task_name: task_data.name.clone(),
                        patterns: watch_config.patterns.clone(),
                        ignore_patterns: watch_config.ignore_patterns.clone(),
                        debounce_ms: watch_config.debounce_ms,
                    };
                    self.watch_entries.push(entry);
                }
            }
        }

        // Debug info about which tasks are watch-enabled
        for entry in &self.watch_entries {
            println!(
                "WatchPlugin: Will watch task '{}' with patterns: {:?}",
                entry.task_name, entry.patterns
            );
        }

        Ok(())
    }

    /// After everything is built and run once, we enter a loop:
    /// watch the files, re-run the *same* tasks if changes occur.
    /// We do NOT exit the process if they fail; just log and keep watching.
    async fn on_after_run(&mut self, graph: &mut Graph) -> Result<()> {
        if !self.watch_mode {
            return Ok(());
        }

        // We need to pick which task we're re-running. Usually there's a single "target" task
        // from plugin config or from the manager. We'll guess the user is re-running the same
        // task they initially triggered. So we look for "task" in plugin options if you store it there.
        // For the sake of example, let's just re-run *all* watch-enabled tasks.
        let tasks_to_rerun: Vec<_> = self
            .watch_entries
            .iter()
            .map(|e| e.task_name.clone())
            .collect();

        // If no tasks have watch patterns, just return
        if tasks_to_rerun.is_empty() {
            println!("WatchPlugin: No tasks with watch patterns. Exiting watch mode.");
            return Ok(());
        }

        // Prepare watchers
        let (mut watcher, rx) = Self::create_watcher()?;
        let combined_debounce_ms = self
            .watch_entries
            .iter()
            .map(|e| e.debounce_ms)
            .max()
            .unwrap_or(500);

        // Add paths/patterns
        // For real wildcard support, you'd need something like `globset` or handle expansions yourself.
        // Here, we just watch each pattern as if it's a path.
        // For simplicity, watch them in recursive mode.
        for entry in &self.watch_entries {
            for patt in &entry.patterns {
                let path = PathBuf::from(patt);
                if let Err(e) = watcher.watch(&path, RecursiveMode::Recursive) {
                    eprintln!("WatchPlugin: Failed to watch path '{}': {}", patt, e);
                }
            }
        }

        // Main watch loop
        println!("WatchPlugin: Initial watch setup complete. Listening for changes...");
        let mut last_run = Instant::now();

        loop {
            // 1) Block for an event
            let event = match rx.recv() {
                Ok(e) => e,
                Err(_e) => {
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

            // 2) Debounce logic: We might want to gather multiple events
            //    for combined_debounce_ms before re-running tasks.
            //    For brevity, we do a simpler approach:
            //    Wait out the debounce window if the last run was too recent.
            let now = Instant::now();
            if now.duration_since(last_run) < Duration::from_millis(combined_debounce_ms) {
                // read further events from the channel to skip duplicates in short time
                continue;
            }
            last_run = now;

            // 3) Summarize the changed files. Some events may have multiple paths.
            let changed_paths = Self::extract_changed_paths(&event);
            Self::print_changed_paths_summary(&changed_paths);

            // 4) Re-run tasks. If they fail, do NOT exit.
            //    Here we call the "manager" or whatever logic you use to run tasks again.
            println!(
                "WatchPlugin: Re-running watch-enabled tasks: {:?}",
                tasks_to_rerun
            );
            for task_name in &tasks_to_rerun {
                if let Err(err) = self.rerun_task(graph, task_name).await {
                    eprintln!(
                        "WatchPlugin: Task '{}' failed with error: {:?}",
                        task_name, err
                    );
                    // Don't break, keep going for other tasks
                }
            }

            // 5) Repeat, continuing to watch. We never exit until user Ctrl-C, etc.
        }

        Ok(())
    }
}
