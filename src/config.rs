use crate::errors::{BodoError, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::{Validate, ValidationError};

/// Helper function to check for reserved task names and additional constraints
fn validate_task_name(name: &str) -> std::result::Result<(), ValidationError> {
    let reserved = [
        "watch",
        "default_task",
        "pre_deps",
        "post_deps",
        "concurrently",
    ];

    // Check reserved words
    if reserved.contains(&name) {
        let mut err = ValidationError::new("reserved_name");
        err.message = Some(format!("'{}' is a reserved task name", name).into());
        return Err(err);
    }

    // Check length (1 to 100)
    if name.is_empty() || name.len() > 100 {
        let mut err = ValidationError::new("invalid_length");
        err.message = Some("Task name length must be between 1 and 100".into());
        return Err(err);
    }

    // Disallow '/', '.' or '..'
    if name.contains('/') || name.contains("..") || name.contains('.') {
        let mut err = ValidationError::new("invalid_chars");
        err.message = Some("Task name cannot contain '/', '.' or '..'".into());
        return Err(err);
    }

    Ok(())
}

/// Multi-field validation for TaskConfig
fn validate_task_config(task: &TaskConfig) -> std::result::Result<(), ValidationError> {
    // If there's no command, at least one of pre_deps, post_deps, or concurrently must be non-empty
    let no_command = task.command.is_none();
    let no_pre = task.pre_deps.is_empty();
    let no_post = task.post_deps.is_empty();
    let no_concur = task.concurrently.is_empty();

    if no_command && no_pre && no_post && no_concur {
        let mut err = ValidationError::new("no_op");
        err.message = Some("A task must have a command or some dependencies".into());
        return Err(err);
    }

    // Validate timeout format if present
    if let Some(timeout) = &task.timeout {
        if humantime::parse_duration(timeout).is_err() {
            let mut err = ValidationError::new("invalid_timeout");
            err.message = Some(format!("Invalid timeout format: {}", timeout).into());
            return Err(err);
        }
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

/// Represents a CLI argument that can be passed to a task
#[derive(Debug, Clone, Serialize, Deserialize, Validate, JsonSchema, PartialEq)]
pub struct TaskArgument {
    /// Name of the argument (used as environment variable)
    #[validate(length(min = 1, max = 64))]
    pub name: String,

    /// Description of what the argument is for
    #[validate(length(min = 0, max = 128))]
    pub description: Option<String>,

    /// Whether this argument must be provided
    #[serde(default)]
    pub required: bool,

    /// Default value if not provided
    pub default: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Validate, JsonSchema)]
#[validate(schema(function = "validate_task_config"))]
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

    /// Timeout duration (e.g. "30s", "1m")
    pub timeout: Option<String>,

    /// Environment variables for the task
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Additional paths to add to PATH
    #[serde(default)]
    pub exec_paths: Vec<String>,

    /// CLI arguments that can be passed to this task
    #[serde(default, rename = "args")]
    pub arguments: Vec<TaskArgument>,

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

#[cfg(test)]
mod tests {
    use super::*;
    use validator::ValidationErrors;

    #[test]
    fn test_validate_task_name_reserved() {
        let mut config = TaskConfig::default();
        config._name_check = Some("default_task".to_string());
        let result = config.validate();
        assert!(matches!(result, Err(ValidationErrors { .. })));
    }

    #[test]
    fn test_task_argument_boundaries() {
        // Test valid name length
        let valid_arg = TaskArgument {
            name: "a".repeat(64),
            description: Some("x".repeat(128)),
            ..Default::default()
        };
        assert!(valid_arg.validate().is_ok());

        // Test invalid name length
        let invalid_name = TaskArgument {
            name: "a".repeat(65),
            ..valid_arg.clone()
        };
        assert!(invalid_name.validate().is_err());

        // Test invalid description length
        let invalid_desc = TaskArgument {
            description: Some("x".repeat(129)),
            ..valid_arg
        };
        assert!(invalid_desc.validate().is_err());
    }

    #[test]
    fn test_valid_task_config_variations() {
        // Valid task with command only
        let valid_with_cmd = TaskConfig {
            command: Some("echo valid".to_string()),
            ..Default::default()
        };
        assert!(valid_with_cmd.validate().is_ok());

        // Valid task with dependencies only
        let valid_with_deps = TaskConfig {
            pre_deps: vec![Dependency::Command {
                command: "echo pre".to_string(),
            }],
            ..Default::default()
        };
        assert!(valid_with_deps.validate().is_ok());

        // Valid task with concurrent deps
        let valid_concurrent = TaskConfig {
            concurrently: vec![Dependency::Task {
                task: "other-task".to_string(),
            }],
            ..Default::default()
        };
        assert!(valid_concurrent.validate().is_ok());
    }

    #[test]
    fn test_watch_config_validation() {
        let valid_watch = WatchConfig {
            patterns: vec!["**/*.rs".to_string()],
            debounce_ms: 1000,
            ignore_patterns: vec!["target/**".to_string()],
            ..Default::default()
        };
        assert!(valid_watch.validate().is_ok());
    }

    #[test]
    fn test_concurrently_options_validation() {
        let valid_opts = ConcurrentlyOptions {
            max_concurrent_tasks: Some(10),
            ..Default::default()
        };
        assert!(valid_opts.validate().is_ok());

        let invalid_opts = ConcurrentlyOptions {
            max_concurrent_tasks: Some(0),
            ..Default::default()
        };
        assert!(invalid_opts.validate().is_err());
    }

    #[test]
    fn test_dependency_deserialization_variants() {
        let yaml = r#"
        - task: "complex::task::name"
        - command: "echo 'hello world' | grep hello"
        "#;
        let deps: Vec<Dependency> = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(deps.len(), 2);
        assert!(matches!(&deps[0], Dependency::Task { task } if task == "complex::task::name"));
        assert!(
            matches!(&deps[1], Dependency::Command { command } if command == "echo 'hello world' | grep hello")
        );
    }

    #[test]
    fn test_timeout_validation_edge_cases() {
        let mut valid_task = TaskConfig {
            timeout: Some("1m30s".to_string()),
            ..Default::default()
        };
        valid_task._name_check = Some("valid".to_string());
        assert!(valid_task.validate().is_ok());

        valid_task.timeout = Some("10x".to_string());
        let result = valid_task.validate();
        assert!(matches!(result, Err(ValidationErrors { .. })));
    }

    #[test]
    fn test_load_config_with_watch_settings() {
        let yaml = r#"
        scripts_dirs: ["scripts"]
        tasks:
          watch-task:
            watch:
              patterns: ["src/**/*"]
              debounce_ms: 200
        "#;
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), yaml).unwrap();
        let result = BodoConfig::load(Some(temp_file.path().to_str().unwrap().to_string()));
        assert!(result.is_ok());
    }

    #[test]
    fn test_no_op_task_validation() {
        let invalid_task = TaskConfig::default();
        let result = invalid_task.validate();
        assert!(matches!(result, Err(ValidationErrors { .. })));
    }

    #[test]
    fn test_task_config_with_mixed_dependencies() {
        let valid_mixed = TaskConfig {
            command: Some("echo main".to_string()),
            pre_deps: vec![Dependency::Command {
                command: "echo pre".to_string(),
            }],
            post_deps: vec![Dependency::Task {
                task: "cleanup".to_string(),
            }],
            ..Default::default()
        };
        assert!(valid_mixed.validate().is_ok());
    }
}
