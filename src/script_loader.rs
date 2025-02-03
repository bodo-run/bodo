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

    fn merge_envs(
        global_env: &HashMap<String, String>,
        script_env: &HashMap<String, String>,
        task_env: &HashMap<String, String>,
    ) -> HashMap<String, String> {
        let mut merged = HashMap::new();
        for (k, v) in global_env {
            merged.insert(k.clone(), v.clone());
        }
        for (k, v) in script_env {
            merged.insert(k.clone(), v.clone());
        }
        for (k, v) in task_env {
            merged.insert(k.clone(), v.clone());
        }
        merged
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

        // Load tasks from BodoConfig.tasks
        if !config.tasks.is_empty() {
            let script_display_name = "".to_string();
            let script_id = "config".to_string();

            for (task_name, task_config) in &config.tasks {
                self.validate_task_config(task_config, task_name, Path::new("config"))?;

                let task_config = {
                    let mut tc = task_config.clone();
                    tc.env = Self::merge_envs(&global_env, &HashMap::new(), &tc.env);
                    tc.exec_paths =
                        Self::merge_exec_paths(&global_exec_paths, &Vec::new(), &tc.exec_paths);
                    tc
                };

                let task_id = self.create_task_node(
                    &mut graph,
                    &script_id,
                    &script_display_name,
                    task_name,
                    &task_config,
                );
                self.register_task(&script_id, task_name, task_id, &mut graph)?;

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
                            graph.add_edge(task_id, dep_id)?;
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
        let mut task = task_config.clone();
        task._name_check = Some(task_name.to_string());

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

    fn load_script(
        &mut self,
        graph: &mut Graph,
        path: &Path,
        script_id: &str,
        global_env: &HashMap<String, String>,
        global_exec_paths: &[String],
    ) -> Result<()> {
        let abs = path.canonicalize()?;
        if self.loaded_scripts.contains_key(&abs) {
            return Ok(());
        }

        self.loaded_scripts
            .insert(abs.clone(), script_id.to_string());

        let contents = fs::read_to_string(path)?;
        let yaml: serde_yaml::Value = serde_yaml::from_str(&contents)?;

        let script_env = if let Some(env_val) = yaml.get("env") {
            if let Some(map) = env_val.as_mapping() {
                let mut senv = HashMap::new();
                for (k, v) in map {
                    if let (Some(k_str), Some(v_str)) = (k.as_str(), v.as_str()) {
                        senv.insert(k_str.to_string(), v_str.to_string());
                    }
                }
                senv
            } else {
                HashMap::new()
            }
        } else {
            HashMap::new()
        };

        let script_exec_paths = if let Some(paths_val) = yaml.get("exec_paths") {
            if let Some(arr) = paths_val.as_sequence() {
                arr.iter()
                    .filter_map(|x| x.as_str().map(String::from))
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        let yaml_name = yaml.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let script_display_name = if script_id == "scripts" {
            "".to_string()
        } else if yaml_name.is_empty() {
            script_id.to_string()
        } else {
            yaml_name.to_string()
        };

        let mut task_ids = Vec::new();
        if let Some(default_task) = yaml.get("default_task") {
            let mut task_config: TaskConfig =
                serde_yaml::from_value::<TaskConfig>(default_task.clone())?;

            task_config.env = Self::merge_envs(global_env, &script_env, &task_config.env);
            task_config.exec_paths = Self::merge_exec_paths(
                global_exec_paths,
                &script_exec_paths,
                &task_config.exec_paths,
            );

            self.validate_task_config(&task_config, "default", path)?;

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

        if let Some(tasks) = yaml.get("tasks").and_then(|v| v.as_mapping()) {
            for (k, v) in tasks {
                let name = k.as_str().unwrap_or_default().to_string();
                let mut task_config: TaskConfig = serde_yaml::from_value::<TaskConfig>(v.clone())?;

                task_config.env = Self::merge_envs(global_env, &script_env, &task_config.env);
                task_config.exec_paths = Self::merge_exec_paths(
                    global_exec_paths,
                    &script_exec_paths,
                    &task_config.exec_paths,
                );

                self.validate_task_config(&task_config, &name, path)?;

                let task_id = self.create_task_node(
                    graph,
                    script_id,
                    &script_display_name,
                    &name,
                    &task_config,
                );
                self.register_task(script_id, &name, task_id, graph)?;
                task_ids.push((task_id, task_config));
            }
        }

        for (task_id, task_config) in task_ids {
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
                            env: std::collections::HashMap::new(),
                            watch: None,
                        }));
                        graph.add_edge(cmd_node_id, task_id)?;
                    }
                }
            }
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
                            env: std::collections::HashMap::new(),
                            watch: None,
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
        graph.task_registry.insert(full_key, node_id);

        if task_name == "default" {
            let script_key = script_id.to_string();
            graph.task_registry.entry(script_key).or_insert(node_id);
        }

        let task_key = task_name.to_string();
        graph.task_registry.entry(task_key).or_insert(node_id);

        Ok(())
    }

    fn parse_cross_file_ref(
        &self,
        reference: &str,
        referencing_file: &Path,
    ) -> Option<(PathBuf, String)> {
        if !(reference.contains(".yaml") || reference.contains(".yml")) {
            return None;
        }

        let full_ref = PathBuf::from(reference);
        let referencing_dir = referencing_file.parent().unwrap_or_else(|| Path::new("."));

        let mut found_yaml = false;
        let mut path_part = PathBuf::new();

        let mut components = full_ref.components().peekable();
        while let Some(comp) = components.by_ref().next() {
            path_part.push(comp);
            if let Some(ext) = path_part.extension() {
                let ext_s = ext.to_string_lossy().to_lowercase();
                if ext_s == "yaml" || ext_s == "yml" {
                    found_yaml = true;
                    break;
                }
            }
        }

        if !found_yaml {
            return None;
        }

        let remaining: Vec<_> = components.collect();
        let task_part = if !remaining.is_empty() {
            Some(
                remaining
                    .iter()
                    .map(|c| c.as_os_str().to_string_lossy())
                    .collect::<Vec<_>>()
                    .join("/"),
            )
        } else {
            Some("default".to_string())
        };

        let abs_script = referencing_dir.join(path_part).canonicalize().ok()?;
        let subtask = task_part.unwrap_or_else(|| "default".to_string());

        Some((abs_script, subtask))
    }

    fn resolve_dependency(
        &mut self,
        dep: &str,
        referencing_file: &Path,
        graph: &mut Graph,
    ) -> Result<u64> {
        if let Some((script_path, fallback_name)) = self.parse_cross_file_ref(dep, referencing_file)
        {
            let empty_global_env = HashMap::new();
            self.load_script(graph, &script_path, &fallback_name, &empty_global_env, &[])?;
        }

        if let Some(&id) = graph.task_registry.get(dep) {
            return Ok(id);
        }

        let script_key = format!("{} {}", referencing_file.display(), dep);
        if let Some(&id) = graph.task_registry.get(&script_key) {
            return Ok(id);
        }

        Err(BodoError::PluginError(format!(
            "Dependency not found: {}",
            dep
        )))
    }
}
