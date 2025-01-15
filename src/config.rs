use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_json;
use serde_yaml;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fs;
use tracing::debug;

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct BodoConfig {
    pub env_files: Option<Vec<String>>,
    pub executable_map: Option<Vec<ExecutableMap>>,
    pub max_concurrency: Option<usize>,
    pub plugins: Option<Vec<String>>,
    pub disable_color: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum ConcurrentItem {
    Task {
        task: String,
        output: Option<OutputConfig>,
    },
    Command {
        command: String,
        name: Option<String>,
        output: Option<OutputConfig>,
    },
}

impl fmt::Display for ConcurrentItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConcurrentItem::Task { task, .. } => write!(f, "{}", task),
            ConcurrentItem::Command { name, .. } => {
                if let Some(name) = name {
                    write!(f, "{}", name)
                } else {
                    write!(f, "command")
                }
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum ColorSpec {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OutputConfig {
    pub prefix: Option<String>,
    pub color: Option<ColorSpec>,
    pub disable_color: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ConcurrentlyOptions {
    pub fail_fast: bool,
    #[serde(default)]
    pub timeout: Option<u64>, // in seconds
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TaskConfig {
    pub command: Option<String>,
    pub cwd: Option<String>,
    pub env: Option<HashMap<String, String>>,
    #[serde(rename = "pre_deps")]
    pub dependencies: Option<Vec<ConcurrentItem>>,
    pub plugins: Option<Vec<String>>,
    pub concurrently: Option<Vec<ConcurrentItem>>,
    pub concurrently_options: Option<ConcurrentlyOptions>,
    pub description: Option<String>,
    pub silent: Option<bool>,
    pub output: Option<OutputConfig>,
    pub disable_color: Option<bool>,
    pub tasks: Option<HashMap<String, TaskConfig>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ExecutableMap {
    pub executable: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ScriptConfig {
    pub name: Option<String>,
    pub description: Option<String>,
    pub exec_paths: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
    pub default_task: TaskConfig,
    pub tasks: Option<HashMap<String, TaskConfig>>,
    pub disable_color: Option<bool>,
}

pub fn load_bodo_config() -> Result<BodoConfig, Box<dyn Error>> {
    let config_paths = [
        "bodo.json",
        "bodo.yaml",
        "bodo.yml",
        ".bodo/config.json",
        ".bodo/config.yaml",
        ".bodo/config.yml",
    ];

    for path in config_paths.iter() {
        if let Ok(contents) = fs::read_to_string(path) {
            let config = if path.ends_with(".json") {
                serde_json::from_str(&contents).ok()
            } else {
                serde_yaml::from_str(&contents).ok()
            };

            if let Some(config) = config {
                return Ok(config);
            }
        }
    }

    Ok(BodoConfig::default())
}

pub fn load_script_config(task_name: &str) -> Result<ScriptConfig, Box<dyn Error>> {
    // Start from the current directory
    let mut current_dir = std::env::current_dir()?;
    debug!(
        "Starting search from current directory: {}",
        current_dir.display()
    );

    loop {
        // First try to find script.yaml in the scripts directory
        let script_path = current_dir.join("scripts").join("script.yaml");
        debug!("Checking path: {}", script_path.display());
        if script_path.exists() {
            debug!("Found script at: {}", script_path.display());
            let contents = fs::read_to_string(&script_path)?;
            let config: ScriptConfig = serde_yaml::from_str(&contents)?;
            return Ok(config);
        }

        // Then try to find script.yaml in the task-specific directory
        let task_script_path = current_dir
            .join("scripts")
            .join(task_name)
            .join("script.yaml");
        debug!("Checking path: {}", task_script_path.display());
        if task_script_path.exists() {
            debug!("Found script at: {}", task_script_path.display());
            let contents = fs::read_to_string(&task_script_path)?;
            let config: ScriptConfig = serde_yaml::from_str(&contents)?;
            return Ok(config);
        }

        // Try parent directory
        if !current_dir.pop() {
            debug!("Reached root directory, no more parents");
            break;
        }
        debug!("Moving to parent directory: {}", current_dir.display());
    }

    // If we get here, we didn't find the script
    let current_dir = std::env::current_dir()?;
    let expected_path = current_dir
        .join("scripts")
        .join(task_name)
        .join("script.yaml");
    Err(format!(
        "Task {} not found. Create a script.yaml file at:{}  - {}{}or run bodo from a directory containing a scripts/{} directory",
        task_name.red(),
        "\n",
        expected_path.display(),
        "\n",
        task_name
    ).into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;

    fn create_temp_config_file(content: &str, extension: &str) -> PathBuf {
        let mut temp_path = std::env::temp_dir();
        temp_path.push(format!("test_config.{}", extension));

        let mut file = File::create(&temp_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();

        temp_path
    }

    fn cleanup_temp_file(path: PathBuf) {
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_default_config() {
        let config = BodoConfig::default();
        assert!(config.env_files.is_none());
        assert!(config.executable_map.is_none());
        assert!(config.max_concurrency.is_none());
        assert!(config.plugins.is_none());
    }

    #[test]
    fn test_load_json_config() {
        let content = r#"{
            "env_files": [".env"],
            "executable_map": [
                {
                    "executable": "node",
                    "path": "/usr/local/bin/node"
                }
            ],
            "max_concurrency": 4,
            "plugins": ["plugin1"]
        }"#;

        let temp_path = create_temp_config_file(content, "json");
        std::env::set_current_dir(temp_path.parent().unwrap()).unwrap();

        let config: BodoConfig = serde_json::from_str(content).unwrap();
        assert!(config.env_files.is_some());
        assert!(config.executable_map.is_some());
        assert_eq!(config.max_concurrency, Some(4));

        cleanup_temp_file(temp_path);
    }

    #[test]
    fn test_load_yaml_config() {
        let content = r#"
env_files:
  - .env
executable_map:
  - executable: node
    path: /usr/local/bin/node
max_concurrency: 4
plugins:
  - plugin1
"#;

        let temp_path = create_temp_config_file(content, "yaml");
        std::env::set_current_dir(temp_path.parent().unwrap()).unwrap();

        let config: BodoConfig = serde_yaml::from_str(content).unwrap();
        assert!(config.env_files.is_some());
        assert!(config.executable_map.is_some());
        assert_eq!(config.max_concurrency, Some(4));

        cleanup_temp_file(temp_path);
    }

    #[test]
    fn test_executable_map() {
        let map = ExecutableMap {
            executable: Some("node".to_string()),
            path: Some("/usr/local/bin/node".to_string()),
        };

        assert_eq!(map.executable.as_ref().unwrap(), "node");
        assert_eq!(map.path.as_ref().unwrap(), "/usr/local/bin/node");
    }
}
