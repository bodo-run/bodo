use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use validator::Validate;
use walkdir::WalkDir;

use crate::{
    config::{BodoConfig, Dependency, TaskConfig},
    errors::{BodoError, Result},
    graph::{CommandData, Graph, NodeKind, TaskData},
};
use log::warn;

pub struct ScriptLoader {
    pub name_to_id: HashMap<String, u64>,
    pub task_registry: HashMap<String, u64>,
    loaded_scripts: HashMap<PathBuf, String>,
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
            loaded_scripts: HashMap::new(),
        }
    }

    // Helper function for merging environment variables
    fn merge_envs(
        global_env: &HashMap<String, String>,
        script_env: &HashMap<String, String>,
        task_env: &HashMap<String, String>,
    ) -> HashMap<String, String> {
        let mut result = global_env.clone();
        result.extend(script_env.clone());
        result.extend(task_env.clone());
        result
    }

    fn merge_exec_paths(
        global_paths: &[String],
        script_paths: &[String],
        task_paths: &[String],
    ) -> Vec<String> {
        let mut merged = Vec::new();
        merged.extend(global_paths.iter().cloned());
        merged.extend(script_paths.iter().cloned());
        merged.extend(task_paths.iter().cloned());
        merged
    }

    pub fn build_graph(&mut self, config: BodoConfig) -> Result<Graph> {
        config.validate().map_err(BodoError::from)?;

        let mut graph = Graph::new();
        let mut paths_to_load = vec![];
        let mut root_script_abs: Option<PathBuf> = None;

        let global_env = config.env.clone();
        let global_exec_paths = config.exec_paths.clone();

        if let Some(ref root_script) = config.root_script {
            let root_path = PathBuf::from(root_script);
            if root_path.exists() {
                root_script_abs = Some(root_path.canonicalize()?);
                paths_to_load.insert(0, (root_path, "".to_string()));
            }
        }

        if let Some(ref scripts_dirs) = config.scripts_dirs {
            for dir in scripts_dirs {
                let dir_path = PathBuf::from(dir);
                if !dir_path.exists() {
                    continue;
                }
                for entry in WalkDir::new(&dir_path)
                    .follow_links(true)
                    .into_iter()
                    .filter_map(|e| e.ok())
                {
                    let path = entry.path().to_path_buf();
                    if let Some(ref root_abs) = root_script_abs {
                        if let Ok(candidate) = path.canonicalize() {
                            if candidate == *root_abs {
                                continue;
                            }
                        }
                    }
                    let file_name = path.file_name().and_then(|n| n.to_str());
                    if let Some(name) = file_name {
                        if name == "script.yaml" || name == "script.yml" {
                            let script_display = path
                                .parent()
                                .and_then(|p| p.file_name())
                                .and_then(|n| n.to_str())
                                .unwrap_or("default")
                                .to_string();
                            paths_to_load.push((path, script_display));
                        } else if name.ends_with(".yaml") || name.ends_with(".yml") {
                            let script_display = path
                                .file_stem()
                                .and_then(|n| n.to_str())
                                .unwrap_or("default")
                                .to_string();
                            paths_to_load.push((path, script_display));
                        }
                    }
                }
            }
        }

        for (path, script_name) in paths_to_load {
            self.load_script(
                &mut graph,
                &path,
                &script_name,
                &global_env,
                &global_exec_paths,
            )?;
        }

        // Process tasks from config.tasks
        if !config.tasks.is_empty() {
            let script_id = "config".to_string();
            let script_display_name = "config".to_string();
            let script_env = HashMap::new();
            let script_exec_paths = vec![];

            for (name, task_config) in config.tasks {
                // Merge environments and exec_paths for each task
                let env = Self::merge_envs(&global_env, &script_env, &task_config.env);
                let exec_paths = Self::merge_exec_paths(
                    &global_exec_paths,
                    &script_exec_paths,
                    &task_config.exec_paths,
                );

                // Validate task config
                self.validate_task_config(&task_config, &name, Path::new("config"))?;

                // Create task node
                let task_id = self.create_task_node(
                    &mut graph,
                    &script_id,
                    &script_display_name,
                    &name,
                    &task_config,
                );

                // Update the task data with merged env and exec_paths
                if let NodeKind::Task(ref mut task_data) = graph.nodes[task_id as usize].kind {
                    task_data.env = env;
                    task_data.exec_paths = exec_paths;
                }

                self.register_task(&script_id, &name, task_id, &mut graph)?;

                // Handle dependencies
                for dep in &task_config.pre_deps {
                    match dep {
                        Dependency::Task { task } => {
                            let dep_id =
                                self.resolve_dependency(task, Path::new("config"), &mut graph)?;
                            graph.add_edge(dep_id, task_id)?;
                        }
                        Dependency::Command { command } => {
                            let cmd_node_id = graph.add_node(NodeKind::Command(CommandData {
                                raw_command: command.clone(),
                                description: None,
                                working_dir: None,
                                env: std::collections::HashMap::new(),
                                watch: None,
                            }));
                            graph.add_edge(cmd_node_id, task_id)?;
                        }
                    }
                }

                for dep in &task_config.post_deps {
                    match dep {
                        Dependency::Task { task } => {
                            let dep_id =
                                self.resolve_dependency(task, Path::new("config"), &mut graph)?;
                            graph.add_edge(task_id, dep_id)?;
                        }
                        Dependency::Command { command } => {
                            let cmd_node_id = graph.add_node(NodeKind::Command(CommandData {
                                raw_command: command.clone(),
                                description: None,
                                working_dir: None,
                                env: std::collections::HashMap::new(),
                                watch: None,
                            }));
                            graph.add_edge(task_id, cmd_node_id)?;
                        }
                    }
                }
            }
        }

        Ok(graph)
    }

    pub fn load_script(
        &mut self,
        graph: &mut Graph,
        path: &Path,
        script_display_name: &str,
        global_env: &HashMap<String, String>,
        global_exec_paths: &[String],
    ) -> Result<()> {
        // Read and parse the script file
        let content = fs::read_to_string(path)?;
        let script_config: BodoConfig = serde_yaml::from_str(&content)?;

        let script_env = script_config.env.clone();
        let script_exec_paths = script_config.exec_paths.clone();

        // Process default_task if present
        let script_id = path.display().to_string();

        if let Some(default_task_config) = script_config.default_task {
            let default_task_name = "default";
            self.validate_task_config(&default_task_config, default_task_name, path)?;

            // Merge environments and exec_paths for default_task
            let env = Self::merge_envs(global_env, &script_env, &default_task_config.env);
            let exec_paths = Self::merge_exec_paths(
                global_exec_paths,
                &script_exec_paths,
                &default_task_config.exec_paths,
            );

            let node_id = self.create_task_node(
                graph,
                &script_id,
                script_display_name,
                default_task_name, // Using "default" as the task name
                &default_task_config,
            );

            // Update the task data with merged env and exec_paths
            if let NodeKind::Task(ref mut task_data) = graph.nodes[node_id as usize].kind {
                task_data.env = env;
                task_data.exec_paths = exec_paths;
            }

            self.register_task(&script_id, default_task_name, node_id, graph)?;

            // Handle dependencies
            for dep in &default_task_config.pre_deps {
                match dep {
                    Dependency::Task { task } => {
                        let dep_id = self.resolve_dependency(task, path, graph)?;
                        graph.add_edge(dep_id, node_id)?;
                    }
                    Dependency::Command { command } => {
                        let cmd_node_id = graph.add_node(NodeKind::Command(CommandData {
                            raw_command: command.clone(),
                            description: None,
                            working_dir: None,
                            env: HashMap::new(),
                            watch: None,
                        }));
                        graph.add_edge(cmd_node_id, node_id)?;
                    }
                }
            }

            for dep in &default_task_config.post_deps {
                match dep {
                    Dependency::Task { task } => {
                        let dep_id = self.resolve_dependency(task, path, graph)?;
                        graph.add_edge(node_id, dep_id)?;
                    }
                    Dependency::Command { command } => {
                        let cmd_node_id = graph.add_node(NodeKind::Command(CommandData {
                            raw_command: command.clone(),
                            description: None,
                            working_dir: None,
                            env: HashMap::new(),
                            watch: None,
                        }));
                        graph.add_edge(node_id, cmd_node_id)?;
                    }
                }
            }
        }

        // For each task in the script, create nodes in the graph
        for (task_name, task_config) in script_config.tasks {
            self.validate_task_config(&task_config, &task_name, path)?;

            // Merge environments and exec_paths for each task
            let env = Self::merge_envs(global_env, &script_env, &task_config.env);
            let exec_paths = Self::merge_exec_paths(
                global_exec_paths,
                &script_exec_paths,
                &task_config.exec_paths,
            );

            let node_id = self.create_task_node(
                graph,
                &script_id,
                script_display_name,
                &task_name,
                &task_config,
            );

            // Update the task data with merged env and exec_paths
            if let NodeKind::Task(ref mut task_data) = graph.nodes[node_id as usize].kind {
                task_data.env = env;
                task_data.exec_paths = exec_paths;
            }

            self.register_task(&script_id, &task_name, node_id, graph)?;

            // Handle dependencies
            for dep in &task_config.pre_deps {
                match dep {
                    Dependency::Task { task } => {
                        let dep_id = self.resolve_dependency(task, path, graph)?;
                        graph.add_edge(dep_id, node_id)?;
                    }
                    Dependency::Command { command } => {
                        let cmd_node_id = graph.add_node(NodeKind::Command(CommandData {
                            raw_command: command.clone(),
                            description: None,
                            working_dir: None,
                            env: HashMap::new(),
                            watch: None,
                        }));
                        graph.add_edge(cmd_node_id, node_id)?;
                    }
                }
            }

            for dep in &task_config.post_deps {
                match dep {
                    Dependency::Task { task } => {
                        let dep_id = self.resolve_dependency(task, path, graph)?;
                        graph.add_edge(node_id, dep_id)?;
                    }
                    Dependency::Command { command } => {
                        let cmd_node_id = graph.add_node(NodeKind::Command(CommandData {
                            raw_command: command.clone(),
                            description: None,
                            working_dir: None,
                            env: HashMap::new(),
                            watch: None,
                        }));
                        graph.add_edge(node_id, cmd_node_id)?;
                    }
                }
            }
        }

        Ok(())
    }

    // Other methods remain the same
}
