use colored::Colorize;
use std::{any::Any, cmp::Ordering, collections::HashMap};

use bodo::{
    graph::{Graph, NodeKind},
    plugin::Plugin,
    Result,
};

// Task info represents (task_name, description, script_id)
type TaskInfo = (String, Option<String>, String);
type ScriptTasks = Vec<(String, Vec<TaskInfo>)>;

struct TaskLine {
    #[allow(dead_code)]
    script_name: String,
    is_heading: bool,
    left_col: String,
    desc: Option<String>,
}

pub struct PrintListPlugin;

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

    fn on_graph_build(&mut self, graph: &mut Graph) -> Result<()> {
        let mut tasks_by_script: HashMap<String, Vec<TaskInfo>> = HashMap::new();
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
        let mut sorted_tasks: ScriptTasks = tasks_by_script.into_iter().collect();
        sorted_tasks.sort_by(|(k1, _), (k2, _)| {
            if k1.is_empty() && !k2.is_empty() {
                Ordering::Less
            } else if !k1.is_empty() && k2.is_empty() {
                Ordering::Greater
            } else {
                k1.cmp(k2)
            }
        });
        let mut lines = Vec::<TaskLine>::new();

        for (script_name, tasks) in &sorted_tasks {
            if !script_name.is_empty() {
                lines.push(TaskLine {
                    script_name: script_name.clone(),
                    is_heading: true,
                    left_col: script_name.bold().blue().to_string(),
                    desc: None,
                });
            }
            for (name, desc, script_id) in tasks {
                if script_name.is_empty() {
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

        let mut max_left_width = 0;
        for line in &lines {
            if !line.is_heading {
                let width = line.left_col.len();
                if width > max_left_width {
                    max_left_width = width;
                }
            }
        }
        let min_space = 20;
        let padded_width = max_left_width.max(min_space);

        let mut printed_first_heading = false;
        for line in lines {
            if line.is_heading {
                if printed_first_heading {
                    tracing::info!("");
                }
                tracing::info!("{}", line.left_col);
                printed_first_heading = true;
                continue;
            }
            if let Some(desc) = line.desc {
                tracing::info!(
                    "  {:<width$} {}",
                    line.left_col,
                    desc.dimmed(),
                    width = padded_width
                );
            } else {
                tracing::info!("  {}", line.left_col);
            }
        }
        tracing::info!("");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bodo::graph::{Graph, NodeKind, TaskData};
    use std::collections::HashMap;

    #[test]
    fn test_print_list_plugin() {
        let mut plugin = PrintListPlugin;

        let mut graph = Graph::new();

        // Create two tasks with different properties
        let task_data1 = TaskData {
            name: "task1".to_string(),
            description: Some("First task".to_string()),
            command: Some("echo Task 1".to_string()),
            working_dir: None,
            env: HashMap::new(),
            exec_paths: vec![],
            arguments: vec![],
            pre_deps: vec![],
            post_deps: vec![],
            concurrently: vec![],
            concurrently_options: Default::default(),
            is_default: false,
            script_id: "script1".to_string(),
            script_display_name: "Script 1".to_string(),
            watch: None,
        };

        let task_data2 = TaskData {
            name: "task2".to_string(),
            description: Some("Second task".to_string()),
            command: Some("echo Task 2".to_string()),
            working_dir: None,
            env: HashMap::new(),
            exec_paths: vec![],
            arguments: vec![],
            pre_deps: vec![],
            post_deps: vec![],
            concurrently: vec![],
            concurrently_options: Default::default(),
            is_default: false,
            script_id: "script2".to_string(),
            script_display_name: "Script 2".to_string(),
            watch: None,
        };

        // Add nodes to graph
        graph.add_node(NodeKind::Task(task_data1));
        graph.add_node(NodeKind::Task(task_data2));

        // Apply the plugin
        let result = plugin.on_graph_build(&mut graph);
        assert!(result.is_ok(), "Plugin execution failed");
    }
}
