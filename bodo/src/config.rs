use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Deserialize, Serialize)]
pub struct BodoConfig {
    pub tasks: Option<Vec<TaskConfig>>,
    pub env_files: Option<Vec<String>>,
    pub executable_map: Option<Vec<ExecutableMap>>,
    pub max_concurrency: Option<usize>,
    pub plugins: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TaskConfig {
    pub name: String,
    pub command: String,
    pub cwd: Option<String>,
    pub env: Option<Vec<String>>,
    pub dependencies: Option<Vec<String>>,
    pub plugins: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ExecutableMap {
    pub executable: Option<String>,
    pub path: Option<String>,
}

impl Default for BodoConfig {
    fn default() -> Self {
        Self {
            tasks: None,
            env_files: None,
            executable_map: None,
            max_concurrency: Some(4),
            plugins: None,
        }
    }
}

pub fn load_bodo_config() -> BodoConfig {
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
                return config;
            }
        }
    }

    BodoConfig::default()
} 