pub mod print_command_plugin;
mod types;

pub use print_command_plugin::PrintCommandPlugin;
pub use types::BodoPlugin;

use crate::config::TaskConfig;
use serde_json::json;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Clone)]
pub enum Plugin {
    Native(PrintCommandPlugin),
    External(PathBuf),
}

#[derive(Clone)]
pub struct PluginManager {
    plugins: Vec<Plugin>,
    verbose: bool,
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginManager {
    pub fn new() -> Self {
        let mut manager = Self {
            plugins: vec![],
            verbose: false,
        };
        manager
            .plugins
            .push(Plugin::Native(PrintCommandPlugin::new()));
        manager
    }

    pub fn set_verbose(&mut self, verbose: bool) {
        self.verbose = verbose;
    }

    pub fn register_plugin(&mut self, plugin_path: PathBuf) {
        self.plugins.push(Plugin::External(plugin_path));
    }

    fn get_bridge_script_path(&self, extension: &str) -> Result<PathBuf> {
        let bridge_name = match extension {
            "js" | "ts" => "bodo-plugin-bridge.js",
            "py" => "bodo-plugin-bridge.py",
            "rb" => "bodo-plugin-bridge.rb",
            "sh" => "bodo-plugin-bridge.sh",
            _ => {
                return Err(Box::new(PluginError {
                    message: format!("Unsupported plugin type: {}", extension),
                }))
            }
        };

        let bridge_path = if cfg!(test) {
            // In test mode, look for bridges in the project root
            std::env::current_dir()?
                .join("src")
                .join("plugin")
                .join("bridges")
                .join(bridge_name)
        } else {
            // In normal mode, look for bridges in the installation directory
            std::env::current_exe()?
                .parent()
                .ok_or_else(|| PluginError {
                    message: "Could not determine executable directory".to_string(),
                })?
                .join("bridges")
                .join(bridge_name)
        };

        if !bridge_path.exists() {
            // If not found in the default location, try looking in the project root
            let project_root_path = std::env::current_dir()?
                .join("src")
                .join("plugin")
                .join("bridges")
                .join(bridge_name);
            if project_root_path.exists() {
                return Ok(project_root_path);
            }

            return Err(Box::new(PluginError {
                message: format!("Bridge script not found: {}", bridge_path.display()),
            }));
        }

        Ok(bridge_path)
    }

    fn run_external_plugin(
        &self,
        plugin_path: &Path,
        _hook_name: &str,
        data: serde_json::Value,
    ) -> Result<()> {
        let plugin_abs_path = if plugin_path.is_absolute() {
            plugin_path.to_path_buf()
        } else {
            std::env::current_dir()?.join(plugin_path)
        };

        let extension = plugin_path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| PluginError {
                message: format!("Plugin {} has no extension", plugin_path.display()),
            })?;

        let bridge_path = self.get_bridge_script_path(extension)?;

        let interpreter = match extension {
            "js" | "ts" => "node",
            "py" => "python3",
            "rb" => "ruby",
            "sh" => "bash",
            _ => unreachable!(),
        };

        let mut cmd = Command::new(interpreter);
        cmd.arg(&bridge_path)
            .env("BODO_PLUGIN_FILE", &plugin_abs_path)
            .env("BODO_OPTS", data.to_string())
            .env("BODO_VERBOSE", self.verbose.to_string())
            .current_dir(
                plugin_abs_path
                    .parent()
                    .unwrap_or(&std::env::current_dir()?),
            );

        let output = cmd.output().map_err(|e| PluginError {
            message: format!("Failed to execute {} plugin: {}", plugin_path.display(), e),
        })?;

        if !output.status.success() {
            return Err(Box::new(PluginError {
                message: format!(
                    "Plugin {} failed with code {:?}",
                    plugin_path.display(),
                    output.status.code()
                ),
            }));
        }

        if !output.stdout.is_empty() {
            print!("{}", String::from_utf8_lossy(&output.stdout));
        }
        if !output.stderr.is_empty() && self.verbose {
            eprint!("{}", String::from_utf8_lossy(&output.stderr));
        }

        Ok(())
    }

    pub fn on_before_task_run(&self, task_name: &str) -> Result<()> {
        for plugin in &self.plugins {
            match plugin {
                Plugin::Native(p) => p.on_before_task_run(task_name),
                Plugin::External(path) => {
                    let data = json!({
                        "hook": "onBeforeTaskRun",
                        "taskName": task_name,
                        "cwd": std::env::current_dir()?.display().to_string(),
                        "timestamp": std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    });
                    self.run_external_plugin(path, "onBeforeTaskRun", data)?;
                }
            }
        }
        Ok(())
    }

    pub fn on_after_task_run(&self, task_name: &str, status_code: i32) -> Result<()> {
        for plugin in &self.plugins {
            match plugin {
                Plugin::Native(p) => p.on_after_task_run(task_name, status_code),
                Plugin::External(path) => {
                    let data = json!({
                        "hook": "onAfterTaskRun",
                        "taskName": task_name,
                        "status": status_code,
                        "timestamp": std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    });
                    self.run_external_plugin(path, "onAfterTaskRun", data)?;
                }
            }
        }
        Ok(())
    }

    pub fn on_error(&self, task_name: &str, err: &dyn Error) -> Result<()> {
        for plugin in &self.plugins {
            match plugin {
                Plugin::Native(p) => p.on_error(task_name, err),
                Plugin::External(path) => {
                    let data = json!({
                        "hook": "onError",
                        "taskName": task_name,
                        "error": err.to_string(),
                        "timestamp": std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    });
                    self.run_external_plugin(path, "onError", data)?;
                }
            }
        }
        Ok(())
    }

    pub fn on_resolve_command(&self, task: &TaskConfig) -> Result<()> {
        for plugin in &self.plugins {
            match plugin {
                Plugin::Native(p) => p.on_resolve_command(task),
                Plugin::External(path) => {
                    let data = json!({
                        "hook": "onResolveCommand",
                        "task": task,
                        "timestamp": std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    });
                    self.run_external_plugin(path, "onResolveCommand", data)?;
                }
            }
        }
        Ok(())
    }

    pub fn on_command_ready(&self, command: &str, task_name: &str) -> Result<()> {
        for plugin in &self.plugins {
            match plugin {
                Plugin::Native(p) => p.on_command_ready(command, task_name),
                Plugin::External(path) => {
                    let data = json!({
                        "hook": "onCommandReady",
                        "command": command,
                        "taskName": task_name,
                        "timestamp": std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    });
                    self.run_external_plugin(path, "onCommandReady", data)?;
                }
            }
        }
        Ok(())
    }

    pub fn on_bodo_exit(&self, exit_code: i32) -> Result<()> {
        for plugin in &self.plugins {
            match plugin {
                Plugin::Native(p) => p.on_bodo_exit(exit_code),
                Plugin::External(path) => {
                    let data = json!({
                        "hook": "onBodoExit",
                        "exitCode": exit_code,
                        "timestamp": std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    });
                    self.run_external_plugin(path, "onBodoExit", data)?;
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
struct PluginError {
    message: String,
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for PluginError {}
