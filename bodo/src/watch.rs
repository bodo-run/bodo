use crate::task::TaskManager;
use notify::{RecommendedWatcher, RecursiveMode, Watcher, Event, EventKind};
use std::sync::mpsc::channel;
use std::time::Duration;
use std::path::Path;

pub struct WatchManager {
    task_manager: TaskManager,
}

impl WatchManager {
    pub fn new(task_manager: TaskManager) -> Self {
        Self { task_manager }
    }

    pub fn watch_and_run(&self, group: &str, subtask: Option<&str>) -> Result<(), String> {
        println!("Starting watch mode for task: {}", group);
        
        let (tx, rx) = channel();
        let mut watcher = RecommendedWatcher::new(
            move |result: Result<Event, notify::Error>| {
                if let Ok(event) = result {
                    let _ = tx.send(event);
                }
            },
            notify::Config::default(),
        ).map_err(|e| format!("Failed to create watcher: {}", e))?;

        // Watch the current directory
        watcher.watch(Path::new("."), RecursiveMode::Recursive)
            .map_err(|e| format!("Failed to watch directory: {}", e))?;

        // Initial run
        if let Err(e) = self.task_manager.run_task(group, subtask) {
            eprintln!("Initial task run failed: {}", e);
        }

        loop {
            match rx.recv() {
                Ok(event) => {
                    match event.kind {
                        EventKind::Create(_) |
                        EventKind::Modify(_) |
                        EventKind::Remove(_) => {
                            println!("\nChange detected, re-running task...");
                            if let Err(e) = self.task_manager.run_task(group, subtask) {
                                eprintln!("Task run failed: {}", e);
                            }
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    return Err(format!("Watch error: {}", e));
                }
            }
        }
    }
} 