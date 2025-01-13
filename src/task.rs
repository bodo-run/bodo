use crate::config::{BodoConfig, TaskConfig};
use crate::env::EnvManager;
use crate::graph::TaskGraph;
use crate::plugin::PluginManager;
use crate::prompt::PromptManager;
use std::error::Error;
use std::process::Command;

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