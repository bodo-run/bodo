use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskConfig {
    pub description: Option<String>,
    pub command: Option<String>,
    pub cwd: Option<String>,
    #[serde(default)]
    pub pre_deps: Vec<Dependency>,
    #[serde(default)]
    pub post_deps: Vec<Dependency>,
    pub watch: Option<WatchConfig>,
    pub timeout: Option<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl BodoConfig {
    pub async fn load(_config_path: Option<String>) -> Result<Self> {
        Ok(BodoConfig::default())
    }
}
