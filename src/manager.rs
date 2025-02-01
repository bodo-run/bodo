use crate::{
    config::{BodoConfig, TaskConfig},
    errors::BodoError,
    graph::{Graph, NodeKind, TaskData},
    plugin::{PluginConfig, PluginManager},
    script_loader::ScriptLoader,
    Result,
};

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

    pub fn build_graph(&mut self, config: BodoConfig) -> Result<&Graph> {
        self.config = config.clone();
        let mut loader = ScriptLoader::new();
        self.graph = loader.build_graph(config)?;

        if let Some(cycle) = self.graph.detect_cycle() {
            let error_msg = self.graph.format_cycle_error(&cycle);
            return Err(BodoError::PluginError(error_msg));
        }
        Ok(&self.graph)
    }

    pub fn get_task_config(&self, task_name: &str) -> Result<TaskConfig> {
        let node_id = self
            .graph
            .task_registry
            .get(task_name)
            .ok_or_else(|| BodoError::TaskNotFound(task_name.to_string()))?;

        let node = self.graph.nodes.get(*node_id as usize).ok_or_else(|| {
            BodoError::PluginError(format!("Invalid node ID for task '{}'", task_name))
        })?;

        let task_data = match &node.kind {
            NodeKind::Task(t) => t,
            _ => {
                return Err(BodoError::PluginError(format!(
                    "Node '{}' is not a Task node",
                    task_name
                )));
            }
        };

        Ok(TaskConfig {
            description: task_data.description.clone(),
            command: task_data.command.clone(),
            cwd: task_data.working_dir.clone(),
            env: task_data.env.clone(),
            pre_deps: Vec::new(),
            post_deps: Vec::new(),
            watch: None,
            timeout: None,
            concurrently_options: Default::default(),
            concurrently: vec![],
            exec_paths: task_data.exec_paths.clone(),
            _name_check: None,
        })
    }

    pub fn initialize(&mut self) -> Result<()> {
        let config = BodoConfig {
            root_script: Some("scripts/main.yaml".into()),
            scripts_dirs: Some(vec!["scripts/".into()]),
            tasks: Default::default(),
            env: Default::default(),
            exec_paths: Default::default(),
        };
        self.build_graph(config)?;
        Ok(())
    }

    pub fn run_plugins(&mut self, config: Option<PluginConfig>) -> Result<()> {
        self.plugin_manager.sort_plugins();
        self.plugin_manager.run_lifecycle(&mut self.graph, config)?;
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

    pub fn run_task(&mut self, task_name: &str) -> Result<()> {
        // If we wanted a simpler entry point to run a single task (bypassing the plugin pipeline).
        // Currently not used, but you could call it in watch plugin if desired.
        let _node_id = *self
            .graph
            .task_registry
            .get(task_name)
            .ok_or_else(|| BodoError::TaskNotFound(task_name.to_string()))?;
        // your synchronous run logic here or reuse plugin approach
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
