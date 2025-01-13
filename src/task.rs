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

            let status = cmd.status().map_err(|e| {
                let error: Box<dyn Error> = Box::new(e);
                self.plugin_manager
                    .on_error(task_name, error.as_ref())
                    .unwrap();
                error
            })?;

            self.plugin_manager
                .on_after_task_run(task_name, status.code().unwrap_or(1))?;

            if !status.success() {
                let error: Box<dyn Error> = Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Command failed with status: {}", status),
                ));
                self.plugin_manager.on_error(task_name, error.as_ref())?;
                return Err(error);
            }
        }

        Ok(())
    }

    pub fn run_concurrently(&mut self, task_name: &str) -> Result<(), Box<dyn Error>> {
        if let Some(concurrent_items) = &self.config.concurrently {
            let mut children = Vec::new();

            let current_script_config = load_script_config(task_name)?;

            for item in concurrent_items {
                match item {
                    ConcurrentItem::Task { task } => {
                        let task_config = if task.contains(':') {
                            let parts: Vec<&str> = task.split(':').collect();
                            if parts.len() != 2 {
                                return Err(
                                    format!("Invalid task reference format: {}", task).into()
                                );
                            }
                            let script_config = load_script_config(parts[0])?;
                            if let Some(tasks) = &script_config.tasks {
                                tasks.get(parts[1]).cloned().ok_or_else(|| {
                                    format!(
                                        "Task '{}' not found in script '{}'",
                                        parts[1], parts[0]
                                    )
                                })?
                            } else {
                                return Err(
                                    format!("No tasks defined in script '{}'", parts[0]).into()
                                );
                            }
                        } else if let Some(tasks) = &current_script_config.tasks {
                            if let Some(task_config) = tasks.get(task) {
                                task_config.clone()
                            } else {
                                let script_config = load_script_config(task)?;
                                script_config.default_task
                            }
                        } else {
                            let script_config = load_script_config(task)?;
                            script_config.default_task
                        };

                        if let Some(command) = task_config.command {
                            let subtask_name = format!("{}:{}", task_name, task);
                            self.plugin_manager
                                .on_command_ready(&command, &subtask_name)?;
                            let child = self.spawn_command(&command)?;
                            children.push((child, subtask_name));
                        }
                    }
                    ConcurrentItem::Command { command } => {
                        let command_name = format!("{}:command", task_name);
                        self.plugin_manager
                            .on_command_ready(command, &command_name)?;
                        let child = self.spawn_command(command)?;
                        children.push((child, command_name));
                    }
                }
            }

            for (mut child, subtask_name) in children {
                let status = child.wait()?;
                if !status.success() {
                    let error: Box<dyn Error> = Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Concurrent task failed with status: {}", status),
                    ));
                    self.plugin_manager
                        .on_error(&subtask_name, error.as_ref())?;
                    return Err(error);
                }
            }
        }

        Ok(())
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
            silent: None,
        };
        let env_manager = EnvManager::new();
        let plugin_manager = PluginManager::new();
        let prompt_manager = PromptManager::new();

        let _task_manager = TaskManager::new(config, env_manager, plugin_manager, prompt_manager);
    }
}
