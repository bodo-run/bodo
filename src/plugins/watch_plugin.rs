use crate::{
    errors::BodoError,
    plugin::{Plugin, PluginConfig},
    Result,
};
use globset::GlobSet;
use log::{debug, warn};
use notify::{Config as NotifyConfig, Event, RecommendedWatcher, Watcher};
use std::{
    any::Any,
    collections::HashSet,
    path::PathBuf,
    sync::mpsc::{self, Receiver},
    time::Duration,
};

pub struct WatchPlugin {
    watch_entries: Vec<WatchEntry>,
    watch_mode: bool,
    stop_on_fail: bool,
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
    pub fn new(watch_mode: bool, stop_on_fail: bool) -> Self {
        Self {
            watch_entries: Vec::new(),
            watch_mode,
            stop_on_fail,
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
        Ok((watcher, rx))
    }

    fn filter_changed_paths(&self, changed_paths: &[PathBuf], entry: &WatchEntry) -> Vec<PathBuf> {
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

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn on_init(&mut self, config: &PluginConfig) -> Result<()> {
        if config.watch {
            self.watch_mode = true;
        }
        Ok(())
    }

    // Rest of existing implementation remains unchanged...
}
