use async_trait::async_trait;
use std::any::Any;

use crate::{
    graph::{Graph, NodeKind},
    plugin::Plugin,
    Result,
};

pub struct PrintListPlugin;

#[async_trait]
impl Plugin for PrintListPlugin {
    fn name(&self) -> &'static str {
        "PrintListPlugin"
    }

    async fn on_graph_build(&mut self, graph: &mut Graph) -> Result<()> {
        println!("\nAvailable tasks:");
        for node in &graph.nodes {
            if let NodeKind::Task(task_data) = &node.kind {
                let script_info = task_data
                    .script_name
                    .as_ref()
                    .map(|s| format!(" (from {})", s))
                    .unwrap_or_default();

                let desc = task_data
                    .description
                    .as_ref()
                    .map(|s| format!(" - {}", s))
                    .unwrap_or_default();

                println!("  {}{}{}", task_data.name, script_info, desc);
            }
        }
        println!();
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
