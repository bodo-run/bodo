use std::any::Any;

use crate::plugin::Plugin;

pub struct ExecutionPlugin {
    task_name: Option<String>,
}

impl ExecutionPlugin {
    pub fn new() -> Self {
        Self { task_name: None }
    }
}

impl Default for ExecutionPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for ExecutionPlugin {
    fn name(&self) -> &'static str {
        "ExecutionPlugin"
    }

    fn priority(&self) -> i32 {
        95
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
