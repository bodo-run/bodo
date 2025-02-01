use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Dependency {
    Task { task: String },
    Command { command: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BodoConfig {
    pub root_script: Option<String>,
    pub scripts_dirs: Option<Vec<String>>,
    pub tasks: HashMap<String, TaskConfig>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub exec_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WatchConfig {
    pub patterns: Vec<String>,
    #[serde(default = "default_debounce_ms")]
    pub debounce_ms: u64,
    #[serde(default)]
    pub ignore_patterns: Vec<String>,
}
fn default_debounce_ms() -> u64 {
    500
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ConcurrentlyOptions {
    pub fail_fast: Option<bool>,
    pub max_concurrent_tasks: Option<usize>,
    pub prefix_output: Option<bool>,
    pub prefix_color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct TaskConfig {
    pub description: Option<String>,
    pub command: Option<String>,
    pub cwd: Option<String>,
    #[serde(default)]
    pub pre_deps: Vec<Dependency>,
    #[serde(default)]
    pub post_deps: Vec<Dependency>,
    #[serde(default)]
    pub concurrently_options: ConcurrentlyOptions,
    #[serde(default)]
    pub concurrently: Vec<Dependency>,
    pub watch: Option<WatchConfig>,
    pub timeout: Option<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub exec_paths: Vec<String>,
}

impl BodoConfig {
    pub fn load(_config_path: Option<String>) -> Result<Self> {
        // You could load from a file, etc.
        Ok(BodoConfig::default())
    }
}
