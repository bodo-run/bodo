use crate::config::{load_script_config, BodoConfig, TaskConfig};
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

    pub fn run_task(&mut self, task_name: &str) -> Result<(), Box<dyn Error>> {
        // Load script config
        let script_config = load_script_config(task_name)?;

        // Set environment variables from script config
        if let Some(env_vars) = script_config.env {
            for (key, value) in env_vars {
                std::env::set_var(key, value);
            }
        }

        // Add execution paths to PATH
        if let Some(exec_paths) = script_config.exec_paths {
            self.env_manager.inject_exec_paths(&exec_paths);
        }

        // Run plugins before task
        self.plugin_manager.run_plugins_for_task(task_name);

        // Run the default task
        self.execute_task(&script_config.default_task)?;

        Ok(())
    }

    fn execute_task(&self, task: &TaskConfig) -> Result<(), Box<dyn Error>> {
        let command = task.command.clone();

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
            )
            .into());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_config() -> BodoConfig {
        BodoConfig {
            env_files: None,
            executable_map: None,
            max_concurrency: None,
            plugins: None,
        }
    }
}
