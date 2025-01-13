use crate::task::TaskManager;
use std::error::Error;

#[allow(dead_code)]
pub struct WatchManager {
    task_manager: TaskManager,
}

impl WatchManager {
    pub fn new(task_manager: TaskManager) -> Self {
        Self { task_manager }
    }

    pub fn watch_and_run(
        &self,
        _task_group: &str,
        _subtask: Option<&str>,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}
