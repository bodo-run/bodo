use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::{
    config::{BodoConfig, TaskConfig},
    errors::{BodoError, Result},
    graph::{Graph, NodeKind, TaskData},
};

/// Simplified ScriptLoader that handles loading task configurations from files.
pub struct ScriptLoader {
    // Track tasks across all files: "fileLabel#taskName" -> node ID
    name_to_id: HashMap<String, u64>,
}

impl Default for ScriptLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptLoader {
    pub fn new() -> Self {
        Self {
            name_to_id: HashMap::new(),
        }
    }

    /// Stub that returns an empty Graph.
    /// In a real implementation, parse your scripts and fill the graph.
    pub async fn build_graph(&mut self, config: BodoConfig) -> Result<Graph> {
        let mut graph = Graph::new();
        let mut paths_to_load = vec![];

        // Load the root script if exists
        if let Some(ref root_script) = config.root_script {
            paths_to_load.push(PathBuf::from(root_script));
        }

        // Load from scripts_dir if configured
        if let Some(ref scripts_dirs) = config.scripts_dirs {
            for scripts_dir in scripts_dirs {
                let dir_path = PathBuf::from(scripts_dir);
                if dir_path.exists() && dir_path.is_dir() {
                    let root_script = dir_path.join("script.yaml");
                    if root_script.exists() {
                        paths_to_load.push(root_script);
                    }
                }
            }
        }

        // Load each file in sequence
        for path in paths_to_load {
            if path.is_file() {
                if let Err(e) = self.load_one_file(&path, &mut graph) {
                    eprintln!("Warning: could not parse {:?}: {}", path, e);
                }
            }
        }

        Ok(graph)
    }

    fn register_task(
        &mut self,
        _file_label: &str,
        task_name: &str,
        node_id: u64,
        graph: &mut Graph,
    ) -> Result<()> {
        // Use simple task name as registry key
        let key = task_name.to_string();

        // Check for name collisions
        if graph.task_registry.contains_key(&key) {
            return Err(BodoError::PluginError(format!(
                "Task name collision: {}",
                key
            )));
        }

        self.name_to_id.insert(key.clone(), node_id);
        graph.task_registry.insert(key, node_id);

        Ok(())
    }

    fn load_one_file(&mut self, path: &Path, graph: &mut Graph) -> Result<()> {
        let file_stem = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let contents = fs::read_to_string(path)?;
        let yaml: serde_yaml::Value = serde_yaml::from_str(&contents).map_err(|e| {
            BodoError::PluginError(format!("YAML parse error in {:?}: {}", path, e))
        })?;

        let default_obj = yaml
            .get("default_task")
            .cloned()
            .unwrap_or_else(|| serde_yaml::Value::Mapping(Default::default()));
        let tasks_obj = yaml
            .get("tasks")
            .cloned()
            .unwrap_or_else(|| serde_yaml::Value::Mapping(Default::default()));

        let default_task: TaskConfig = serde_yaml::from_value(default_obj)
            .map_err(|e| BodoError::PluginError(format!("Cannot parse default_task: {e}")))?;

        let tasks_map: HashMap<String, TaskConfig> =
            if let serde_yaml::Value::Mapping(_) = tasks_obj {
                serde_yaml::from_value(tasks_obj)
                    .map_err(|e| BodoError::PluginError(format!("Cannot parse tasks: {e}")))?
            } else {
                HashMap::new()
            };

        // Create nodes for each task
        let default_id = self.create_task_node(graph, &file_stem, "default", &default_task);
        self.register_task(&file_stem, "default", default_id, graph)?;

        for (name, task_config) in tasks_map {
            let task_id = self.create_task_node(graph, &file_stem, &name, &task_config);
            self.register_task(&file_stem, &name, task_id, graph)?;

            let node = &mut graph.nodes[task_id as usize];
            if !task_config.pre_deps.is_empty() {
                node.metadata.insert(
                    "pre_deps".to_string(),
                    serde_json::to_string(&task_config.pre_deps).unwrap(),
                );
            }
            if !task_config.post_deps.is_empty() {
                node.metadata.insert(
                    "post_deps".to_string(),
                    serde_json::to_string(&task_config.post_deps).unwrap(),
                );
            }
        }

        Ok(())
    }

    fn create_task_node(
        &self,
        graph: &mut Graph,
        file_label: &str,
        name: &str,
        cfg: &TaskConfig,
    ) -> u64 {
        let task_data = TaskData {
            name: name.to_string(),
            description: cfg.description.clone(),
            command: cfg.command.clone(),
            working_dir: cfg.cwd.clone(),
            env: cfg.env.clone(),
            is_default: false,
            script_name: Some(file_label.to_string()),
        };
        let node_id = graph.add_node(NodeKind::Task(task_data));

        // Add watch config to metadata if present
        if let Some(watch) = &cfg.watch {
            let node = &mut graph.nodes[node_id as usize];
            node.metadata
                .insert("watch".to_string(), serde_json::to_string(watch).unwrap());
        }

        if let Some(timeout) = &cfg.timeout {
            let node = &mut graph.nodes[node_id as usize];
            node.metadata.insert("timeout".to_string(), timeout.clone());
        }

        node_id
    }
}
