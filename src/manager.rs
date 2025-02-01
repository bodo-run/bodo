use crate::{
    config::{BodoConfig, ConcurrentlyOptions, Dependency, TaskConfig},
    errors::BodoError,
    graph::{Graph, NodeKind, TaskData},
    plugin::{PluginConfig, PluginManager},
    script_loader::ScriptLoader,
    task::TaskManager,
    Result,
};
use std::collections::HashMap;

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

        // Check for cycles after building the graph
        if self.graph.has_cycle() {
            return Err(BodoError::PluginError(
                "Circular dependency detected in task graph".to_string(),
            ));
        }

        Ok(&self.graph)
    }

    pub fn get_task_config(&self, task_name: &str) -> Result<TaskConfig> {
        // Look up the node ID in the task registry
        let node_id = self
            .graph
            .task_registry
            .get(task_name)
            .ok_or_else(|| BodoError::TaskNotFound(task_name.to_string()))?;

        // Grab the node from the graph
        let node = self.graph.nodes.get(*node_id as usize).ok_or_else(|| {
            BodoError::PluginError(format!("Invalid node ID for task '{}'", task_name))
        })?;

        // Ensure it's actually a Task node
        let task_data = match &node.kind {
            NodeKind::Task(t) => t,
            _ => {
                return Err(BodoError::PluginError(format!(
                    "Node '{}' is not a Task node",
                    task_name
                )));
            }
        };

        // Convert TaskData -> TaskConfig
        // (If you want pre_deps/post_deps, you must store them somewhere,
        //  or you can return empty arrays as shown here.)
        Ok(TaskConfig {
            description: task_data.description.clone(),
            command: task_data.command.clone(),
            cwd: task_data.working_dir.clone(),
            env: task_data.env.clone(),
            // Set these to empty if you do not store them in metadata:
            pre_deps: Vec::new(),
            post_deps: Vec::new(),
            watch: None,
            timeout: None,

            // If your `TaskConfig` also has concurrency fields, set them:
            concurrently_options: Default::default(),
            concurrently: vec![],
        })
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
            .map(|t| t.script_id.clone())
    }

    pub async fn run_task(&self, task_name: &str) -> Result<()> {
        let node_id = self
            .graph
            .task_registry
            .get(task_name)
            .ok_or_else(|| BodoError::TaskNotFound(task_name.to_string()))?;

        let node = &self.graph.nodes[*node_id as usize];
        match &node.kind {
            NodeKind::Task(task_data) => {
                // Create a task config from the task data
                let task_config = TaskConfig {
                    description: task_data.description.clone(),
                    command: task_data.command.clone(),
                    cwd: task_data.working_dir.clone(),
                    pre_deps: Vec::new(), // TODO: Implement dependency resolution
                    post_deps: Vec::new(),
                    watch: None,
                    timeout: None,
                    env: task_data.env.clone(),
                    concurrently_options: Default::default(),
                    concurrently: Vec::new(),
                };

                let task_manager = TaskManager::new(task_config, &self.plugin_manager);
                task_manager.run_task(task_name).map_err(|e| {
                    BodoError::PluginError(format!("Failed to run task {}: {}", task_name, e))
                })?;
            }
            NodeKind::ConcurrentGroup(group_data) => {
                // Get concurrent items from metadata
                let concurrent_items: Vec<Dependency> = node
                    .metadata
                    .get("concurrently_json")
                    .and_then(|json| serde_json::from_str(json).ok())
                    .unwrap_or_default();

                // For concurrent groups, create a task config with the concurrent settings
                let task_config = TaskConfig {
                    description: None,
                    command: None,
                    cwd: None,
                    pre_deps: Vec::new(),
                    post_deps: Vec::new(),
                    watch: None,
                    timeout: None,
                    env: HashMap::new(),
                    concurrently_options: ConcurrentlyOptions {
                        fail_fast: Some(group_data.fail_fast),
                        max_concurrent_tasks: group_data.max_concurrent,
                    },
                    concurrently: concurrent_items,
                };

                let task_manager = TaskManager::new(task_config, &self.plugin_manager);
                task_manager.run_task(task_name).map_err(|e| {
                    BodoError::PluginError(format!(
                        "Failed to run concurrent group {}: {}",
                        task_name, e
                    ))
                })?;
            }
            NodeKind::Command(_) => {
                return Err(BodoError::PluginError(format!(
                    "Cannot directly run command node '{}'",
                    task_name
                )));
            }
        }

        Ok(())
    }

    pub fn get_task_name_by_name(&self, task_name: &str) -> Option<String> {
        self.get_task_by_name(task_name).map(|t| t.name.clone())
    }

    pub fn task_exists(&self, task_name: &str) -> bool {
        self.graph.task_registry.contains_key(task_name)
    }

    pub fn get_all_tasks(&self) -> Vec<&TaskData> {
        self.graph
            .nodes
            .iter()
            .filter_map(|n| match &n.kind {
                NodeKind::Task(t) => Some(t),
                _ => None,
            })
            .collect()
    }
}
