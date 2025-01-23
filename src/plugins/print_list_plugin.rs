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

    fn priority(&self) -> i32 {
        0
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    async fn on_graph_build(&mut self, graph: &mut Graph) -> Result<()> {
        println!("\nAvailable tasks:");
        for node in &graph.nodes {
            if let NodeKind::Task(task_data) = &node.kind {
                let name = if let Some(ref script_name) = task_data.script_name {
                    format!("{}#{}", script_name, task_data.name)
                } else {
                    task_data.name.clone()
                };

                if let Some(ref desc) = task_data.description {
                    println!("  {} - {}", name, desc);
                } else {
                    println!("  {}", name);
                }
            }
        }
        println!();
        Ok(())
    }
}
