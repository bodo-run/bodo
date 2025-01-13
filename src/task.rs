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
        Self {
            config,
            env_manager,
            task_graph,
            plugin_manager,
            prompt_manager,
        }
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

        // Run task dependencies if any
        if let Some(deps) = &task.dependencies {
            for dep in deps {
                self.run_task(dep, None)?;
            }
        }

        // Run plugins before task
        self.plugin_manager.run_plugins_for_task(&task.name);

        // Run the actual task
        self.execute_task(task, subtask)?;

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

    fn setup_task_manager(config: &BodoConfig) -> TaskManager {
        let env_manager = EnvManager::new();
        let mut task_graph = TaskGraph::new();
        
        // Add tasks to graph
        if let Some(tasks) = &config.tasks {
            for task in tasks {
                task_graph.add_task(&task.name, task.dependencies.as_ref().unwrap_or(&vec![]));
            }
        }
        
        let plugin_manager = PluginManager::new(&config);
        let prompt_manager = PromptManager::new();

        TaskManager::new(
            &config,
            env_manager,
            task_graph,
            plugin_manager,
            prompt_manager,
        )
    }

    #[test]
    fn test_task_manager_creation() {
        let config = setup_test_config();
        let task_manager = setup_task_manager(&config);

        assert!(task_manager.config.tasks.is_some());
        assert_eq!(task_manager.config.tasks.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_run_simple_task() {
        let config = setup_test_config();
        let task_manager = setup_task_manager(&config);

        let result = task_manager.run_task("test_task", None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_task_with_dependencies() {
        let config = setup_test_config();
        let task_manager = setup_task_manager(&config);

        let result = task_manager.run_task("task_with_deps", None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_nonexistent_task() {
        let config = setup_test_config();
        let task_manager = setup_task_manager(&config);

        let result = task_manager.run_task("nonexistent_task", None);
        assert!(result.is_err());
    }
} 