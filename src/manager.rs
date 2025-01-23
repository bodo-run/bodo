use crate::{
    config::{BodoConfig, TaskConfig},
    errors::BodoError,
    graph::{Graph, NodeKind, TaskData},
    plugin::{PluginConfig, PluginManager},
    script_loader::ScriptLoader,
    task::TaskManager,
    Result,
};

/// Simplified GraphManager that no longer references ScriptLoader.
pub struct GraphManager {
    pub config: BodoConfig,
    pub graph: Graph,
    pub plugin_manager: PluginManager,
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

    pub async fn build_graph(&mut self, config: BodoConfig) -> Result<&Graph> {
        self.config = config.clone();
        let mut loader = ScriptLoader::new();
        self.graph = loader.build_graph(config).await?;
        Ok(&self.graph)
    }

    pub async fn initialize(&mut self) -> Result<()> {
        let config = BodoConfig {
            root_script: Some("scripts/main.yaml".into()),
            scripts_dirs: Some(vec!["scripts/".into()]),
            tasks: Default::default(),
        };
        self.build_graph(config).await?;
        Ok(())
    }

    pub async fn run_plugins(&mut self, config: Option<PluginConfig>) -> Result<()> {
        let cfg = config.unwrap_or_default();
        self.plugin_manager.sort_plugins();
        self.plugin_manager
            .run_lifecycle(&mut self.graph, Some(cfg))
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
                if t.is_default {
                    return Some(t);
                }
            }
        }
        None
    }

    pub fn get_task_script_name(&self, task_name: &str) -> Option<String> {
        self.get_task_by_name(task_name)
            .and_then(|t| t.script_name.clone())
    }

    pub async fn run_task(&self, task_name: &str) -> Result<()> {
        let task = if let Some(node_id) = self.graph.task_registry.get(task_name) {
            // First try to find a task by exact name in the registry
            let node = self
                .graph
                .nodes
                .get(*node_id as usize)
                .ok_or_else(|| BodoError::PluginError("Invalid node ID".to_string()))?;
            if let NodeKind::Task(task_data) = &node.kind {
                task_data
            } else {
                return Err(BodoError::PluginError(
                    "Registry points to non-task node".to_string(),
                ));
            }
        } else {
            // If not found in registry, try to find by name
            self.get_task_by_name(task_name)
                .ok_or_else(|| BodoError::TaskNotFound(task_name.to_string()))?
        };

        // Create a task config from the task data
        let task_config = TaskConfig {
            description: task.description.clone(),
            command: task.command.clone(),
            cwd: task.working_dir.clone(),
            pre_deps: Vec::new(), // TODO: Implement dependency resolution
            post_deps: Vec::new(),
            watch: None,
            timeout: None,
            env: task.env.clone(),
        };

        let mut task_manager = TaskManager::new(task_config, &self.plugin_manager);
        task_manager.run_task(task_name).map_err(|e| {
            BodoError::PluginError(format!("Failed to run task {}: {}", task_name, e))
        })?;

        Ok(())
    }

    pub fn get_task_name_by_name(&self, task_name: &str) -> Option<String> {
        self.get_task_by_name(task_name).map(|t| t.name.clone())
    }

    pub fn task_exists(&self, task_name: &str) -> bool {
        self.graph.task_registry.contains_key(task_name)
    }
}
