use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::{
    config::{BodoConfig, Dependency, TaskConfig},
    errors::{BodoError, Result},
    graph::{CommandData, Graph, NodeKind, TaskData},
};

/// Simplified ScriptLoader that handles loading task configurations from files.
pub struct ScriptLoader {
    // Track tasks across all files: "script_id task_name" -> node ID
    pub name_to_id: HashMap<String, u64>,
    pub task_registry: HashMap<String, u64>,

    // Track which scripts we've already loaded, plus a map from abs path to script name
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

    /// Load scripts and build the task graph
    pub async fn build_graph(&mut self, config: BodoConfig) -> Result<Graph> {
        let mut graph = Graph::new();
        let mut paths_to_load = vec![];
        let mut root_script_abs: Option<PathBuf> = None;

        // Load the root script if exists
        if let Some(ref root_script) = config.root_script {
            let root_path = PathBuf::from(root_script);
            if root_path.exists() {
                root_script_abs = Some(root_path.canonicalize()?);
                paths_to_load.push((root_path, "".to_string()));
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
                    // Skip the root script file if a match
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
                            // For these files, use the parent directory name as the display name
                            let script_display = path
                                .parent()
                                .and_then(|p| p.file_name())
                                .and_then(|n| n.to_str())
                                .unwrap_or("default")
                                .to_string();
                            paths_to_load.push((path, script_display));
                        } else if name.ends_with(".yaml") || name.ends_with(".yml") {
                            // For other YAML files, use the file stem as display name
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

        // Load each script file
        for (path, script_name) in paths_to_load {
            self.load_script(&mut graph, &path, &script_name)?;
        }

        Ok(graph)
    }

    fn load_script(&mut self, graph: &mut Graph, path: &Path, script_id: &str) -> Result<()> {
        // If we've already loaded this script (by its absolute path), skip
        let abs = path.canonicalize()?;
        if self.loaded_scripts.contains_key(&abs) {
            return Ok(());
        }

        // Mark it as loaded
        self.loaded_scripts
            .insert(abs.clone(), script_id.to_string());

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

        // Get script display name from yaml or fallback to script_id
        let yaml_name = yaml.get("name").and_then(|v| v.as_str()).unwrap_or("");
        // For the root script, we want an empty display name;
        // otherwise, we use the provided name.
        let script_display_name = if script_id == "scripts" {
            "".to_string()
        } else if yaml_name.is_empty() {
            script_id.to_string()
        } else {
            yaml_name.to_string()
        };

        // Load default task if present
        let mut task_ids = Vec::new();
        if let Some(default_task) = yaml.get("default_task") {
            let task_config = match serde_yaml::from_value(default_task.clone()) {
                Ok(config) => config,
                Err(e) => {
                    eprintln!("Warning: Invalid default task in {}: {}", path.display(), e);
                    return Ok(());
                }
            };
            let default_id = self.create_task_node(
                graph,
                script_id,
                &script_display_name,
                "default",
                &task_config,
            );
            self.register_task(script_id, "default", default_id, graph)?;
            task_ids.push((default_id, task_config));
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

        // Create nodes for each task in tasks map: use empty display name for root script tasks.
        for (name, task_config) in tasks_map {
            let task_display_name = if script_id == "scripts" {
                "".to_string()
            } else {
                script_display_name.clone()
            };
            let task_id =
                self.create_task_node(graph, script_id, &task_display_name, &name, &task_config);
            self.register_task(script_id, &name, task_id, graph)?;
            task_ids.push((task_id, task_config));
        }

        // Create edges for dependencies
        for (task_id, task_config) in task_ids {
            // Add pre-dependencies
            for dep in task_config.pre_deps {
                match dep {
                    Dependency::Task { task } => {
                        let dep_id = self.resolve_dependency(&task, path, graph)?;
                        graph.add_edge(dep_id, task_id)?;
                    }
                    Dependency::Command { command } => {
                        let cmd_node_id = graph.add_node(NodeKind::Command(CommandData {
                            raw_command: command,
                            description: None,
                            working_dir: None,
                            watch: None,
                            env: HashMap::new(),
                        }));
                        graph.add_edge(cmd_node_id, task_id)?;
                    }
                }
            }

            // Add post-dependencies
            for dep in task_config.post_deps {
                match dep {
                    Dependency::Task { task } => {
                        let dep_id = self.resolve_dependency(&task, path, graph)?;
                        graph.add_edge(task_id, dep_id)?;
                    }
                    Dependency::Command { command } => {
                        let cmd_node_id = graph.add_node(NodeKind::Command(CommandData {
                            raw_command: command,
                            description: None,
                            working_dir: None,
                            watch: None,
                            env: HashMap::new(),
                        }));
                        graph.add_edge(task_id, cmd_node_id)?;
                    }
                }
            }
        }

        Ok(())
    }

    fn create_task_node(
        &self,
        graph: &mut Graph,
        script_id: &str,
        script_display_name: &str,
        name: &str,
        cfg: &TaskConfig,
    ) -> u64 {
        // Create a normal task node
        let task_data = TaskData {
            name: name.to_string(),
            description: cfg.description.clone(),
            command: cfg.command.clone(),
            working_dir: cfg.cwd.clone(),
            env: cfg.env.clone(),
            is_default: name == "default",
            script_id: script_id.to_string(),
            script_display_name: script_display_name.to_string(),
            watch: None,
        };

        let node_id = graph.add_node(NodeKind::Task(task_data));

        // If there's a concurrently array, store it in metadata
        if !cfg.concurrently.is_empty() {
            let node = graph.nodes.get_mut(node_id as usize).unwrap();

            // Store concurrency array as JSON
            node.metadata.insert(
                "concurrently".to_string(),
                serde_json::to_string(&cfg.concurrently).unwrap_or_default(),
            );

            // Store concurrency options
            if let Some(ff) = cfg.concurrently_options.fail_fast {
                node.metadata
                    .insert("fail_fast".to_string(), ff.to_string());
            }
            if let Some(mc) = cfg.concurrently_options.max_concurrent_tasks {
                node.metadata
                    .insert("max_concurrent".to_string(), mc.to_string());
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
        // Register with full key
        let full_key = format!("{} {}", script_id, task_name);
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
            let script_key = script_id.to_string();
            graph.task_registry.entry(script_key).or_insert(node_id);
        }

        // Register task under its name if it doesn't conflict
        let task_key = task_name.to_string();
        graph.task_registry.entry(task_key).or_insert(node_id);

        Ok(())
    }

    /// Helper to parse a cross-file reference like "../other.yaml/build"
    /// and return (absolute_script_path, optional_task_name).
    /// If there's no `/build` part, we treat it as default.
    fn parse_cross_file_ref(
        &self,
        reference: &str,
        referencing_file: &Path,
    ) -> Option<(PathBuf, String)> {
        // If not .yaml or .yml, it's not cross-file
        if !(reference.contains(".yaml") || reference.contains(".yml")) {
            return None;
        }

        // We'll treat everything up to ".yaml" or ".yml" as the file,
        // and anything after the next slash as the task
        // e.g. "foo/bar.yaml/baz" => file="foo/bar.yaml", task="baz"
        // e.g. "../other.yml" => file="../other.yml", task=default
        let full_ref = PathBuf::from(reference);
        let referencing_dir = referencing_file.parent().unwrap_or_else(|| Path::new("."));

        // Attempt to split the path into (someFile.yaml) + (subtask)
        // Strategy: search from left to right for a .yaml or .yml extension
        let mut found_yaml = false;
        let mut path_part = PathBuf::new();
        let mut task_part = None;

        // We'll iterate components to find the file that has .yaml or .yml
        let mut components = full_ref.components().peekable();
        while let Some(comp) = components.next() {
            path_part.push(comp);
            if let Some(ext) = path_part.extension() {
                let ext_s = ext.to_string_lossy().to_lowercase();
                if ext_s == "yaml" || ext_s == "yml" {
                    // we found the script file portion
                    found_yaml = true;
                    break;
                }
            }
        }

        if !found_yaml {
            // There's .yaml in the string but we didn't parse it properly as extension?
            return None;
        }

        // If there are leftover components after the .yaml / .yml, that's the subtask
        let remaining: Vec<_> = components.collect();
        if !remaining.is_empty() {
            // subtask name is the join of all remaining components with "/"
            let joined = remaining
                .iter()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            task_part = Some(joined);
        } else {
            // no subtask => default
            task_part = Some("default".to_string());
        }

        // Now we form the absolute path to the .yaml file
        let abs_script = referencing_dir.join(path_part).canonicalize().ok()?;
        let subtask = task_part.unwrap_or_else(|| "default".to_string());

        Some((abs_script, subtask))
    }

    /// Modified: resolve_dependency checks if `dep` is cross-file.
    /// If so, we load the external script (if needed) and return the correct node ID.
    fn resolve_dependency(
        &mut self,
        dep: &str,
        referencing_file: &Path,
        graph: &mut Graph,
    ) -> Result<u64> {
        // 1) If there's a known local or global name, return it
        if let Some(&id) = graph.task_registry.get(dep) {
            return Ok(id);
        }

        // 2) Possibly it's a cross-file reference like "../other.yaml/build"
        if let Some((script_path, subtask_name)) = self.parse_cross_file_ref(dep, referencing_file)
        {
            // Load the other script if not loaded
            if !self.loaded_scripts.contains_key(&script_path) {
                // Create a fallback display name from file stem
                let fallback_name = script_path
                    .file_stem()
                    .map(|x| x.to_string_lossy().to_string())
                    .unwrap_or_else(|| "external".to_string());

                self.load_script(graph, &script_path, &fallback_name)?;
            }

            // Now we have a script_id for that file
            // get the "display name" we stored when first loaded
            let loaded_id = self.loaded_scripts.get(&script_path).ok_or_else(|| {
                BodoError::PluginError(format!(
                    "Script not found after load: {}",
                    script_path.display()
                ))
            })?;
            // We'll build the registry key: "displayName task" or just "displayName" if default
            let full_key = if subtask_name == "default" {
                loaded_id.to_string()
            } else {
                format!("{} {}", loaded_id, subtask_name)
            };

            if let Some(&id) = graph.task_registry.get(&full_key) {
                return Ok(id);
            }
            return Err(BodoError::PluginError(format!(
                "Dependency not found in loaded script: {} (task '{}')",
                script_path.display(),
                subtask_name
            )));
        }

        // 3) Last attempt: maybe "script_id task" style => already in registry?
        let script_key = format!("{} {}", referencing_file.display(), dep);
        if let Some(&id) = graph.task_registry.get(&script_key) {
            return Ok(id);
        }

        // 4) Fallback error
        Err(BodoError::PluginError(format!(
            "Dependency not found: {}",
            dep
        )))
    }
}
