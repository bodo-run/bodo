use crate::config::TaskConfig;
use crate::env::EnvManager;
use crate::plugin::PluginManager;
use crate::prompt::PromptManager;
use std::error::Error;
use std::process::{Command, ExitStatus};

pub struct TaskManager {
    config: TaskConfig,
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
        self.plugin_manager.on_before_task_run(task_name)?;

        let command = self.config.command.clone();

        match self.execute_command(&command, task_name) {
            Ok(status) if status.success() => {
                self.plugin_manager.on_after_task_run(task_name, 0)?;
                Ok(())
            }
            Ok(_) => {
                self.plugin_manager.on_after_task_run(task_name, -1)?;
                Err("Task failed".into())
            }
            Err(e) => {
                self.plugin_manager.on_error(task_name, &e.to_string())?;
                Err(e)
            }
        }
    }

    pub fn on_exit(&mut self, exit_code: i32) -> Result<(), Box<dyn Error>> {
        self.plugin_manager.on_bodo_exit(exit_code)
    }

    fn execute_command(
        &mut self,
        command: &str,
        task_name: &str,
    ) -> Result<ExitStatus, Box<dyn Error>> {
        let mut task_config = self.config.clone();
        self.plugin_manager.on_resolve_command(&mut task_config)?;
        self.plugin_manager.on_command_ready(command, task_name)?;

        let mut cmd_parts = command.split_whitespace();
        let program = cmd_parts.next().ok_or("Empty command")?;
        let args: Vec<_> = cmd_parts.collect();

        let mut cmd = Command::new(program);
        cmd.args(&args)
            .current_dir(task_config.cwd.as_deref().unwrap_or("."));

        // Add environment variables
        for (key, value) in self.env_manager.get_env() {
            cmd.env(key, value);
        }

        cmd.status()
            .map_err(|e| format!("Failed to execute command: {}", e).into())
    }

    pub fn confirm_task_execution(&mut self, task_name: &str) -> Result<bool, Box<dyn Error>> {
        Ok(self.prompt_manager.confirm(&format!("Run task '{}'?", task_name)))
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
