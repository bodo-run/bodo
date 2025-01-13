use crate::task::TaskManager;
use std::error::Error;

#[allow(dead_code)]
pub struct WatchManager<'a> {
    task_manager: TaskManager<'a>,
}

impl<'a> WatchManager<'a> {
    pub fn new(task_manager: TaskManager<'a>) -> Self {
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
