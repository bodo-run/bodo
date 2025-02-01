use crate::{
    graph::{Graph, NodeKind},
    plugin::Plugin,
    Result,
};
use async_trait::async_trait;
use std::any::Any;

pub struct WatchPlugin {}

impl WatchPlugin {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Plugin for WatchPlugin {
    fn name(&self) -> &'static str {
        "WatchPlugin"
    }

    fn priority(&self) -> i32 {
        90 // Lower than ExecutionPlugin (95)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    async fn on_graph_build(&mut self, graph: &mut Graph) -> Result<()> {
        // Find tasks with watch configuration
        let watch_tasks: Vec<_> = graph
            .nodes
            .iter()
            .filter_map(|node| {
                if let NodeKind::Task(task) = &node.kind {
                    if task.watch.is_some() {
                        Some(task)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        // For now, simply print the tasks that would be watched
        for task in watch_tasks {
            println!(
                "Would watch task: {} with paths: {:?}",
                task.name, task.watch
            );
        }

        Ok(())
    }
}
