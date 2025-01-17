use async_trait::async_trait;
use serde_json::Value;
use std::any::Any;

use crate::{
    graph::{ConcurrentGroupData, Graph, NodeKind},
    plugin::Plugin,
    Result,
};

pub struct ConcurrencyPlugin;

#[async_trait]
impl Plugin for ConcurrencyPlugin {
    fn name(&self) -> &'static str {
        "ConcurrencyPlugin"
    }

    async fn on_graph_build(&mut self, graph: &mut Graph) -> Result<()> {
        let mut transformations = Vec::new();

        // Scan for tasks that have concurrency info in metadata
        for node in &graph.nodes {
            if let NodeKind::Task(_) = &node.kind {
                if let Some(conc_data) = node.metadata.get("concurrently") {
                    transformations.push((node.id, conc_data.clone()));
                }
            }
        }

        // Convert each concurrency metadata into a dedicated concurrency node
        for (task_id, conc_str) in transformations {
            // Remove metadata from the original task so we don't process it again
            let original_node = &mut graph.nodes[task_id as usize];
            original_node.metadata.remove("concurrently");

            // Parse concurrency settings
            let json_val: Value = serde_json::from_str(&conc_str)?;
            let child_ids = match json_val.get("children") {
                Some(Value::Array(arr)) => {
                    arr.iter().filter_map(|v| v.as_u64()).collect::<Vec<u64>>()
                }
                _ => vec![],
            };
            let fail_fast = match json_val.get("fail_fast") {
                Some(Value::Bool(b)) => *b,
                _ => false,
            };
            let max_concurrent = match json_val.get("max_concurrent") {
                Some(Value::Number(num)) => num.as_u64().map(|n| n as usize),
                _ => None,
            };
            let timeout_secs = match json_val.get("timeout_secs") {
                Some(Value::Number(num)) => num.as_u64(),
                _ => None,
            };

            // Create the concurrency node
            let group_data = ConcurrentGroupData {
                child_nodes: child_ids,
                fail_fast,
                max_concurrent,
                timeout_secs,
            };
            let group_id = graph.add_node(NodeKind::ConcurrentGroup(group_data));

            // Add an edge from the task to the concurrency node
            let _ = graph.add_edge(task_id, group_id);
        }

        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
