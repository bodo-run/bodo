use crate::plugin::BodoPlugin;

#[derive(Clone)]
pub struct PrintCommandPlugin;

impl PrintCommandPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PrintCommandPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl BodoPlugin for PrintCommandPlugin {
    fn on_command_ready(&self, command: &str, task_name: &str) {
        // Don't print if the task is marked as silent
        if let Some(task_config) = task_name.split_once(':').and_then(|(group, task)| {
            let script_config = crate::config::load_script_config(group).ok()?;
            script_config.subtasks?.get(task).cloned()
        }) {
            if task_config.silent.unwrap_or(false) {
                return;
            }
        }
        println!("> {}: {}", task_name, command);
    }
}
