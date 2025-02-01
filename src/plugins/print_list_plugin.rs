use async_trait::async_trait;
use colored::Colorize;
use std::{any::Any, cmp::Ordering, collections::HashMap};

use crate::{
    graph::{Graph, NodeKind},
    plugin::Plugin,
    Result,
};

/// Represents a line to print:
/// - If `script_name` is non-empty and `is_heading` is true, this line is a heading (e.g., "Build Script").
/// - If `is_heading` is false, this line is a task (left column plus optional description).
struct TaskLine {
    #[allow(dead_code)] // no sure why this is dead code
    script_name: String,
    is_heading: bool,
    left_col: String,
    desc: Option<String>,
}

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
        // 1) Group tasks by script name
        let mut tasks_by_script: HashMap<String, Vec<(String, Option<String>, String)>> =
            HashMap::new();
        for node in &graph.nodes {
            if let NodeKind::Task(task_data) = &node.kind {
                tasks_by_script
                    .entry(task_data.script_display_name.clone())
                    .or_default()
                    .push((
                        task_data.name.clone(),
                        task_data.description.clone(),
                        task_data.script_id.clone(),
                    ));
            }
        }

        // 2) Sort so the root script (empty string) appears first
        let mut sorted_tasks: Vec<(String, Vec<(String, Option<String>, String)>)> =
            tasks_by_script.into_iter().collect();
        sorted_tasks.sort_by(|(k1, _), (k2, _)| {
            if k1.is_empty() && !k2.is_empty() {
                Ordering::Less
            } else if !k1.is_empty() && k2.is_empty() {
                Ordering::Greater
            } else {
                k1.cmp(k2)
            }
        });

        // 3) Build a list of lines we want to print in order
        let mut lines = Vec::<TaskLine>::new();

        for (script_name, tasks) in &sorted_tasks {
            // If this is not the root script, push a heading line
            if !script_name.is_empty() {
                lines.push(TaskLine {
                    script_name: script_name.clone(),
                    is_heading: true,
                    left_col: script_name.bold().blue().to_string(),
                    desc: None,
                });
            }

            // Then push task lines
            for (name, desc, script_id) in tasks {
                if script_name.is_empty() {
                    // Root tasks
                    // If the name is default, display "default_task" so it matches the sample
                    let left = if name == "default" {
                        "default_task".to_string()
                    } else {
                        name.clone()
                    };

                    lines.push(TaskLine {
                        script_name: "".into(),
                        is_heading: false,
                        left_col: left,
                        desc: desc.clone(),
                    });
                } else {
                    // Tasks under a script
                    // If name == default, just show script_id; otherwise "script_id name"
                    let left = if name == "default" {
                        script_id.clone()
                    } else {
                        format!("{} {}", script_id, name)
                    };

                    lines.push(TaskLine {
                        script_name: script_name.clone(),
                        is_heading: false,
                        left_col: left,
                        desc: desc.clone(),
                    });
                }
            }
        }

        // 4) Find the max width for the left column among all *tasks* (ignore headings for alignment)
        let mut max_left_width = 0;
        for line in &lines {
            // Only measure real tasks
            if !line.is_heading {
                let width = line.left_col.len();
                if width > max_left_width {
                    max_left_width = width;
                }
            }
        }

        // Ensure some minimum
        let min_space = 20;
        let padded_width = max_left_width.max(min_space);

        // 5) Print everything. Headings get printed with no left alignment handling;
        //    tasks use a fixed width so the descriptions line up.
        // Track if we've printed a heading before, so we can control newlines.
        let mut printed_first_heading = false;

        for line in lines {
            if line.is_heading {
                if printed_first_heading {
                    println!(); // blank line before subsequent headings
                }
                println!("{}", line.left_col); // heading text
                printed_first_heading = true;
                continue;
            }

            // Print a task line
            if let Some(desc) = line.desc {
                println!(
                    "  {:<width$} {}",
                    line.left_col,
                    desc.dimmed(),
                    width = padded_width
                );
            } else {
                // No description
                println!("  {}", line.left_col);
            }
        }

        println!();
        Ok(())
    }
}
