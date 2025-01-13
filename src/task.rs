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

            for item in concurrent_items {
                match item {
                    ConcurrentItem::Task { task } => {
                        let script_config = load_script_config(task)?;
                        let task_config = script_config.default_task;
                        if let Some(command) = task_config.command {
                            let child = self.spawn_command(&command)?;
                            children.push(child);
                        }
                    }
                    ConcurrentItem::Command { command } => {
                        let child = self.spawn_command(command)?;
                        children.push(child);
                    }
                }
            }

            for mut child in children {
                let status = child.wait()?;
                if !status.success() {
                    let error: Box<dyn Error> = Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Concurrent task failed with status: {}", status),
                    ));
                    self.plugin_manager.on_error(task_name, error.as_ref())?;
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
