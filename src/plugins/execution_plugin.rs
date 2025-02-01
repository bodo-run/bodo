use crate::{
    // Remove unused import
    // errors::BodoError,
    graph::{Graph, NodeKind},
    plugin::Plugin,
    Result,
};
use async_trait::async_trait;
use std::any::Any;

pub struct ExecutionPlugin;

impl ExecutionPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ExecutionPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for ExecutionPlugin {
    fn name(&self) -> &'static str {
        "ExecutionPlugin"
    }

    fn priority(&self) -> i32 {
        50
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    async fn on_graph_build(&mut self, graph: &mut Graph) -> Result<()> {
        execute_graph(graph).await
    }
}

pub async fn execute_graph(graph: &mut Graph) -> Result<()> {
    for node in &graph.nodes {
        if let NodeKind::Task(task_data) = &node.kind {
            // TODO: Implement task execution
            let _ = task_data;
        }
    }
    Ok(())
}
