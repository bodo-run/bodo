use async_trait::async_trait;
use std::any::Any;

use crate::{
    errors::BodoError,
    graph::{Graph, NodeKind},
    plugin::Plugin,
    Result,
};

pub struct FakeFailingPlugin;

impl FakeFailingPlugin {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Plugin for FakeFailingPlugin {
    fn name(&self) -> &'static str {
        "FakeFailingPlugin"
    }

    fn priority(&self) -> i32 {
        0
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    async fn on_graph_build(&mut self, graph: &mut Graph) -> Result<()> {
        for node in &graph.nodes {
            if let NodeKind::Task(task_data) = &node.kind {
                // Always fail for testing purposes
                return Err(BodoError::PluginError(format!(
                    "Fake failure for task: {}",
                    task_data.name
                )));
            }
        }
        Ok(())
    }
}
