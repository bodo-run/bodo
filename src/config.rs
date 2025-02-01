use crate::errors::{BodoError, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::{Validate, ValidationError};

/// Helper function to check for reserved task names
fn validate_task_name(name: &str) -> std::result::Result<(), ValidationError> {
    let reserved = [
        "watch",
        "default_task",
        "pre_deps",
        "post_deps",
        "concurrently",
    ];
    if reserved.contains(&name) {
        let mut err = ValidationError::new("reserved_name");
        err.message = Some(format!("'{}' is a reserved task name", name).into());
        return Err(err);
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum Dependency {
    Task { task: String },
    Command { command: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, Validate, JsonSchema)]
pub struct BodoConfig {
    /// Path to the root script (relative to the root of the project)
    pub root_script: Option<String>,

    /// Path to the scripts directory (relative to the root of the project)
    #[validate(length(min = 1))]
    pub scripts_dirs: Option<Vec<String>>,
    #[validate]
    pub tasks: HashMap<String, TaskConfig>,

    /// Environment variables to set for all tasks
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Paths to add to the PATH environment variable for all tasks
    #[serde(default)]
    pub exec_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, JsonSchema, PartialEq)]
pub struct WatchConfig {
    /// Glob patterns to watch for file changes
    #[validate(length(min = 1))]
    pub patterns: Vec<String>,

    /// Debounce time in milliseconds
    #[validate(range(min = 1, max = 60_000))]
    #[serde(default = "default_debounce_ms")]
    pub debounce_ms: u64,

    /// Glob patterns to ignore
    #[serde(default)]
    pub ignore_patterns: Vec<String>,

    /// Automatically enable watch mode. Enabling this will automatically enable watch mode for all tasks
    /// that have the watch option set. Providing --watch is not required.
    #[serde(default)]
    pub auto_watch: bool,
}

fn default_debounce_ms() -> u64 {
    500
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Validate, JsonSchema)]
pub struct ConcurrentlyOptions {
    /// Fail fast if any task fails
    #[serde(default)]
    pub fail_fast: Option<bool>,

    /// Maximum number of concurrent tasks
    #[validate(range(min = 1, max = 1000))]
    pub max_concurrent_tasks: Option<usize>,

    /// Prefix color for the concurrently task
    #[serde(default)]
    pub prefix_color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Validate, JsonSchema)]
pub struct TaskConfig {
    /// Description of the task
    #[validate(length(min = 1, max = 128))]
    pub description: Option<String>,

    /// Command to run
    pub command: Option<String>,

    /// Working directory for the task
    pub cwd: Option<String>,

    /// Pre-dependencies for the task
    #[serde(default)]
    pub pre_deps: Vec<Dependency>,

    /// Post-dependencies for the task
    #[serde(default)]
    pub post_deps: Vec<Dependency>,

    /// Concurrently options for the task
    #[serde(default)]
    #[validate]
    pub concurrently_options: ConcurrentlyOptions,

    /// Concurrently tasks to run
    #[serde(default)]
    pub concurrently: Vec<Dependency>,

    /// Watch options for the task
    #[validate]
    pub watch: Option<WatchConfig>,

    /// Timeout for the task
    pub timeout: Option<String>,

    /// Environment variables to set for the task
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Paths to add to the PATH environment variable for the task
    #[serde(default)]
    pub exec_paths: Vec<String>,

    #[serde(skip)]
    #[validate(custom = "validate_task_name")]
    pub _name_check: Option<String>,
}

impl BodoConfig {
    pub fn load(config_path: Option<String>) -> Result<Self> {
        let config = if let Some(path) = config_path {
            let contents = std::fs::read_to_string(path)?;
            let config: BodoConfig = serde_yaml::from_str(&contents)?;
            config.validate().map_err(BodoError::from)?;
            config
        } else {
            BodoConfig::default()
        };
        Ok(config)
    }

    /// Generate JSON schema for the config
    pub fn generate_schema() -> String {
        let schema = schemars::schema_for!(BodoConfig);
        serde_json::to_string_pretty(&schema).unwrap_or_default()
    }
}
