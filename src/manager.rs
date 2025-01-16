use crate::{
    config::BodoConfig,
    errors::BodoError,
    graph::{Graph, NodeKind, TaskData},
    plugin::{PluginConfig, PluginManager},
    script_loader::ScriptLoader,
    Result,
};

pub struct GraphManager {
    pub config: BodoConfig,
    pub graph: Graph,
    plugin_manager: PluginManager,
    script_loader: ScriptLoader,
}

impl GraphManager {
    pub fn new() -> Self {
        Self {
            config: BodoConfig::default(),
            graph: Graph::new(),
            plugin_manager: PluginManager::new(),
            script_loader: ScriptLoader::new(),
        }
    }

    pub fn register_plugin(&mut self, plugin: Box<dyn crate::plugin::Plugin>) {
        self.plugin_manager.register(plugin);
    }

    pub async fn load_bodo_config(&mut self, config_path: Option<String>) -> Result<()> {
        self.config = BodoConfig::load(config_path).await?;
        Ok(())
    }

    pub async fn build_graph(&mut self, config: BodoConfig) -> Result<()> {
        self.graph = self.script_loader.build_graph(config).await?;
        Ok(())
    }

    pub async fn run_plugins(&mut self, config: Option<PluginConfig>) -> Result<()> {
        let cfg = config.unwrap_or_default();
        self.plugin_manager
            .run_lifecycle(&mut self.graph, &cfg)
            .await?;
        Ok(())
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
        if let Some(t) = self.get_task_by_name(task_name) {
            Some(t.name.clone())
        } else {
            None
        }
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
}
