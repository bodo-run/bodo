use crate::{
    errors::BodoError,
    graph::{Graph, NodeKind},
    plugin::Plugin,
    Result,
};
use async_trait::async_trait;
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
        for node in &graph.nodes {
            if let NodeKind::Task(task_data) = &node.kind {
                // TODO: Implement concurrent task handling
                let _ = task_data;
            }
        }
        Ok(())
    }
}
