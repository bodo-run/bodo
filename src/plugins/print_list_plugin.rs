use async_trait::async_trait;
use colored::Colorize;
use std::{any::Any, cmp::Ordering, collections::HashMap};

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
        // Group tasks by their script display name.
        let mut tasks_by_script: HashMap<String, Vec<(&str, Option<String>, &str)>> =
            HashMap::new();
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

        // Convert the HashMap into a vector, mapping inner types to String.
        let mut sorted_tasks: Vec<(String, Vec<(String, Option<String>, String)>)> =
            tasks_by_script
                .into_iter()
                .map(|(script_name, tasks)| {
                    let tasks = tasks
                        .into_iter()
                        .map(|(name, desc, script_id)| {
                            (name.to_string(), desc, script_id.to_string())
                        })
                        .collect();
                    (script_name, tasks)
                })
                .collect();

        // Sort so that the root tasks (with an empty script name) come first.
        sorted_tasks.sort_by(|(k1, _), (k2, _)| {
            if k1.is_empty() && !k2.is_empty() {
                Ordering::Less
            } else if !k1.is_empty() && k2.is_empty() {
                Ordering::Greater
            } else {
                k1.cmp(k2)
            }
        });

        // Iterate over the sorted vector to print the tasks.
        for (script_name, tasks) in sorted_tasks {
            if script_name.is_empty() {
                // Print tasks for the root script without a header.
                for (name, desc, _script_id) in tasks {
                    // Print (Default) instead of any prefix for the default task.
                    let display_name = if name == "default" {
                        "default_task".to_string()
                    } else {
                        name.to_string()
                    };
                    match desc {
                        Some(desc) => println!("{:<25} {}", display_name, desc),
                        None => println!("{}", display_name),
                    }
                }
            } else {
                // Print a header for non-root scripts.
                println!("\n{}", script_name.bold().blue());
                for (name, desc, script_id) in tasks {
                    // For a default task on non-root scripts prepend the script ID.
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
        }
        println!();
        Ok(())
    }
}
