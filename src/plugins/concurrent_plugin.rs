use crate::{
    errors::BodoError,
    graph::{ConcurrentGroupData, Graph, NodeId, NodeKind},
    plugin::Plugin,
    Result,
};
use async_trait::async_trait;
use serde_json::from_str;
use std::any::Any;

pub struct ConcurrentPlugin;

impl ConcurrentPlugin {
    pub fn new() -> Self {
        Self
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
        let mut concurrent_groups = Vec::new();

        // First pass: collect all concurrent task groups
        for node in &graph.nodes {
            if let NodeKind::Task(_) = &node.kind {
                if let Some(concurrent_meta) = node.metadata.get("concurrently") {
                    let meta: serde_json::Value =
                        serde_json::from_str(concurrent_meta).map_err(|e| {
                            BodoError::PluginError(format!("Invalid concurrent metadata: {}", e))
                        })?;

                    let children: Vec<NodeId> = meta["children"]
                        .as_array()
                        .ok_or_else(|| {
                            BodoError::PluginError(
                                "Missing children in concurrent metadata".to_string(),
                            )
                        })?
                        .iter()
                        .map(|v| v.as_u64().unwrap() as NodeId)
                        .collect();

                    let fail_fast = meta["fail_fast"].as_bool().unwrap_or(true);

                    let max_concurrent = meta["max_concurrent"].as_u64().map(|v| v as usize);

                    concurrent_groups.push((node.id, children.clone(), fail_fast, max_concurrent));
                }
            }
        }

        // Second pass: create concurrent group nodes and edges
        for (parent_id, children, fail_fast, max_concurrent) in concurrent_groups {
            // Create concurrent group node
            let group_node = NodeKind::ConcurrentGroup(ConcurrentGroupData {
                fail_fast,
                max_concurrent,
                child_nodes: children.clone(),
                timeout_secs: None,
            });
            let group_id = graph.add_node(group_node);

            // Add edge from parent to group
            graph.add_edge(parent_id, group_id);

            // Add edges from group to children
            for child_id in children {
                graph.add_edge(group_id, child_id);
            }
        }

        Ok(())
    }
}
