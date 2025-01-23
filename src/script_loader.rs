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

    /// Load scripts and build the task graph
    pub async fn build_graph(&mut self, config: BodoConfig) -> Result<Graph> {
        let mut graph = Graph::new();
        let mut paths_to_load = vec![];

        // Load the root script if exists
        if let Some(ref root_script) = config.root_script {
            let root_path = PathBuf::from(root_script);
            if root_path.exists() {
                paths_to_load.push((root_path, "default".to_string()));
            }
        }

        // Load from scripts_dir if configured
        if let Some(ref scripts_dirs) = config.scripts_dirs {
            for scripts_dir in scripts_dirs {
                let dir_path = PathBuf::from(scripts_dir);
                if dir_path.exists() && dir_path.is_dir() {
                    // First try to load script.yaml in the root
                    let root_script = dir_path.join("script.yaml");
                    if root_script.exists() {
                        paths_to_load.push((root_script, "default".to_string()));
                    }

                    // Then load all subdirectories
                    if let Ok(entries) = fs::read_dir(&dir_path) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if path.is_dir() {
                                let script_path = path.join("script.yaml");
                                if script_path.exists() {
                                    let script_name = path
                                        .file_name()
                                        .unwrap_or_default()
                                        .to_string_lossy()
                                        .to_string();
                                    paths_to_load.push((script_path, script_name));
                                }
                            }
                        }
                    }
                }
            }
        }

        // Load each file in sequence
        for (path, script_name) in paths_to_load {
            if path.is_file() {
                if let Err(e) = self.load_one_file(&path, &script_name, &mut graph) {
                    eprintln!("Warning: could not parse {:?}: {}", path, e);
                }
            }
        }

        // Return empty graph if no tasks were loaded
        Ok(graph)
    }

    fn register_task(
        &mut self,
        script_name: &str,
        task_name: &str,
        node_id: u64,
        graph: &mut Graph,
    ) -> Result<()> {
        // Use script_name:task_name as registry key
        let key = if script_name == "default" {
            task_name.to_string()
        } else {
            format!("{}:{}", script_name, task_name)
        };

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

    fn load_one_file(&mut self, path: &Path, script_name: &str, graph: &mut Graph) -> Result<()> {
        let contents = fs::read_to_string(path)?;
        let yaml: serde_yaml::Value = serde_yaml::from_str(&contents).map_err(|e| {
            BodoError::PluginError(format!("YAML parse error in {:?}: {}", path, e))
        })?;

        // First try to load tasks
        let tasks_obj = yaml
            .get("tasks")
            .cloned()
            .unwrap_or_else(|| serde_yaml::Value::Mapping(Default::default()));

        let tasks_map: HashMap<String, TaskConfig> = match tasks_obj {
            serde_yaml::Value::Mapping(map) => {
                let mut result = HashMap::new();
                for (key, value) in map {
                    let key_str = key.as_str().ok_or_else(|| {
                        BodoError::PluginError("Task name must be a string".to_string())
                    })?;
                    let task_config: TaskConfig = serde_yaml::from_value(value).map_err(|e| {
                        BodoError::PluginError(format!("Cannot parse task {}: {}", key_str, e))
                    })?;
                    result.insert(key_str.to_string(), task_config);
                }
                result
            }
            _ => HashMap::new(),
        };

        // Then try to load default_task if present
        if let Some(default_obj) = yaml.get("default_task") {
            let default_task: TaskConfig = serde_yaml::from_value(default_obj.clone())
                .map_err(|e| BodoError::PluginError(format!("Cannot parse default_task: {e}")))?;

            let default_id = self.create_task_node(graph, script_name, "default", &default_task);
            self.register_task(script_name, "default", default_id, graph)?;
        }

        // Create nodes for each task
        for (name, task_config) in tasks_map {
            let task_id = self.create_task_node(graph, script_name, &name, &task_config);
            self.register_task(script_name, &name, task_id, graph)?;

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
        script_name: &str,
        name: &str,
        cfg: &TaskConfig,
    ) -> u64 {
        let task_data = TaskData {
            name: if script_name == "default" {
                name.to_string()
            } else {
                format!("{}:{}", script_name, name)
            },
            description: cfg.description.clone(),
            command: cfg.command.clone(),
            working_dir: cfg.cwd.clone(),
            env: cfg.env.clone(),
            is_default: name == "default",
            script_name: Some(script_name.to_string()),
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
