use crate::{
    errors::BodoError,
    graph::{CommandData, ConcurrentGroupData, Graph, NodeKind},
    plugin::Plugin,
    Result,
};
use async_trait::async_trait;
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

#[async_trait]
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

    async fn on_graph_build(&mut self, graph: &mut Graph) -> Result<()> {
        let mut nodes_to_process = Vec::new();

        // First pass: collect all nodes that have concurrency metadata
        for node in &graph.nodes {
            if let NodeKind::Task(_) = &node.kind {
                if let Some(concurrent_meta) = node.metadata.get("concurrently") {
                    // Parse the concurrency array from JSON
                    let concur_deps: Vec<Value> =
                        serde_json::from_str(concurrent_meta).map_err(|e| {
                            BodoError::PluginError(format!("Invalid concurrency JSON: {}", e))
                        })?;

                    // Get fail_fast and max_concurrent from metadata
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

        // Second pass: create concurrent group nodes and establish edges
        for (parent_id, concur_deps, fail_fast, max_concurrent) in nodes_to_process {
            let mut child_ids = Vec::new();

            // Process each dependency
            for dep in concur_deps {
                match dep {
                    Value::String(task_name) => {
                        // Look up task in registry
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
                        // Handle command objects
                        if let Some(Value::String(command)) = cmd.get("command") {
                            let cmd_node_id = graph.add_node(NodeKind::Command(CommandData {
                                raw_command: command.clone(),
                                description: None,
                                working_dir: None,
                                watch: None,
                                env: std::collections::HashMap::new(),
                            }));
                            child_ids.push(cmd_node_id);
                        }
                    }
                    _ => {
                        return Err(BodoError::PluginError(
                            "Invalid concurrency dependency format".to_string(),
                        ));
                    }
                }
            }

            // Create the concurrent group node
            let group_node = NodeKind::ConcurrentGroup(ConcurrentGroupData {
                child_nodes: child_ids.clone(),
                fail_fast,
                max_concurrent,
                timeout_secs: None,
            });
            let group_id = graph.add_node(group_node);

            // Add edges: parent -> group -> children
            graph.add_edge(parent_id, group_id)?;
            for child_id in child_ids {
                graph.add_edge(group_id, child_id)?;

                // Mark children so ExecutionPlugin won't re-run them individually outside the group
                let child_node = &mut graph.nodes[child_id as usize];
                child_node
                    .metadata
                    .insert("skip_main_pass".to_string(), "true".to_string());
            }
        }

        Ok(())
    }
}
