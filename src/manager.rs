use crate::config::{BodoConfig, TaskConfig};

pub struct TaskManager {
    pub config: BodoConfig,
}

impl TaskManager {
    pub fn new(config: BodoConfig) -> Self {
        TaskManager { config }
    }

    pub fn validate_task(&self, task: &TaskConfig) -> bool {
        task.validate().is_ok()
    }
}
