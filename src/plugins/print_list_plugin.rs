use async_trait::async_trait;
use std::{any::Any, collections::HashMap};

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
        let mut tasks_by_script: HashMap<String, Vec<(&str, Option<String>)>> = HashMap::new();

        // Group tasks by script display name
        for node in &graph.nodes {
            if let NodeKind::Task(task_data) = &node.kind {
                let entry = tasks_by_script
                    .entry(task_data.script_display_name.clone())
                    .or_default();
                entry.push((&task_data.name, task_data.description.clone()));
            }
        }

        // Print tasks grouped by script
        for (script_name, tasks) in tasks_by_script {
            println!("\n{}", script_name);

            // Print tasks
            for (name, desc) in tasks {
                if name == "default" {
                    if let Some(desc) = desc {
                        println!("  (default)   {}", desc);
                    } else {
                        println!("  (default)");
                    }
                } else if let Some(desc) = desc {
                    println!("  {:<15} {}", name, desc);
                } else {
                    println!("  {}", name);
                }
            }
        }
        println!();
        Ok(())
    }
}
