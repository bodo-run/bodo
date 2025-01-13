use serde::{Deserialize, Serialize};
use serde_json;
use serde_yaml;
use std::fs;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BodoConfig {
    pub tasks: Option<Vec<TaskConfig>>,
    pub env_files: Option<Vec<String>>,
    pub executable_map: Option<Vec<ExecutableMap>>,
    pub max_concurrency: Option<usize>,
    pub plugins: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TaskConfig {
    pub name: String,
    pub command: String,
    pub cwd: Option<String>,
    pub env: Option<Vec<String>>,
    pub dependencies: Option<Vec<String>>,
    pub plugins: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
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
            max_concurrency: None,
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
        assert!(config.tasks.is_none());
        assert!(config.env_files.is_none());
        assert!(config.executable_map.is_none());
        assert!(config.max_concurrency.is_none());
        assert!(config.plugins.is_none());
    }

    #[test]
    fn test_load_json_config() {
        let content = r#"{
            "tasks": [
                {
                    "name": "test",
                    "command": "echo hello",
                    "cwd": ".",
                    "env": ["TEST=true"],
                    "dependencies": ["prep"],
                    "plugins": ["test-plugin"]
                }
            ],
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
        assert!(config.tasks.is_some());
        assert_eq!(config.tasks.as_ref().unwrap().len(), 1);
        
        let task = &config.tasks.as_ref().unwrap()[0];
        assert_eq!(task.name, "test");
        assert_eq!(task.command, "echo hello");
        assert_eq!(task.cwd, Some(".".to_string()));
        
        cleanup_temp_file(temp_path);
    }

    #[test]
    fn test_load_yaml_config() {
        let content = r#"
tasks:
  - name: test
    command: echo hello
    cwd: .
    env: 
      - TEST=true
    dependencies:
      - prep
    plugins:
      - test-plugin
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
        assert!(config.tasks.is_some());
        assert_eq!(config.tasks.as_ref().unwrap().len(), 1);
        
        let task = &config.tasks.as_ref().unwrap()[0];
        assert_eq!(task.name, "test");
        assert_eq!(task.command, "echo hello");
        assert_eq!(task.cwd, Some(".".to_string()));
        
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