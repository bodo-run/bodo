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

        // Group tasks by script
        for node in &graph.nodes {
            if let NodeKind::Task(task_data) = &node.kind {
                let script_name = task_data
                    .script_name
                    .clone()
                    .unwrap_or_else(|| "Root Script".to_string());
                let entry = tasks_by_script.entry(script_name).or_default();
                entry.push((&task_data.name, task_data.description.clone()));
            }
        }

        // Print tasks grouped by script
        for (script_name, tasks) in tasks_by_script {
            // Print script header
            println!("\n{}", script_name);

            // Print tasks
            for (name, desc) in tasks {
                if name == "default_task" {
                    println!("  (default_task)   {}", desc.unwrap_or_default());
                } else {
                    let task_name = if script_name == "Root Script" {
                        name.to_string()
                    } else {
                        format!("{} {}", script_name.split('/').next().unwrap_or(""), name)
                    };

                    if let Some(desc) = desc {
                        println!("  {:<15} {}", task_name, desc);
                    } else {
                        println!("  {}", task_name);
                    }
                }
            }
        }
        println!();
        Ok(())
    }
}
