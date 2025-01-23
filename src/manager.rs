use crate::{
    config::BodoConfig,
    errors::BodoError,
    graph::{Graph, NodeKind, TaskData},
    plugin::{PluginConfig, PluginManager},
    script_loader::ScriptLoader,
    Result,
};

/// Simplified GraphManager that no longer references ScriptLoader.
pub struct GraphManager {
    pub config: BodoConfig,
    pub graph: Graph,
    plugin_manager: PluginManager,
}

impl Default for GraphManager {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphManager {
    pub fn new() -> Self {
        Self {
            config: BodoConfig::default(),
            graph: Graph::new(),
            plugin_manager: PluginManager::new(),
        }
    }

    pub fn register_plugin(&mut self, plugin: Box<dyn crate::plugin::Plugin>) {
        self.plugin_manager.register(plugin);
    }

    pub async fn build_graph(&mut self, config: BodoConfig) -> Result<()> {
        self.config = config.clone();
        let mut loader = ScriptLoader::new();
        self.graph = loader.build_graph(config).await?;
        Ok(())
    }

    pub async fn run_plugins(&mut self, config: Option<PluginConfig>) -> Result<()> {
        let cfg = config.unwrap_or_default();
        self.plugin_manager
            .run_lifecycle(&mut self.graph, &cfg)
            .await?;
        Ok(())
    }

    pub fn get_tasks(&self) -> Vec<&TaskData> {
        self.graph
            .nodes
            .iter()
            .filter_map(|n| {
                if let NodeKind::Task(t) = &n.kind {
                    Some(t)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_task_by_name(&self, task_name: &str) -> Option<&TaskData> {
        for node in &self.graph.nodes {
            if let NodeKind::Task(t) = &node.kind {
                if t.name == task_name {
                    return Some(t);
                }
            }
        }
        None
    }

    pub fn get_default_task(&self) -> Option<&TaskData> {
        for node in &self.graph.nodes {
            if let NodeKind::Task(t) = &node.kind {
                return Some(t);
            }
        }
        None
    }

    pub fn get_task_script_name(&self, task_name: &str) -> Option<String> {
        self.get_task_by_name(task_name).map(|t| t.name.clone())
    }

    pub async fn run_task(&mut self, task_name: &str) -> Result<()> {
        let task = self
            .get_task_by_name(task_name)
            .ok_or_else(|| BodoError::PluginError(format!("Task not found: {}", task_name)))?;

        if let Some(cmd) = &task.command {
            println!("Running task: {}", task_name);
            println!("Command: {}", cmd);
            // Here we'd use tokio::process::Command to actually run the command
            // For now we just print
        }

        Ok(())
    }

    pub fn get_task_name_by_name(&self, task_name: &str) -> Option<String> {
        self.get_task_by_name(task_name).map(|t| t.name.clone())
    }
}
