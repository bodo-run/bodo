use crate::config::load_script_config;
use crate::config::{ConcurrentItem, TaskConfig};
use crate::env::EnvManager;
use crate::plugin::PluginManager;
use crate::prompt::PromptManager;
use std::error::Error;
use std::process::{Child, Command, ExitStatus};

pub struct TaskManager {
    pub config: TaskConfig,
    env_manager: EnvManager,
    plugin_manager: PluginManager,
    prompt_manager: PromptManager,
}

impl TaskManager {
    pub fn new(
        config: TaskConfig,
        env_manager: EnvManager,
        plugin_manager: PluginManager,
        prompt_manager: PromptManager,
    ) -> Self {
        Self {
            config,
            env_manager,
            plugin_manager,
            prompt_manager,
        }
    }

    pub fn run_task(&mut self, task_name: &str) -> Result<(), Box<dyn Error>> {
        if let Some(command) = &self.config.command {
            let mut cmd = Command::new("sh");
            cmd.arg("-c").arg(command);

            if let Some(env_vars) = &self.config.env {
                for (key, value) in env_vars {
                    cmd.env(key, value);
                }
            }

            let status = cmd.status()?;
            if !status.success() {
                return Err(format!("Task '{}' failed", task_name).into());
            }
        }

        Ok(())
    }

    pub fn run_concurrently(&mut self, task_name: &str) -> Result<(), Box<dyn Error>> {
        if let Some(concurrent_items) = &self.config.concurrently {
            let mut handles = vec![];

            for item in concurrent_items {
                match item {
                    ConcurrentItem::Task { task } => {
                        let mut handle = self.spawn_task(task)?;
                        handles.push(handle);
                    }
                    ConcurrentItem::Command { command } => {
                        let mut handle = self.spawn_command(command)?;
                        handles.push(handle);
                    }
                }
            }

            // Wait for all tasks/commands to finish
            for mut handle in handles {
                let status = handle.wait()?;
                if !status.success() {
                    return Err(format!("A concurrent task/command failed").into());
                }
            }
        }

        Ok(())
    }

    fn spawn_task(&self, task_name: &str) -> Result<Child, Box<dyn Error>> {
        let script_config = load_script_config(task_name)?;
        let task_config = TaskConfig {
            command: Some(script_config.default_task.command.unwrap_or_default()),
            cwd: script_config.default_task.cwd,
            env: script_config.default_task.env,
            dependencies: script_config.default_task.dependencies,
            plugins: script_config.default_task.plugins,
            concurrently: None,
        };

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

    #[test]
    fn test_task_manager_creation() {
        let config = TaskConfig {
            command: String::from("echo test"),
            cwd: None,
            env: None,
            dependencies: Some(Vec::new()),
            plugins: None,
        };
        let env_manager = EnvManager::new();
        let plugin_manager = PluginManager::new();
        let prompt_manager = PromptManager::new();

        let _task_manager = TaskManager::new(config, env_manager, plugin_manager, prompt_manager);
    }
}
