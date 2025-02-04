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

pub struct ScriptLoader {
    pub name_to_id: HashMap<String, u64>,
    pub task_registry: HashMap<String, u64>,
    // Removed `loaded_scripts` as it was unused
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

    // Helper function for merging environment variables
    pub fn merge_envs(
        global_env: &HashMap<String, String>,
        script_env: &HashMap<String, String>,
        task_env: &HashMap<String, String>,
    ) -> HashMap<String, String> {
        let mut result = global_env.clone();
        result.extend(script_env.clone());
        result.extend(task_env.clone());
        result
    }

    pub fn merge_exec_paths(
        global_paths: &[String],
        script_paths: &[String],
        task_paths: &[String],
    ) -> Vec<String> {
        let mut merged = Vec::new();
        for p in global_paths.iter().chain(script_paths).chain(task_paths) {
            if !merged.contains(p) {
                merged.push(p.clone());
            }
        }
        merged
    }

    pub fn build_graph(&mut self, config: BodoConfig) -> Result<Graph> {
        config.validate().map_err(BodoError::from)?;

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

        let mut graph = Graph::new();

        for (path, script_display) in paths_to_load {
            self.load_script(
                &mut graph,
                &path,
                &if Some(&path) == root_script_abs.as_ref() {
                    "".to_string()
                } else {
                    path.display().to_string()
                },
                &script_display,
                &global_env,
                &global_exec_paths,
            )?;
        }

        // Process default_task and tasks from config only if root_script is not provided
        if config.root_script.is_none() {
            // Process default_task if present
            if let Some(ref default_task_config) = config.default_task {
                // Use the passed in default values; for config tasks, use the key "default"
                let script_id = "".to_string();
                let script_display_name = "".to_string();
                let script_env = HashMap::new();
                let script_exec_paths = vec![];

                // Merge environments and exec_paths for default_task
                let env = Self::merge_envs(&global_env, &script_env, &default_task_config.env);
                let exec_paths = Self::merge_exec_paths(
                    &global_exec_paths,
                    &script_exec_paths,
                    &default_task_config.exec_paths,
                );

                // Validate task config
                self.validate_task_config(default_task_config, "default", Path::new("config"))?;

                let node_id = self.create_task_node(
                    &mut graph,
                    &script_id,
                    &script_display_name,
                    "default",
                    default_task_config,
                );

                // Update the task data with merged env and exec_paths
                if let NodeKind::Task(ref mut task_data) = graph.nodes[node_id as usize].kind {
                    task_data.env = env;
                    task_data.exec_paths = exec_paths;
                }

                self.register_task(&script_id, "default", node_id, &mut graph)?;

                // Handle dependencies after all tasks are registered
                // (For default_task, dependencies processing could be added if needed)
            }

            // Process tasks from config.tasks
            if !config.tasks.is_empty() {
                // Use the passed in script_id and script_display_name instead of empty strings.
                let script_id = "".to_string();
                let script_display_name = "".to_string();
                let script_env = HashMap::new();
                let script_exec_paths = vec![];

                let mut task_node_ids = HashMap::new();

                // First, create and register all tasks
                for (name, task_config) in &config.tasks {
                    // Merge environments and exec_paths for each task
                    let env = Self::merge_envs(&global_env, &script_env, &task_config.env);
                    let exec_paths = Self::merge_exec_paths(
                        &global_exec_paths,
                        &script_exec_paths,
                        &task_config.exec_paths,
                    );

                    // Validate task config
                    self.validate_task_config(task_config, name, Path::new("config"))?;

                    // Create task node
                    let task_id = self.create_task_node(
                        &mut graph,
                        &script_id,
                        &script_display_name,
                        name,
                        task_config,
                    );

                    // Update the task data with merged env and exec_paths
                    if let NodeKind::Task(ref mut task_data) = graph.nodes[task_id as usize].kind {
                        task_data.env = env;
                        task_data.exec_paths = exec_paths;
                    }

                    self.register_task(&script_id, name, task_id, &mut graph)?;
                    task_node_ids.insert(name.clone(), task_id);
                }

                // Now, process dependencies
                for (name, task_config) in &config.tasks {
                    let node_id = *task_node_ids.get(name).unwrap();

                    // Handle dependencies
                    for dep in &task_config.pre_deps {
                        match dep {
                            Dependency::Task { task } => {
                                let dep_id = self.resolve_dependency(task, &script_id, &graph)?;
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
                                let dep_id = self.resolve_dependency(task, &script_id, &graph)?;
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
            }
        }

        Ok(graph)
    }

    pub fn load_script(
        &mut self,
        graph: &mut Graph,
        path: &Path,
        script_id: &str,
        script_display_name: &str,
        global_env: &HashMap<String, String>,
        global_exec_paths: &Vec<String>,
    ) -> Result<()> {
        // Read the script file
        let content = fs::read_to_string(path)?;

        // Parse the content into a serde_yaml::Value to extract task names
        let script_yaml: serde_yaml::Value = serde_yaml::from_str(&content)?;

        // Now, collect task names and check for duplicates
        let mut task_names = std::collections::HashSet::new();

        if let Some(tasks) = script_yaml.get("tasks") {
            if let Some(task_map) = tasks.as_mapping() {
                for (key, _value) in task_map {
                    if let Some(task_name) = key.as_str() {
                        if !task_names.insert(task_name.to_string()) {
                            return Err(BodoError::PluginError(format!(
                                "Duplicate task name '{}' found in '{}'",
                                task_name,
                                path.display()
                            )));
                        }
                    }
                }
            }
        }

        // Then, parse the content into BodoConfig
        let script_config: BodoConfig = serde_yaml::from_str(&content)?;

        let script_env = script_config.env.clone();
        let script_exec_paths = script_config.exec_paths.clone();

        // Process default_task if present
        if let Some(ref default_task_config) = script_config.default_task {
            // Use the passed in script_id and script_display_name instead of empty strings.
            let curr_script_id = script_id.to_string();
            let curr_script_display = script_display_name.to_string();

            self.validate_task_config(default_task_config, "default", path)?;

            // Merge environments and exec_paths for default_task
            let env = Self::merge_envs(global_env, &script_env, &default_task_config.env);
            let exec_paths = Self::merge_exec_paths(
                global_exec_paths,
                &script_exec_paths,
                &default_task_config.exec_paths,
            );

            // Create and register the default task node
            let node_id = self.create_task_node(
                graph,
                &curr_script_id,
                &curr_script_display,
                "default",
                default_task_config,
            );

            // Update the task data with merged env and exec_paths
            if let NodeKind::Task(ref mut task_data) = graph.nodes[node_id as usize].kind {
                task_data.env = env;
                task_data.exec_paths = exec_paths;
                task_data.is_default = true;
            }

            self.register_task(&curr_script_id, "default", node_id, graph)?;

            // Handle dependencies
            for dep in &default_task_config.pre_deps {
                match dep {
                    Dependency::Task { task } => {
                        let dep_id = self.resolve_dependency(task, &curr_script_id, graph)?;
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
                        let dep_id = self.resolve_dependency(task, &curr_script_id, graph)?;
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

        // Now, process tasks
        if !script_config.tasks.is_empty() {
            let curr_script_id = script_id.to_string();
            let curr_script_display = script_display_name.to_string();
            let script_env = HashMap::new();
            let script_exec_paths = vec![];

            let mut task_node_ids = HashMap::new();

            // First, create and register all tasks
            for (name, task_config) in &script_config.tasks {
                // Merge environments and exec_paths for each task
                let env = Self::merge_envs(global_env, &script_env, &task_config.env);
                let exec_paths = Self::merge_exec_paths(
                    global_exec_paths,
                    &script_exec_paths,
                    &task_config.exec_paths,
                );

                // Validate task config
                self.validate_task_config(task_config, name, path)?;

                // Create task node
                let task_id = self.create_task_node(
                    graph,
                    &curr_script_id,
                    &curr_script_display,
                    name,
                    task_config,
                );

                // Update the task data with merged env and exec_paths
                if let NodeKind::Task(ref mut task_data) = graph.nodes[task_id as usize].kind {
                    task_data.env = env;
                    task_data.exec_paths = exec_paths;
                }

                self.register_task(&curr_script_id, name, task_id, graph)?;
                task_node_ids.insert(name.clone(), task_id);
            }

            // Now, process dependencies
            for (name, task_config) in &script_config.tasks {
                let node_id = *task_node_ids.get(name).unwrap();

                // Handle dependencies
                for dep in &task_config.pre_deps {
                    match dep {
                        Dependency::Task { task } => {
                            let dep_id = self.resolve_dependency(task, &curr_script_id, graph)?;
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
                            let dep_id = self.resolve_dependency(task, &curr_script_id, graph)?;
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
        }

        Ok(())
    }

    fn validate_task_config(
        &self,
        task_config: &TaskConfig,
        task_name: &str,
        _path: &Path,
    ) -> Result<()> {
        let mut task_config = task_config.clone();
        task_config._name_check = Some(task_name.to_string());
        task_config.validate().map_err(BodoError::from)
    }

    fn create_task_node(
        &mut self,
        graph: &mut Graph,
        script_id: &str,
        script_display_name: &str,
        task_name: &str,
        task_config: &TaskConfig,
    ) -> u64 {
        let task_data = TaskData {
            name: task_name.to_string(),
            description: task_config.description.clone(),
            command: task_config.command.clone(),
            working_dir: task_config.cwd.clone(),
            env: task_config.env.clone(),
            exec_paths: task_config.exec_paths.clone(),
            is_default: false,
            script_id: script_id.to_string(),
            script_display_name: script_display_name.to_string(),
            watch: task_config.watch.clone(),
            arguments: task_config.arguments.clone(),
        };
        let node_id = graph.add_node(NodeKind::Task(task_data));
        if !task_config.concurrently.is_empty() {
            if let Ok(json_string) = serde_json::to_string(&task_config.concurrently) {
                graph.nodes[node_id as usize]
                    .metadata
                    .insert("concurrently".to_string(), json_string);
            }
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
        let full_task_name = if script_id.is_empty() {
            task_name.to_string()
        } else {
            format!("{} {}", script_id, task_name)
        };
        if graph.task_registry.contains_key(&full_task_name) {
            return Err(BodoError::PluginError(format!(
                "Duplicate task name '{}' found in '{}'",
                full_task_name, script_id
            )));
        }
        graph.task_registry.insert(full_task_name, node_id);
        Ok(())
    }

    fn resolve_dependency(&self, task: &str, script_id: &str, graph: &Graph) -> Result<u64> {
        let full_task_name = if task.contains(' ') || script_id.is_empty() {
            task.to_string()
        } else {
            format!("{} {}", script_id, task)
        };

        if let Some(&node_id) = graph.task_registry.get(&full_task_name) {
            Ok(node_id)
        } else {
            // Fallback: search for any task whose key ends with " " + task or equals task
            for (key, &id) in &graph.task_registry {
                if key == task || key.ends_with(&format!(" {}", task)) {
                    return Ok(id);
                }
            }
            Err(BodoError::PluginError(format!(
                "Task '{}' not found when resolving dependency",
                task
            )))
        }
    }
}
