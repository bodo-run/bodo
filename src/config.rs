use serde::{Deserialize, Serialize};
use validator::ValidationError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodoConfig {
    pub timeout: u64,
    pub tasks: Vec<TaskConfig>,
    pub watch: Option<WatchConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchConfig {
    pub paths: Vec<String>,
    pub ignore: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskConfig {
    pub name: String,
    pub command: String,
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub task: String,
    pub condition: Option<String>,
}

impl BodoConfig {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.timeout == 0 {
            return Err(ValidationError::new("timeout must be positive"));
        }
        Ok(())
    }
}

impl TaskConfig {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.name.is_empty() {
            return Err(ValidationError::new("task name cannot be empty"));
        }
        Ok(())
    }
}
