use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::{
    config::{BodoConfig, TaskConfig},
    errors::{BodoError, Result},
    graph::{Graph, NodeKind, TaskData},
};

/// Simplified ScriptLoader that handles loading task configurations from files.
pub struct ScriptLoader {
    // Track tasks across all files: "fileLabel#taskName" -> node ID
    pub name_to_id: HashMap<String, u64>,
    pub task_registry: HashMap<String, u64>,
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
            task_registry: HashMap::new(),
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

        // Load additional scripts from scripts_dirs
        if let Some(ref scripts_dirs) = config.scripts_dirs {
            for dir in scripts_dirs {
                let dir_path = PathBuf::from(dir);
                if !dir_path.exists() {
                    continue;
                }

                // Use walkdir to traverse directories recursively
                for entry in WalkDir::new(&dir_path)
                    .follow_links(true)
                    .into_iter()
                    .filter_map(|e| e.ok())
                {
                    let path = entry.path().to_path_buf();
                    let file_name = path.file_name().map(|n| n.to_string_lossy());

                    if let Some(name) = file_name {
                        if name == "script.yaml" || name == "script.yml" {
                            // For script.yaml files, use the parent directory name
                            let script_name = path
                                .parent()
                                .and_then(|p| p.file_name())
                                .map(|n| n.to_string_lossy().into_owned())
                                .unwrap_or_else(|| "default".to_string());
                            paths_to_load.push((path, script_name));
                        } else if name.ends_with(".yaml") || name.ends_with(".yml") {
                            // For other YAML files, use the file stem
                            let script_name = path
                                .file_stem()
                                .map(|n| n.to_string_lossy().into_owned())
                                .unwrap_or_else(|| "default".to_string());
                            paths_to_load.push((path, script_name));
                        }
                    }
                }
            }
        }

        // Load each script file
        for (path, script_name) in paths_to_load {
            self.load_script(&mut graph, &path, &script_name)?;
        }

        Ok(graph)
    }

    fn load_script(&mut self, graph: &mut Graph, path: &Path, script_name: &str) -> Result<()> {
        let contents = fs::read_to_string(path)?;
        let yaml = match serde_yaml::from_str::<serde_yaml::Value>(&contents) {
            Ok(yaml) => yaml,
            Err(e) => {
                eprintln!(
                    "Warning: Skipping invalid YAML file {}: {}",
                    path.display(),
                    e
                );
                return Ok(());
            }
        };

        // Load default task if present
        if let Some(default_task) = yaml.get("default_task") {
            let task_config = match serde_yaml::from_value(default_task.clone()) {
                Ok(config) => config,
                Err(e) => {
                    eprintln!("Warning: Invalid default task in {}: {}", path.display(), e);
                    return Ok(());
                }
            };
            let default_id = self.create_task_node(graph, script_name, "default", &task_config);
            self.register_task(script_name, "default", default_id, graph)?;
        }

        // Load tasks map
        let tasks_map = yaml
            .get("tasks")
            .and_then(|v| v.as_mapping())
            .map(|m| {
                m.iter()
                    .filter_map(|(k, v)| {
                        let name = k.as_str().unwrap_or_default().to_string();
                        match serde_yaml::from_value(v.clone()) {
                            Ok(config) => Some(Ok((name, config))),
                            Err(e) => {
                                eprintln!(
                                    "Warning: Invalid task '{}' in {}: {}",
                                    name,
                                    path.display(),
                                    e
                                );
                                None
                            }
                        }
                    })
                    .collect::<Result<HashMap<String, TaskConfig>>>()
            })
            .transpose()?
            .unwrap_or_default();

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
            name: name.to_string(),
            description: cfg.description.clone(),
            command: cfg.command.clone(),
            working_dir: cfg.cwd.clone(),
            is_default: name == "default",
            script_name: Some(script_name.to_string()),
            env: cfg.env.clone(),
        };

        graph.add_node(NodeKind::Task(task_data))
    }

    fn register_task(
        &mut self,
        script_name: &str,
        task_name: &str,
        node_id: u64,
        graph: &mut Graph,
    ) -> Result<()> {
        // Register with full key
        let full_key = format!("{}#{}", script_name, task_name);
        if graph.task_registry.contains_key(&full_key) {
            return Err(BodoError::PluginError(format!(
                "Duplicate task name: {}",
                task_name
            )));
        }
        self.name_to_id.insert(full_key.clone(), node_id);
        graph.task_registry.insert(full_key, node_id);

        // Register default task under script name
        if task_name == "default" {
            let script_key = script_name.to_string();
            if !graph.task_registry.contains_key(&script_key) {
                graph.task_registry.insert(script_key, node_id);
            }
        }

        // Register task under its name if it doesn't conflict
        let task_key = task_name.to_string();
        if !graph.task_registry.contains_key(&task_key) {
            graph.task_registry.insert(task_key, node_id);
        }

        Ok(())
    }
}
