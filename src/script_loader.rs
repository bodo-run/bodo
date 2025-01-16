use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::{
    config::{ConcurrentItem, ScriptConfig, TaskConfig},
    errors::{BodoError, Result},
    graph::{CommandData, Graph, NodeKind, TaskData},
};

#[derive(Debug, Deserialize, Default, Clone, PartialEq)]
pub struct BodoConfig {
    pub scripts_dir: Option<String>,
    pub scripts_glob: Option<String>,
}

/// Type alias for script config insertion result
type ScriptInsertResult = (
    HashMap<String, u64>,
    Vec<(u64, Vec<ConcurrentItem>, String)>,
);

/// Parse an individual script file from YAML into a ScriptConfig.
fn parse_script_file(path: &PathBuf) -> Result<ScriptConfig> {
    let contents = fs::read_to_string(path)?;
    let script_config: ScriptConfig = serde_yaml::from_str(&contents)
        .map_err(|e| BodoError::PluginError(format!("YAML parse error in {:?}: {}", path, e)))?;
    Ok(script_config)
}

/// Insert the given TaskConfig into the Graph as a Task node.
/// Returns the NodeId of the created node.
fn insert_task_into_graph(
    graph: &mut Graph,
    task_name: &str,
    task_cfg: &TaskConfig,
) -> Result<u64> {
    // Store concurrency as JSON in metadata if present
    let concurrency_json = if let Some(items) = &task_cfg.concurrently {
        serde_json::to_string(items).unwrap_or_else(|_| "[]".to_string())
    } else {
        "[]".to_string()
    };

    let task_data = TaskData {
        name: task_name.to_string(),
        description: task_cfg.description.clone(),
        command: task_cfg.command.clone(),
        working_dir: task_cfg.cwd.clone(),
        is_default: false,
        script_name: Some("Root".to_string()),
    };

    let node_id = graph.add_node(NodeKind::Task(task_data));

    // Store metadata
    let node = &mut graph.nodes[node_id as usize];
    node.metadata
        .insert("concurrently".to_string(), concurrency_json);

    // Store env variables if present
    if let Some(env_map) = &task_cfg.env {
        if let Ok(env_json) = serde_json::to_string(env_map) {
            node.metadata.insert("env".to_string(), env_json);
        }
    }

    // Store output config if present
    if let Some(output_cfg) = &task_cfg.output {
        if let Ok(out_json) = serde_json::to_string(output_cfg) {
            node.metadata.insert("output".to_string(), out_json);
        }
    }

    Ok(node_id)
}

/// Check for cycles in the dependency graph.
fn has_cycle(
    graph: &Graph,
    start: u64,
    visited: &mut HashSet<u64>,
    path: &mut HashSet<u64>,
) -> bool {
    if !visited.insert(start) {
        return false;
    }
    path.insert(start);

    for edge in &graph.edges {
        if edge.from == start
            && (path.contains(&edge.to) || has_cycle(graph, edge.to, visited, path))
        {
            return true;
        }
    }

    path.remove(&start);
    false
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
                    if let Some(&id) = name_to_id.get(&qualified_name) {
                        id
                    } else {
                        return Err(BodoError::PluginError(format!(
                            "Dependency task '{}' not found",
                            task
                        )));
                    }
                };

                // Add the edge
                graph.add_edge(task_id, node_id).map_err(|e| {
                    BodoError::PluginError(format!(
                        "Failed to add edge from '{}' to '{}': {}",
                        task, node_id, e
                    ))
                })?;

                // Check for cycles
                let mut visited = HashSet::new();
                let mut path = HashSet::new();
                if has_cycle(graph, task_id, &mut visited, &mut path) {
                    return Err(BodoError::PluginError(format!(
                        "Circular dependency detected involving task '{}'",
                        task
                    )));
                }
            }
            ConcurrentItem::Command { command, .. } => {
                let cmd_data = CommandData {
                    raw_command: command.clone(),
                    description: None,
                    working_dir: None,
                    watch: None,
                };
                let cmd_node_id = graph.add_node(NodeKind::Command(cmd_data));
                graph.add_edge(cmd_node_id, node_id).map_err(|e| {
                    BodoError::PluginError(format!(
                        "Failed to add edge from command '{}' to '{}': {}",
                        command, node_id, e
                    ))
                })?;
            }
        }
    }
    Ok(())
}

/// Derive the script name from the parent folder name or mark as root
fn derive_script_name(file_path: &Path, graph: &Graph) -> Option<String> {
    // if this is the root script, name should be empty
    // root script is the first node in the graph
    if graph.nodes.len() == 1 {
        return None;
    }

    match file_path.parent() {
        Some(parent) if parent.file_name().is_some() => {
            Some(parent.file_name().unwrap().to_string_lossy().to_string())
        }
        _ => None, // means "no script name / root tasks"
    }
}

/// Insert a ScriptConfig into the graph and return a map of task names to node IDs.
fn insert_script_config_into_graph(
    graph: &mut Graph,
    script: ScriptConfig,
    file_label: &str,
    file_path: &Path,
) -> Result<ScriptInsertResult> {
    let mut name_to_id = HashMap::new();
    let mut dependencies = Vec::new();

    // Derive script name from YAML or parent folder
    let script_name = derive_script_name(file_path, graph);
    let script_name_str = script_name.as_deref().unwrap_or("Root");

    // Insert default task
    let default_task_name = format!("{}#default", file_label);
    let def_id = insert_task_into_graph_with_script_name(
        graph,
        &default_task_name,
        &script.default_task,
        script_name_str,
    )?;
    if let Some(deps) = &script.default_task.dependencies {
        dependencies.push((def_id, deps.clone(), file_label.to_string()));
    }
    name_to_id.insert(default_task_name, def_id);

    // Insert other tasks if present
    if let Some(tasks_map) = script.tasks {
        for (task_name, task_cfg) in tasks_map {
            let fully_qualified = format!("{}#{}", file_label, task_name);
            let node_id = insert_task_into_graph_with_script_name(
                graph,
                &fully_qualified,
                &task_cfg,
                script_name_str,
            )?;
            if let Some(deps) = &task_cfg.dependencies {
                dependencies.push((node_id, deps.clone(), file_label.to_string()));
            }
            name_to_id.insert(fully_qualified, node_id);
        }
    }

    Ok((name_to_id, dependencies))
}

/// A helper that sets the node's metadata with "script_name" too
fn insert_task_into_graph_with_script_name(
    graph: &mut Graph,
    task_name: &str,
    task_cfg: &TaskConfig,
    script_name: &str,
) -> Result<u64> {
    let node_id = insert_task_into_graph(graph, task_name, task_cfg)?;
    let node = &mut graph.nodes[node_id as usize];

    // Store the script name in metadata
    node.metadata
        .insert("script_name".to_string(), script_name.to_string());

    Ok(node_id)
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
            insert_script_config_into_graph(graph, script_config.clone(), &file_label, path)?;
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
