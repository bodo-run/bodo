use crate::config::{BodoConfig, Dependency, TaskConfig};

pub struct ScriptLoader;

impl ScriptLoader {
    pub fn load_dependencies(config: &BodoConfig) -> Vec<Dependency> {
        config
            .tasks
            .iter()
            .flat_map(|task| task.dependencies.clone())
            .collect()
    }
}
