use crate::config::{BodoConfig, TaskConfig};
use crate::env::EnvManager;
use crate::graph::TaskGraph;
use crate::plugin::PluginManager;
use crate::prompt::PromptManager;
use std::error::Error;
use std::process::Command;

#[allow(dead_code)]
pub struct TaskManager<'a> {
    config: &'a BodoConfig,
    env_manager: EnvManager,
    task_graph: TaskGraph,
    plugin_manager: PluginManager<'a>,
    prompt_manager: PromptManager,
}

impl<'a> TaskManager<'a> {
    pub fn new(
        config: &'a BodoConfig,
        env_manager: EnvManager,
        task_graph: TaskGraph,
        plugin_manager: PluginManager<'a>,
        prompt_manager: PromptManager,
    ) -> Self {
        let mut manager = Self {
            config,
            env_manager,
            task_graph,
            plugin_manager,
            prompt_manager,
        };

        // Initialize task graph with all tasks and dependencies
        if let Some(tasks) = &config.tasks {
            for task in tasks {
                manager.task_graph.add_task(task.name.clone());
                if let Some(deps) = &task.dependencies {
                    for dep in deps {
                        manager.task_graph.add_dependency(task.name.clone(), dep.clone());
                    }
                }
            }
        }

        manager
    }

    pub fn run_task(&self, task_group: &str, subtask: Option<&str>) -> Result<(), Box<dyn Error>> {
        // Get task config
        let task = match self.config.tasks.as_ref() {
            Some(tasks) => tasks
                .iter()
                .find(|t| t.name == task_group)
                .ok_or_else(|| format!("Task {} not found", task_group))?,
            None => return Err("No tasks configured".into()),
        };

        // Get execution order from task graph
        let order = self.task_graph.get_execution_order();
        
        // Run tasks in order
        for task_name in order {
            if let Some(task_config) = self.config.tasks.as_ref()
                .and_then(|tasks| tasks.iter().find(|t| t.name == task_name))
            {
                // Run plugins before task
                self.plugin_manager.run_plugins_for_task(&task_config.name);

                // Run the actual task
                self.execute_task(task_config, subtask)?;
            }
        }

        Ok(())
    }

    fn execute_task(&self, task: &TaskConfig, subtask: Option<&str>) -> Result<(), Box<dyn Error>> {
        let command = if let Some(subtask_name) = subtask {
            format!("{} {}", task.command, subtask_name)
        } else {
            task.command.clone()
        };

        let mut cmd_parts = command.split_whitespace();
        let program = cmd_parts.next().ok_or("Empty command")?;
        let args: Vec<_> = cmd_parts.collect();

        let mut cmd = Command::new(program);
        cmd.args(&args)
            .current_dir(task.cwd.as_deref().unwrap_or("."));

        // Add environment variables
        for (key, value) in self.env_manager.get_env() {
            cmd.env(key, value);
        }

        let status = cmd
            .status()
            .map_err(|e| format!("Failed to execute command: {}", e))?;

        if !status.success() {
            return Err(format!(
                "Task failed with exit code: {}",
                status.code().unwrap_or(-1)
            ).into());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_config() -> BodoConfig {
        BodoConfig {
            tasks: Some(vec![
                TaskConfig {
                    name: "test_task".to_string(),
                    command: "echo hello".to_string(),
                    cwd: Some(".".to_string()),
                    dependencies: None,
                    env: None,
                    plugins: None,
                },
                TaskConfig {
                    name: "task_with_deps".to_string(),
                    command: "echo world".to_string(),
                    cwd: Some(".".to_string()),
                    dependencies: Some(vec!["test_task".to_string()]),
                    env: None,
                    plugins: None,
                },
            ]),
            env_files: None,
            executable_map: None,
            max_concurrency: None,
            plugins: None,
        }
    }
} 