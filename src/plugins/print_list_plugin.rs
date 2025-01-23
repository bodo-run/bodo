use async_trait::async_trait;
use colored::Colorize;
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
        let mut tasks_by_script: HashMap<String, Vec<(&str, Option<String>, &str)>> =
            HashMap::new();

        // Group tasks by script display name
        for node in &graph.nodes {
            if let NodeKind::Task(task_data) = &node.kind {
                let entry = tasks_by_script
                    .entry(task_data.script_display_name.clone())
                    .or_default();
                entry.push((
                    &task_data.name,
                    task_data.description.clone(),
                    &task_data.script_id,
                ));
            }
        }

        // Print tasks grouped by script
        for (script_name, tasks) in tasks_by_script {
            println!("\n{}", script_name.bold().blue());

            // Print tasks
            for (name, desc, script_id) in tasks {
                let full_name = if name == "default" {
                    script_id.to_string()
                } else {
                    format!("{} {}", script_id, name)
                };

                match desc {
                    Some(desc) => println!("  {:<25} {}", full_name, desc),
                    None => println!("  {}", full_name),
                }
            }
        }
        println!();
        Ok(())
    }
}
