use crate::{
    errors::BodoError,
    graph::{CommandData, ConcurrentGroupData, Graph, NodeKind},
    plugin::Plugin,
    Result,
};
use serde_json::Value;
use std::any::Any;

pub struct ConcurrentPlugin;

impl ConcurrentPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ConcurrentPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for ConcurrentPlugin {
    fn name(&self) -> &'static str {
        "ConcurrentPlugin"
    }

    fn priority(&self) -> i32 {
        100
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn on_graph_build(&mut self, graph: &mut Graph) -> Result<()> {
        let mut nodes_to_process = Vec::new();
        for node in &graph.nodes {
            if let NodeKind::Task(_) = &node.kind {
                if let Some(concurrent_meta) = node.metadata.get("concurrently") {
                    let concur_deps: Vec<Value> =
                        serde_json::from_str(concurrent_meta).map_err(|e| {
                            BodoError::PluginError(format!("Invalid concurrency JSON: {}", e))
                        })?;
                    let fail_fast = node
                        .metadata
                        .get("fail_fast")
                        .and_then(|v| v.parse::<bool>().ok())
                        .unwrap_or(true);
                    let max_concurrent = node
                        .metadata
                        .get("max_concurrent")
                        .and_then(|v| v.parse::<usize>().ok());
                    nodes_to_process.push((node.id, concur_deps, fail_fast, max_concurrent));
                }
            }
        }
        for (parent_id, concur_deps, fail_fast, max_concurrent) in nodes_to_process {
            let mut child_ids = Vec::new();
            for dep in concur_deps {
                match dep {
                    Value::String(task_name) => {
                        if let Some(&dep_id) = graph.task_registry.get(&task_name) {
                            child_ids.push(dep_id);
                        } else {
                            return Err(BodoError::PluginError(format!(
                                "Concurrent task not found: {}",
                                task_name
                            )));
                        }
                    }
                    Value::Object(cmd) => {
                        if let Some(Value::String(task_name)) = cmd.get("task") {
                            if let Some(&dep_id) = graph.task_registry.get(task_name) {
                                child_ids.push(dep_id);
                            } else {
                                return Err(BodoError::PluginError(format!(
                                    "Concurrent task not found: {}",
                                    task_name
                                )));
                            }
                        } else if let Some(Value::String(command)) = cmd.get("command") {
                            let cmd_node_id = graph.add_node(NodeKind::Command(CommandData {
                                raw_command: command.clone(),
                                description: None,
                                working_dir: None,
                                watch: None,
                                env: std::collections::HashMap::new(),
                            }));
                            child_ids.push(cmd_node_id);
                        } else {
                            return Err(BodoError::PluginError(
                                "Invalid concurrency dependency format (must have task or command)"
                                    .to_string(),
                            ));
                        }
                    }
                    _ => {
                        return Err(BodoError::PluginError(
                            "Invalid concurrency dependency format".to_string(),
                        ));
                    }
                }
            }
            let group_node = NodeKind::ConcurrentGroup(ConcurrentGroupData {
                child_nodes: child_ids.clone(),
                fail_fast,
                max_concurrent,
                timeout_secs: None,
            });
            let group_id = graph.add_node(group_node);
            graph.add_edge(parent_id, group_id)?;
            for child_id in child_ids {
                graph.add_edge(group_id, child_id)?;
                let child_node = &mut graph.nodes[child_id as usize];
                child_node
                    .metadata
                    .insert("skip_main_pass".to_string(), "true".to_string());
            }
        }
        Ok(())
    }
}
