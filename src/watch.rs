use std::error::Error;
use std::path::Path;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::time::Duration;

use crate::task::TaskManager;

pub struct WatchManager<'a> {
    task_manager: TaskManager<'a>,
}

impl<'a> WatchManager<'a> {
    pub fn new(task_manager: TaskManager<'a>) -> Self {
        Self { task_manager }
    }

    pub fn watch_and_run(&self, task_group: &str, subtask: Option<&str>) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
} 