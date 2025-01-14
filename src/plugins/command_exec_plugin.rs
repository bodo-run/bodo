use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::process::Command;
use tracing::{info, warn};

use crate::{
    errors::PluginError,
    graph::{Graph, NodeKind},
    plugin::{Plugin, PluginConfig},
};

/// A plugin that executes commands and tasks using Tokio.
pub struct CommandExecPlugin {
    pub shell: String,
    pub shell_args: Vec<String>,
    pub current_env: HashMap<String, String>,
    pub working_dir: PathBuf,
}

impl CommandExecPlugin {
    pub fn new() -> Self {
        CommandExecPlugin {
            shell: std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string()),
            shell_args: vec!["-c".to_string()],
            current_env: std::env::vars().collect(),
            working_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        }
    }

    async fn execute_command(
        &self,
        cmd: &str,
        env: &HashMap<String, String>,
        working_dir: Option<&str>,
    ) -> Result<(), PluginError> {
        let mut command = Command::new(&self.shell);
        command.args(&self.shell_args);
        command.arg(cmd);

        // Set up environment
        let mut final_env = self.current_env.clone();
        final_env.extend(env.clone());
        command.envs(final_env);

        // Set working directory if specified
        if let Some(dir) = working_dir {
            command.current_dir(dir);
        } else {
            command.current_dir(&self.working_dir);
        }

        info!("Executing command: {}", cmd);
        let output = command
            .output()
            .await
            .map_err(|e| PluginError::Execution(format!("Failed to execute command: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Command failed: {}", stderr);
            return Err(PluginError::Execution(format!(
                "Command failed with status: {}",
                output.status
            )));
        }

        Ok(())
    }
}

#[async_trait]
impl Plugin for CommandExecPlugin {
    fn name(&self) -> &'static str {
        "CommandExecPlugin"
    }

    async fn on_init(&mut self, config: &PluginConfig) -> Result<(), PluginError> {
        if let Some(options) = &config.options {
            if let Some(shell) = options.get("shell") {
                if let Some(shell) = shell.as_str() {
                    self.shell = shell.to_string();
                }
            }
            if let Some(shell_args) = options.get("shell_args") {
                if let Some(args) = shell_args.as_array() {
                    self.shell_args = args
                        .iter()
                        .filter_map(|a| a.as_str().map(String::from))
                        .collect();
                }
            }
            if let Some(working_dir) = options.get("working_dir") {
                if let Some(dir) = working_dir.as_str() {
                    self.working_dir = PathBuf::from(dir);
                }
            }
        }
        Ok(())
    }

    async fn on_before_execute(&mut self, graph: &mut Graph) -> Result<(), PluginError> {
        for node in &mut graph.nodes {
            match &node.kind {
                NodeKind::Task(task) => {
                    let env: HashMap<String, String> = node
                        .metadata
                        .iter()
                        .filter_map(|(k, v)| {
                            if let Some(env_key) = k.strip_prefix("env.") {
                                Some((env_key.to_string(), v.clone()))
                            } else {
                                None
                            }
                        })
                        .collect();

                    // Execute task dependencies first
                    for dep in &task.dependencies {
                        if let Some(dep_node) = graph.get_node_by_name(dep) {
                            if let NodeKind::Command(cmd) = &dep_node.kind {
                                self.execute_command(
                                    &cmd.raw_command,
                                    &env,
                                    task.working_dir.as_deref(),
                                )
                                .await?;
                            }
                        }
                    }
                }
                NodeKind::Command(cmd) => {
                    let env: HashMap<String, String> = node
                        .metadata
                        .iter()
                        .filter_map(|(k, v)| {
                            if let Some(env_key) = k.strip_prefix("env.") {
                                Some((env_key.to_string(), v.clone()))
                            } else {
                                None
                            }
                        })
                        .collect();

                    self.execute_command(&cmd.raw_command, &env, cmd.working_dir.as_deref())
                        .await?;
                }
            }
        }
        Ok(())
    }
}
