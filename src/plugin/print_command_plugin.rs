use crate::config::load_script_config;
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
        if let Some((group, task)) = task_name.split_once(':') {
            if let Ok(script_config) = load_script_config(group) {
                if let Some(tasks) = &script_config.tasks {
                    if let Some(task_config) = tasks.get(task) {
                        if task_config.silent.unwrap_or(false) {
                            return;
                        }
                    }
                }

                // Check if this task is part of a concurrent group
                if let Some(concurrent_items) = &script_config.default_task.concurrently {
                    let concurrent_count = concurrent_items.len();
                    use colored::Colorize;
                    println!(
                        "[{}/{}] {} ({}): {}",
                        "PARALLEL".bold().blue(),
                        concurrent_count,
                        task_name.bold(),
                        "concurrent".dimmed(),
                        command.green()
                    );
                    return;
                }
            }
        }

        use colored::Colorize;
        println!("> {}: {}", task_name, command.green());
    }
}
