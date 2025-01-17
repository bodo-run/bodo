use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
}

impl BodoConfig {
    pub async fn load(_config_path: Option<String>) -> Result<Self> {
        Ok(BodoConfig::default())
    }
}
