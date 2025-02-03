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
    pub fn merge_envs(
        global_env: &HashMap<String, String>,
        script_env: &HashMap<String, String>,
        task_env: &HashMap<String, String>,
    ) -> HashMap<String, String> {
        let mut merged = HashMap::new();
        // Start with global
        for (k, v) in global_env {
            merged.insert(k.clone(), v.clone());
        }
        // Merge script-level, overriding if keys conflict
        for (k, v) in script_env {
            merged.insert(k.clone(), v.clone());
        }
        // Merge task-level
        for (k, v) in task_env {
            merged.insert(k.clone(), v.clone());
        }
        merged
    }

    // Helper function for merging exec_paths
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
        // Validate the BodoConfig first
        config.validate().map_err(BodoError::from)?;

        let mut graph = Graph::new();
        let mut paths_to_load = vec![];
        let mut root_script_abs: Option<PathBuf> = None;

        // Store global env and exec_paths to pass to load_script
        let global_env = config.env.clone();
        let global_exec_paths = config.exec_paths.clone();

        if let Some(ref root_script) = config.root_script {
            let root_path = PathBuf::from(root_script);
            if root_path.exists() {
                root_script_abs = Some(root_path.canonicalize()?);
                paths_to_load.push((root_path, "".to_string()));
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

        // [New code] Process tasks from config.tasks
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
                let task_data = TaskData {
                    name: name.clone(),
                    description: task_config.description.clone(),
                    command: task_config.command.clone(),
                    working_dir: task_config.cwd.clone(),
                    env,
                    exec_paths,
                    is_default: name == "default",
                    script_id: script_id.clone(),
                    script_display_name: script_display_name.clone(),
                    watch: task_config.watch.clone(),
                };

                let task_id = graph.add_node(NodeKind::Task(task_data));
                self.register_task(&script_id, &name, task_id, &mut graph)?;

                // Handle dependencies
                for dep in &task_config.pre_deps {
                    match dep {
                        Dependency::Task { task } => {
                            let dep_id =
                                self.resolve_dependency(task, Path::new("config"), &mut graph)?;
                            graph.add_edge(*dep_id, task_id)?;
                        }
                        Dependency::Command { command } => {
                            let cmd_node_id = graph.add_node(NodeKind::Command(CommandData {
                                raw_command: command.clone(),
                                description: None,
                                working_dir: None,
                                env: HashMap::new(),
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
                            graph.add_edge(task_id, *dep_id)?;
                        }
                        Dependency::Command { command } => {
                            let cmd_node_id = graph.add_node(NodeKind::Command(CommandData {
                                raw_command: command.clone(),
                                description: None,
                                working_dir: None,
                                env: HashMap::new(),
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

    fn validate_task_config(
        &self,
        task_config: &TaskConfig,
        task_name: &str,
        path: &Path,
    ) -> Result<()> {
        // Set the task name for validation
        let mut task = task_config.clone();
        task._name_check = Some(task_name.to_string());

        // Run validation
        if let Err(e) = task.validate() {
            warn!("Invalid task '{}' in {}: {}", task_name, path.display(), e);
            return Err(BodoError::ValidationError(format!(
                "Task '{}' in {} failed validation: {}",
                task_name,
                path.display(),
                e
            )));
        }
        Ok(())
    }

    pub fn load_script(
        &mut self,
        graph: &mut Graph,
        path: &Path,
        script_id: &str,
        global_env: &HashMap<String, String>,
        global_exec_paths: &[String],
    ) -> Result<()> {
        // ...
        // Existing implementation of load_script...
        // Omitted here for brevity
    }

    fn create_task_node(
        &self,
        graph: &mut Graph,
        script_id: &str,
        script_display_name: &str,
        name: &str,
        cfg: &TaskConfig,
    ) -> u64 {
        let task_data = TaskData {
            name: name.to_string(),
            description: cfg.description.clone(),
            command: cfg.command.clone(),
            working_dir: cfg.cwd.clone(),
            env: cfg.env.clone(),
            exec_paths: cfg.exec_paths.clone(),
            is_default: name == "default",
            script_id: script_id.to_string(),
            script_display_name: script_display_name.to_string(),
            watch: cfg.watch.clone(),
        };

        let node_id = graph.add_node(NodeKind::Task(task_data));

        if !cfg.concurrently.is_empty() {
            let node = &mut graph.nodes[node_id as usize];
            node.metadata.insert(
                "concurrently".to_string(),
                serde_json::to_string(&cfg.concurrently).unwrap_or_default(),
            );
            if let Some(ff) = cfg.concurrently_options.fail_fast {
                node.metadata
                    .insert("fail_fast".to_string(), ff.to_string());
            }
            if let Some(mc) = cfg.concurrently_options.max_concurrent_tasks {
                node.metadata
                    .insert("max_concurrent".to_string(), mc.to_string());
            }
        }

        if let Some(timeout_str) = &cfg.timeout {
            graph.nodes[node_id as usize]
                .metadata
                .insert("timeout".to_string(), timeout_str.clone());
        }

        node_id
    }

    fn register_task(
        &mut self,
        script_id: &str,
        task_name: &str,
        node_id: u64,
        graph: &mut Graph,
    ) -> Result<()> {
        let full_key = format!("{} {}", script_id, task_name);
        if graph.task_registry.contains_key(&full_key) {
            return Err(BodoError::PluginError(format!(
                "Duplicate task name: {}",
                task_name
            )));
        }
        self.name_to_id.insert(full_key.clone(), node_id);
        graph.task_registry.insert(full_key.clone(), node_id);

        if task_name == "default" {
            let script_key = script_id.to_string();
            graph.task_registry.entry(script_key).or_insert(node_id);
        }

        let task_key = task_name.to_string();
        graph.task_registry.entry(task_key).or_insert(node_id);

        Ok(())
    }

    fn resolve_dependency(
        &mut self,
        dep: &str,
        referencing_file: &Path,
        graph: &mut Graph,
    ) -> Result<&u64> {
        if let Some((script_path, fallback_name)) = self.parse_cross_file_ref(dep, referencing_file)
        {
            // Create empty global env for cross-file references
            let empty_global_env = HashMap::new();
            self.load_script(graph, &script_path, &fallback_name, &empty_global_env, &[])?;
        }

        if let Some(id) = graph.task_registry.get(dep) {
            return Ok(id);
        }

        let script_key = format!("{} {}", referencing_file.display(), dep);
        if let Some(id) = graph.task_registry.get(&script_key) {
            return Ok(id);
        }

        Err(BodoError::PluginError(format!(
            "Dependency not found: {}",
            dep
        )))
    }

    // Other methods unchanged...
}
