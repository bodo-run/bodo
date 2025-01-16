use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::{
    config::{BodoConfig, ConcurrentItem, ScriptConfig, TaskConfig},
    errors::{BodoError, Result},
    graph::{CommandData, Graph, NodeKind, ScriptFileData, TaskData},
};

/// Type alias for task name to node ID mapping
type TaskNameMap = HashMap<String, u64>;
/// Type alias for dependency information
type DependencyInfo = Vec<(u64, Vec<ConcurrentItem>, String)>;
/// Type alias for script config insertion result
type ScriptInsertResult = (TaskNameMap, DependencyInfo);

/// Parse an individual script file from YAML into a ScriptConfig.
fn parse_script_file(path: &PathBuf) -> Result<ScriptConfig> {
    let contents = fs::read_to_string(path)?;
    let script_config: ScriptConfig = serde_yaml::from_str(&contents)
        .map_err(|e| BodoError::PluginError(format!("YAML parse error in {:?}: {}", path, e)))?;
    Ok(script_config)
}

/// Insert a task into the graph and return its node ID
fn insert_task_into_graph(
    graph: &mut Graph,
    task_name: &str,
    task_cfg: &TaskConfig,
    script_name: Option<String>,
) -> Result<u64> {
    let task_data = TaskData {
        name: task_name.to_string(),
        description: task_cfg.description.clone(),
        command: task_cfg.command.clone(),
        working_dir: task_cfg.cwd.clone(),
        is_default: false,
        script_name,
    };

    let task_id = graph.add_node(NodeKind::Task(task_data));

    // Store concurrency as JSON in metadata if present
    if let Some(items) = &task_cfg.concurrently {
        let concurrency_json = serde_json::to_string(items).unwrap_or_else(|_| "[]".to_string());
        let node = &mut graph.nodes[task_id as usize];
        node.metadata
            .insert("concurrently".to_string(), concurrency_json);
    }

    // Store env variables if present
    if let Some(env_map) = &task_cfg.env {
        if let Ok(env_json) = serde_json::to_string(env_map) {
            let node = &mut graph.nodes[task_id as usize];
            node.metadata.insert("env".to_string(), env_json);
        }
    }

    // Store output config if present
    if let Some(output_cfg) = &task_cfg.output {
        if let Ok(out_json) = serde_json::to_string(output_cfg) {
            let node = &mut graph.nodes[task_id as usize];
            node.metadata.insert("output".to_string(), out_json);
        }
    }

    Ok(task_id)
}

/// Insert a command into the graph and return its node ID
fn insert_command_into_graph(graph: &mut Graph, command: &str) -> u64 {
    let cmd_data = CommandData {
        raw_command: command.to_string(),
        description: None,
        working_dir: None,
        watch: None,
    };
    graph.add_node(NodeKind::Command(cmd_data))
}

/// Insert edges based on dependencies from a node to its dependencies.
fn insert_dependency_edges(
    graph: &mut Graph,
    node_id: u64,
    dependencies: &[ConcurrentItem],
    name_to_id: &HashMap<String, u64>,
    current_file: &str,
) -> Result<()> {
    for item in dependencies {
        match item {
            ConcurrentItem::Task { task, .. } => {
                // Try fully qualified name first
                let task_id = if let Some(&id) = name_to_id.get(task) {
                    id
                } else {
                    // Try with current file prefix
                    let qualified_name = format!("{}#{}", current_file, task);
                    name_to_id.get(&qualified_name).copied().ok_or_else(|| {
                        BodoError::PluginError(format!("Dependency task '{}' not found", task))
                    })?
                };

                // Add the edge from this task to its dependency
                graph.add_edge(task_id, node_id).map_err(|e| {
                    BodoError::PluginError(format!(
                        "Failed to add edge from '{}' to '{}': {}",
                        task, node_id, e
                    ))
                })?;
            }
            ConcurrentItem::Command { command, .. } => {
                let cmd_id = insert_command_into_graph(graph, command);
                graph.add_edge(cmd_id, node_id).map_err(|e| {
                    BodoError::PluginError(format!(
                        "Failed to add edge from command to task: {}",
                        e
                    ))
                })?;
            }
        }
    }
    Ok(())
}

/// Insert a ScriptConfig into the graph and return a map of task names to node IDs.
fn insert_script_config_into_graph(
    graph: &mut Graph,
    script_name: &str,
    script_cfg: &ScriptConfig,
) -> Result<ScriptInsertResult> {
    let mut name_to_id = HashMap::new();
    let mut dependencies = Vec::new();

    // Create command node first
    let cmd_data = CommandData {
        raw_command: "script command".to_string(),
        description: None,
        working_dir: None,
        watch: None,
    };
    let cmd_id = graph.add_node(NodeKind::Command(cmd_data));

    // Insert default task
    let default_task_id = insert_task_into_graph(
        graph,
        "default",
        &script_cfg.default_task,
        Some(script_name.to_string()),
    )?;
    if let Some(deps) = &script_cfg.default_task.dependencies {
        dependencies.push((default_task_id, deps.clone(), script_name.to_string()));
    }
    name_to_id.insert("default".to_string(), default_task_id);

    // Insert other tasks
    if let Some(tasks) = &script_cfg.tasks {
        for (task_name, task_cfg) in tasks {
            let task_id =
                insert_task_into_graph(graph, task_name, task_cfg, Some(script_name.to_string()))?;
            if let Some(deps) = &task_cfg.dependencies {
                dependencies.push((task_id, deps.clone(), script_name.to_string()));
            }
            name_to_id.insert(task_name.clone(), task_id);
        }
    }

    // Create script file node last
    let script_data = ScriptFileData {
        name: script_name.to_string(),
        description: script_cfg.description.clone(),
        tasks: Vec::new(),
        default_task: None,
    };
    let script_id = graph.add_node(NodeKind::ScriptFile(script_data));

    // Add edges from script to tasks
    graph.add_edge(script_id, default_task_id).map_err(|e| {
        BodoError::PluginError(format!(
            "Failed to add edge from script to default task: {}",
            e
        ))
    })?;

    Ok((name_to_id, dependencies))
}

/// Load scripts from the given paths into the graph.
pub fn load_scripts(paths: &[PathBuf], graph: &mut Graph) -> Result<()> {
    let mut global_name_to_id = HashMap::new();
    let mut all_dependencies = Vec::new();

    // First pass: Parse files and insert tasks
    for path in paths {
        if !path.exists() {
            continue;
        }

        let script_config = match parse_script_file(path) {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("Warning: could not parse {:?}: {}", path, e);
                continue;
            }
        };

        let file_label = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let (local_map, deps) =
            insert_script_config_into_graph(graph, &file_label, &script_config)?;
        global_name_to_id.extend(local_map);
        all_dependencies.extend(deps);
    }

    // Second pass: Add dependency edges
    for (node_id, deps, file_label) in all_dependencies {
        insert_dependency_edges(graph, node_id, &deps, &global_name_to_id, &file_label)?;
    }

    Ok(())
}

/// Load BodoConfig from a file path or return default.
pub fn load_bodo_config(config_path: Option<&str>) -> Result<BodoConfig> {
    if let Some(path_str) = config_path {
        let path = PathBuf::from(path_str);
        if path.exists() {
            let contents = fs::read_to_string(&path)?;
            let config = serde_yaml::from_str::<BodoConfig>(&contents)
                .or_else(|_| serde_json::from_str::<BodoConfig>(&contents))
                .unwrap_or_default();
            return Ok(config);
        }
    }
    Ok(BodoConfig::default())
}
