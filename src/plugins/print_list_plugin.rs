use async_trait::async_trait;
use colored::Colorize;
use std::any::Any;
use std::collections::BTreeMap;

use crate::{
    errors::Result,
    graph::{Graph, NodeKind},
    plugin::{Plugin, PluginConfig},
};

/// Struct to hold basic info about a task for rendering.
#[derive(Debug)]
struct TaskInfo {
    left_label: String,  // e.g. (default task) or "build clippy"
    description: String, // possibly multi-line
}

/// Info about each script file group
#[derive(Debug)]
struct FileGroup {
    /// The file path, e.g. scripts/build/script.yaml
    path: String,
    /// Top-level script description
    script_desc: String,
    /// List of tasks
    tasks: Vec<TaskInfo>,
}

/// A plugin that prints a "nice help" when `--list` is requested.
#[derive(Default)]
pub struct PrintListPlugin {
    show_help: bool,
}

impl PrintListPlugin {
    pub fn new(show_help: bool) -> Self {
        Self { show_help }
    }

    /// Breaks multi-line descriptions into lines.
    fn split_description_lines(desc: &str) -> Vec<String> {
        desc.lines().map(|l| l.to_string()).collect()
    }

    /// We gather tasks by the node's `script_file` metadata.
    /// For each file, we also note the `script_desc`, which should be the same for all tasks from that file.
    /// Then we store the tasks in `tasks: Vec<TaskInfo>`.
    fn gather_file_groups(&self, graph: &Graph) -> BTreeMap<String, FileGroup> {
        let mut map = BTreeMap::new();

        for node in &graph.nodes {
            if let NodeKind::Task(task_data) = &node.kind {
                // The `script_file` is set in script_loader
                let path = node
                    .metadata
                    .get("script_file")
                    .cloned()
                    .unwrap_or_else(|| "unknown_path.yaml".to_string());

                let script_desc = node
                    .metadata
                    .get("script_desc")
                    .cloned()
                    .unwrap_or_default();

                // Build the left label
                // If it's default task in the root script => "(default task)"
                // Else we want "<script_prefix> <actual_task>"
                let left_label = if task_data.is_default && task_data.script_name.is_none() {
                    "(default task)".to_string()
                } else {
                    // Get the script name from metadata
                    let script_name = node
                        .metadata
                        .get("script_name")
                        .cloned()
                        .unwrap_or("Root".to_string());
                    // Get the task name part after "#"
                    let short_name = task_data.name.split('#').last().unwrap_or(&task_data.name);
                    if script_name == "Root" {
                        short_name.to_string()
                    } else {
                        format!("{} {}", script_name, short_name)
                    }
                };

                // The description is from `task_data.description`, which can be multi-line.
                let task_desc = task_data
                    .description
                    .as_ref()
                    .map(|s| s.to_string())
                    .unwrap_or_default();

                // Insert or update the group
                let group = map.entry(path.clone()).or_insert_with(|| FileGroup {
                    path,
                    script_desc: script_desc.clone(),
                    tasks: Vec::new(),
                });

                group.tasks.push(TaskInfo {
                    left_label,
                    description: task_desc,
                });
            }
        }

        map
    }

    /// Compute the maximum width for the left_label within a group of tasks
    fn compute_left_label_width(&self, tasks: &[TaskInfo]) -> usize {
        tasks.iter().map(|t| t.left_label.len()).max().unwrap_or(0)
    }

    /// Render one file group in the fancy style
    fn render_file_group(&self, group: &FileGroup) -> String {
        let mut out = String::new();

        // The path is dimmed
        let path_dim = group.path.dimmed();
        out.push_str(&format!("{}\n", path_dim));

        // If the script_desc is not empty, also print it (dimmed), possibly multiple lines
        if !group.script_desc.trim().is_empty() {
            for line in group.script_desc.lines() {
                out.push_str(&format!("{}\n", line.dimmed()));
            }
        }
        out.push('\n');

        // We align left_label with descriptions
        let left_width = self.compute_left_label_width(&group.tasks);
        for task in &group.tasks {
            // Split multi-line desc
            let lines = Self::split_description_lines(&task.description);

            if lines.is_empty() || (lines.len() == 1 && lines[0].trim().is_empty()) {
                // If no description or empty => just print the left_label
                let label_colored = if task.left_label == "(default task)" {
                    task.left_label.bright_green().bold()
                } else {
                    task.left_label.bright_green()
                };
                out.push_str(&format!(
                    "  {:<width$}\n",
                    label_colored,
                    width = left_width
                ));
            } else {
                // Print the first line with the label
                let label_colored = if task.left_label == "(default task)" {
                    task.left_label.bright_green().bold()
                } else {
                    task.left_label.bright_green()
                };
                let first_line = &lines[0];
                out.push_str(&format!(
                    "  {:<width$}  {}\n",
                    label_colored,
                    first_line.dimmed(),
                    width = left_width
                ));
                // If there are more lines, print them aligned under the description column
                for extra_line in lines.iter().skip(1) {
                    if extra_line.trim().is_empty() {
                        out.push_str(&format!(
                            "  {:<width$}\n",
                            "", // just blank
                            width = left_width
                        ));
                    } else {
                        out.push_str(&format!(
                            "  {:<width$}  {}\n",
                            "",
                            extra_line.dimmed(),
                            width = left_width
                        ));
                    }
                }
            }
        }

        out.push('\n');
        out
    }
}

#[async_trait]
impl Plugin for PrintListPlugin {
    fn name(&self) -> &'static str {
        "PrintListPlugin"
    }

    async fn on_init(&mut self, _config: &PluginConfig) -> Result<()> {
        Ok(())
    }

    async fn on_graph_build(&mut self, graph: &mut Graph) -> Result<()> {
        if self.show_help {
            // Gather everything
            let file_groups = self.gather_file_groups(graph);

            // If no tasks found
            if file_groups.is_empty() {
                println!("No tasks found.\n");
                return Ok(());
            }

            // Render each group
            let mut output = String::new();
            for (_, group) in file_groups {
                output.push_str(&self.render_file_group(&group));
            }

            print!("{}", output);
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn on_task_start(&mut self) {
        // Nothing to do
    }
}
