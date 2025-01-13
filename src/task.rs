use crate::config::{load_script_config, BodoConfig, TaskConfig};
use crate::env::EnvManager;
use crate::graph::TaskGraph;
use crate::plugin::PluginManager;
use crate::prompt::PromptManager;
use std::error::Error;
use std::process::Command;

#[allow(dead_code)]
pub struct TaskManager {
    config: BodoConfig,
    env_manager: EnvManager,
    task_graph: TaskGraph,
    plugin_manager: PluginManager,
    prompt_manager: PromptManager,
}

impl TaskManager {
    pub fn new(
        config: BodoConfig,
        env_manager: EnvManager,
        task_graph: TaskGraph,
        plugin_manager: PluginManager,
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
        self.plugin_manager.on_before_run(task_name);

        // Run the default task
        let result = self.execute_task(&script_config.default_task, task_name);

        match &result {
            Ok(_) => {
                self.plugin_manager.on_after_run(task_name, 0);
            }
            Err(e) => {
                self.plugin_manager.on_error(task_name, e.as_ref());
                self.plugin_manager.on_after_run(task_name, -1);
            }
        }

        result
    }

    pub fn cleanup(&mut self, exit_code: i32) {
        self.plugin_manager.on_bodo_exit(exit_code);
    }

    fn execute_task(&mut self, task: &TaskConfig, task_name: &str) -> Result<(), Box<dyn Error>> {
        let mut task = task.clone();
        self.plugin_manager.on_resolve_command(&mut task);

        let command = task.command.clone();
        self.plugin_manager.on_command_ready(&command, task_name);

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

    #[test]
    fn test_task_manager_creation() {
        let config = BodoConfig::default();
        let env_manager = EnvManager::new();
        let task_graph = TaskGraph::new();
        let plugin_manager = PluginManager::new(config.clone());
        let prompt_manager = PromptManager::new();

        let task_manager = TaskManager::new(
            config,
            env_manager,
            task_graph,
            plugin_manager,
            prompt_manager,
        );

        assert!(task_manager.config.plugins.is_none());
    }
}
