mod types;

pub use types::BodoPlugin;

use crate::config::TaskConfig;
use serde_json::json;
use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug)]
pub struct PluginError {
    message: String,
}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for PluginError {}

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug, Default)]
pub struct PluginManager {
    plugins: Vec<PathBuf>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_plugin(&mut self, plugin_path: PathBuf) {
        self.plugins.push(plugin_path);
    }

    fn get_bridge_script(plugin_path: &Path) -> Option<(&'static str, &'static str)> {
        match plugin_path.extension()?.to_str()? {
            "js" | "ts" => Some(("node", "bodo-plugin-bridge.js")),
            "py" => Some(("python3", "bodo-plugin-bridge.py")),
            "rb" => Some(("ruby", "bodo-plugin-bridge.rb")),
            _ => None,
        }
    }

    fn run_plugin_hook(&self, _hook_name: &str, data: serde_json::Value) -> Result<()> {
        for plugin_path in &self.plugins {
            let (interpreter, bridge) =
                Self::get_bridge_script(plugin_path).ok_or_else(|| PluginError {
                    message: "Unsupported plugin file extension".to_string(),
                })?;

            let bridge_path = PathBuf::from("src/plugin/bridges").join(bridge);

            let status = Command::new(interpreter)
                .arg(bridge_path)
                .env("BODO_PLUGIN_FILE", plugin_path)
                .env("BODO_OPTS", data.to_string())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()
                .map_err(|e| PluginError {
                    message: format!("Failed to execute {} plugin: {}", plugin_path.display(), e),
                })?;

            if !status.success() {
                return Err(Box::new(PluginError {
                    message: format!(
                        "Plugin {} failed with code {:?}",
                        plugin_path.display(),
                        status.code()
                    ),
                }));
            }
        }
        Ok(())
    }

    pub fn on_before_task_run(&self, task_name: &str) -> Result<()> {
        let data = json!({
            "hook": "onBeforeTaskRun",
            "taskName": task_name,
            "cwd": std::env::current_dir()?.to_string_lossy(),
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        });
        self.run_plugin_hook("onBeforeTaskRun", data)
    }

    pub fn on_after_task_run(&self, task_name: &str, status: i32) -> Result<()> {
        let data = json!({
            "hook": "onAfterTaskRun",
            "taskName": task_name,
            "status": status,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        });
        self.run_plugin_hook("onAfterTaskRun", data)
    }

    pub fn on_error(&self, task_name: &str, error: &str) -> Result<()> {
        let data = json!({
            "hook": "onError",
            "taskName": task_name,
            "error": error,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        });
        self.run_plugin_hook("onError", data)
    }

    pub fn on_bodo_exit(&self, exit_code: i32) -> Result<()> {
        let data = json!({
            "hook": "onBodoExit",
            "exitCode": exit_code,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        });
        self.run_plugin_hook("onBodoExit", data)
    }

    pub fn on_resolve_command(&self, task: &mut TaskConfig) -> Result<()> {
        let data = json!({
            "hook": "onResolveCommand",
            "task": task,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        });
        self.run_plugin_hook("onResolveCommand", data)
    }

    pub fn on_command_ready(&self, command: &str, task_name: &str) -> Result<()> {
        let data = json!({
            "hook": "onCommandReady",
            "command": command,
            "taskName": task_name,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        });
        self.run_plugin_hook("onCommandReady", data)
    }
}
