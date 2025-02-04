use crate::config::validate_task_name;
use crate::errors::BodoError;
use crate::graph::{Graph, NodeKind, TaskData};
use crate::{BodoConfig, Result};
use std::collections::HashMap;
use std::fs;

pub struct ScriptLoader;

impl Default for ScriptLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptLoader {
    pub fn new() -> Self {
        ScriptLoader
    }

    pub fn build_graph(&mut self, config: BodoConfig) -> Result<Graph> {
        let mut graph = Graph::new();
        // If a root_script is specified, load tasks from that file.
        if let Some(root_script) = config.root_script {
            let content = fs::read_to_string(&root_script)?;
            let parsed: BodoConfig = serde_yaml::from_str(&content)?;
            for (task_name, task_config) in parsed.tasks {
                // Validate task name
                validate_task_name(&task_name)
                    .map_err(|e| BodoError::ValidationError(e.to_string()))?;
                let full_key = format!("{} {}", root_script, task_name);
                if graph.task_registry.contains_key(&full_key) {
                    return Err(BodoError::ValidationError(format!(
                        "duplicate task: {}",
                        task_name
                    )));
                }
                let task_data = TaskData {
                    name: task_name.clone(),
                    description: task_config.description,
                    command: task_config.command,
                    working_dir: task_config.cwd,
                    env: task_config.env,
                    exec_paths: task_config.exec_paths,
                    arguments: task_config.arguments,
                    is_default: false,
                    script_id: root_script.clone(),
                    script_display_name: root_script.clone(),
                    watch: task_config.watch,
                    pre_deps: task_config.pre_deps,
                    post_deps: task_config.post_deps,
                    concurrently: task_config.concurrently,
                    concurrently_options: task_config.concurrently_options,
                };
                let node_id = graph.add_node(NodeKind::Task(task_data));
                graph.task_registry.insert(full_key, node_id);
            }
            if let Some(default_task) = parsed.default_task {
                let task_data = TaskData {
                    name: "default".to_string(),
                    description: default_task.description,
                    command: default_task.command,
                    working_dir: default_task.cwd,
                    env: default_task.env,
                    exec_paths: default_task.exec_paths,
                    arguments: default_task.arguments,
                    is_default: true,
                    script_id: root_script.clone(),
                    script_display_name: root_script.clone(),
                    watch: default_task.watch,
                    pre_deps: default_task.pre_deps,
                    post_deps: default_task.post_deps,
                    concurrently: default_task.concurrently,
                    concurrently_options: default_task.concurrently_options,
                };
                let node_id = graph.add_node(NodeKind::Task(task_data));
                graph.task_registry.insert("default".to_string(), node_id);
            }
        } else {
            // Process tasks directly from the configuration if no root_script is given.
            for (task_name, task_config) in config.tasks {
                validate_task_name(&task_name)
                    .map_err(|e| BodoError::ValidationError(e.to_string()))?;
                if graph.task_registry.contains_key(&task_name) {
                    return Err(BodoError::ValidationError(format!(
                        "duplicate task: {}",
                        task_name
                    )));
                }
                let task_data = TaskData {
                    name: task_name.clone(),
                    description: task_config.description,
                    command: task_config.command,
                    working_dir: task_config.cwd,
                    env: task_config.env,
                    exec_paths: task_config.exec_paths,
                    arguments: task_config.arguments,
                    is_default: false,
                    script_id: "".to_string(),
                    script_display_name: "".to_string(),
                    watch: task_config.watch,
                    pre_deps: task_config.pre_deps,
                    post_deps: task_config.post_deps,
                    concurrently: task_config.concurrently,
                    concurrently_options: task_config.concurrently_options,
                };
                let node_id = graph.add_node(NodeKind::Task(task_data));
                graph.task_registry.insert(task_name, node_id);
            }
            if let Some(default_task) = config.default_task {
                let task_data = TaskData {
                    name: "default".to_string(),
                    description: default_task.description,
                    command: default_task.command,
                    working_dir: default_task.cwd,
                    env: default_task.env,
                    exec_paths: default_task.exec_paths,
                    arguments: default_task.arguments,
                    is_default: true,
                    script_id: "".to_string(),
                    script_display_name: "".to_string(),
                    watch: default_task.watch,
                    pre_deps: default_task.pre_deps,
                    post_deps: default_task.post_deps,
                    concurrently: default_task.concurrently,
                    concurrently_options: default_task.concurrently_options,
                };
                let node_id = graph.add_node(NodeKind::Task(task_data));
                graph.task_registry.insert("default".to_string(), node_id);
            }
        }
        Ok(graph)
    }

    // This function is intended for testing purposes.
    pub fn merge_envs(
        global: &HashMap<String, String>,
        script: &HashMap<String, String>,
        task: &HashMap<String, String>,
    ) -> HashMap<String, String> {
        let mut merged = global.clone();
        for (k, v) in script {
            merged.insert(k.clone(), v.clone());
        }
        for (k, v) in task {
            merged.insert(k.clone(), v.clone());
        }
        merged
    }

    pub fn merge_exec_paths(
        global: &Vec<String>,
        script: &Vec<String>,
        task: &Vec<String>,
    ) -> Vec<String> {
        let mut seen = std::collections::HashSet::new();
        let mut result = Vec::new();
        for path in global.iter().chain(script).chain(task) {
            if seen.insert(path.clone()) {
                result.push(path.clone());
            }
        }
        result
    }
}
