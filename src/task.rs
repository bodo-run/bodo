use crate::config::load_script_config;
use crate::config::{ConcurrentItem, TaskConfig};
use crate::env::EnvManager;
use crate::plugin::PluginManager;
use crate::prompt::PromptManager;
use std::error::Error;
use std::process::{Child, Command};

pub struct TaskManager {
    pub config: TaskConfig,
    pub(crate) plugin_manager: PluginManager,
}

impl TaskManager {
    pub fn new(
        config: TaskConfig,
        _env_manager: EnvManager,
        plugin_manager: PluginManager,
        _prompt_manager: PromptManager,
    ) -> Self {
        Self {
            config,
            plugin_manager,
        }
    }

    pub fn run_task(&mut self, task_name: &str) -> Result<(), Box<dyn Error>> {
        self.plugin_manager.on_before_task_run(task_name)?;

        if let Some(command) = &self.config.command {
            let mut cmd = Command::new("sh");
            cmd.arg("-c").arg(command);

            if let Some(env_vars) = &self.config.env {
                for (key, value) in env_vars {
                    cmd.env(key, value);
                }
            }

            self.plugin_manager.on_command_ready(command, task_name)?;

            let status = cmd.status()?;
            if !status.success() {
                let error = format!("Task '{}' failed", task_name);
                self.plugin_manager.on_error(task_name, &error)?;
                return Err(error.into());
            }
        }

        self.plugin_manager.on_after_task_run(task_name, 0)?;
        Ok(())
    }

    pub fn run_concurrently(&mut self, _task_name: &str) -> Result<(), Box<dyn Error>> {
        if let Some(concurrent_items) = &self.config.concurrently {
            let mut handles = vec![];

            for item in concurrent_items {
                match item {
                    ConcurrentItem::Task { task } => {
                        let parts: Vec<&str> = task.split(':').collect();
                        let (task_name, subtask) = match parts.as_slice() {
                            [task, subtask] => (*task, Some(*subtask)),
                            [task] => (*task, None),
                            _ => {
                                return Err(Box::<dyn Error>::from(format!(
                                    "Invalid task format: {}",
                                    task
                                )))
                            }
                        };

                        let script_config = load_script_config(task_name)?;
                        let task_config = if let Some(subtask_name) = subtask {
                            if let Some(subtasks) = &script_config.subtasks {
                                subtasks
                                    .get(subtask_name)
                                    .ok_or_else(|| {
                                        Box::<dyn Error>::from(format!(
                                            "Subtask '{}' not found",
                                            subtask_name
                                        ))
                                    })?
                                    .clone()
                            } else {
                                return Err(Box::<dyn Error>::from(format!(
                                    "No subtasks defined in {}",
                                    task_name
                                )));
                            }
                        } else {
                            script_config.default_task
                        };

                        let handle = self.spawn_task_with_config(&task_config)?;
                        handles.push(handle);
                    }
                    ConcurrentItem::Command { command } => {
                        let handle = self.spawn_command(command)?;
                        handles.push(handle);
                    }
                }
            }

            // Wait for all tasks/commands to finish
            for mut handle in handles {
                let status = handle.wait()?;
                if !status.success() {
                    return Err(Box::<dyn Error>::from("A concurrent task/command failed"));
                }
            }
        }

        Ok(())
    }

    fn spawn_task_with_config(&self, task_config: &TaskConfig) -> Result<Child, Box<dyn Error>> {
        // Build command
        let mut cmd = Command::new("sh");
        if let Some(command) = &task_config.command {
            cmd.arg("-c").arg(command);
        }

        // Set environment
        if let Some(env_vars) = &task_config.env {
            for (key, value) in env_vars {
                cmd.env(key, value);
            }
        }

        Ok(cmd.spawn()?)
    }

    fn spawn_command(&self, command: &str) -> Result<Child, Box<dyn Error>> {
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(command);
        Ok(cmd.spawn()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::EnvManager;
    use crate::prompt::PromptManager;

    #[test]
    fn test_task_manager_creation() {
        let config = TaskConfig {
            command: Some(String::from("echo test")),
            cwd: None,
            env: None,
            dependencies: Some(Vec::new()),
            plugins: None,
            concurrently: None,
            description: None,
        };
        let env_manager = EnvManager::new();
        let plugin_manager = PluginManager::new();
        let prompt_manager = PromptManager::new();

        let _task_manager = TaskManager::new(config, env_manager, plugin_manager, prompt_manager);
    }
}
