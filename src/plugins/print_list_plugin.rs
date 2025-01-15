use async_trait::async_trait;
use colored::Colorize;
use std::any::Any;
use std::collections::BTreeMap;

use crate::{
    errors::Result,
    graph::{Graph, NodeKind},
    plugin::{Plugin, PluginConfig},
};

/// Type alias for the task groups map
type TaskGroups = BTreeMap<String, Vec<(String, Option<String>)>>;

/// A plugin that prints a "nice help" when `--list` is requested.
/// We gather tasks grouped by `script_name` metadata. NodeKind::Command is
/// ignored in the listing.
#[derive(Default)]
pub struct PrintListPlugin {
    show_help: bool,
}

impl PrintListPlugin {
    pub fn new(show_help: bool) -> Self {
        Self { show_help }
    }

    /// Group tasks by their `script_name` metadata, returning a map:
    ///   script_name -> Vec<(task_name, task_desc)>
    fn gather_tasks(&self, graph: &Graph) -> TaskGroups {
        let mut map = BTreeMap::new();

        // First pass: gather tasks and deduplicate by name
        for node in &graph.nodes {
            if let NodeKind::Task(task_data) = &node.kind {
                let script_name = node
                    .metadata
                    .get("script_name")
                    .cloned()
                    .unwrap_or_else(|| "Root".to_string());

                let entry = map.entry(script_name).or_insert(Vec::new());

                // Check if this task name already exists
                let task_base_name = task_data.name.split('#').last().unwrap_or(&task_data.name);
                if !entry.iter().any(|(name, _): &(String, Option<String>)| {
                    name.split('#').last().unwrap_or(name) == task_base_name
                }) {
                    entry.push((task_data.name.clone(), task_data.description.clone()));
                }
            }
        }

        // Sort tasks within each group
        for tasks in map.values_mut() {
            tasks.sort_by(|a, b| {
                // Default task always comes first
                if a.0.ends_with("#default") {
                    return std::cmp::Ordering::Less;
                }
                if b.0.ends_with("#default") {
                    return std::cmp::Ordering::Greater;
                }
                // Otherwise sort by task name
                let a_name = a.0.split('#').last().unwrap_or(&a.0);
                let b_name = b.0.split('#').last().unwrap_or(&b.0);
                a_name.cmp(b_name)
            });
        }

        // Ensure "Root" tasks come first by creating a new ordered map
        let mut ordered_map = BTreeMap::new();
        if let Some(root_tasks) = map.remove("Root") {
            ordered_map.insert("Root".to_string(), root_tasks);
        }
        ordered_map.extend(map);

        ordered_map
    }

    /// Renders the tasks in the "nice help" format requested.
    fn build_help_output(&self, graph: &Graph) -> String {
        let groups = self.gather_tasks(graph);
        if groups.is_empty() {
            return "No tasks found.\n".to_string();
        }

        // Calculate max task name length across all groups
        let max_name_len = groups
            .values()
            .flat_map(|tasks| tasks.iter())
            .map(|(name, _)| {
                if name.ends_with("#default") {
                    "(default task)".len()
                } else {
                    name.split('#').last().unwrap_or(name).len()
                }
            })
            .max()
            .unwrap_or(0);

        let mut output = String::new();

        for (script_name, tasks) in groups {
            // Print header based on script name
            let header = if script_name == "Root" {
                "Root tasks".bright_blue()
            } else {
                script_name.bright_blue()
            };
            output.push_str(&format!("{}\n\n", header));

            for (t_name, t_desc) in tasks {
                let display_name = if t_name.ends_with("#default") {
                    "(default task)".bright_green().bold()
                } else {
                    // Remove script# prefix for all tasks
                    let name = t_name.split('#').last().unwrap_or(&t_name);
                    name.bright_green().bold()
                };

                if let Some(d) = t_desc {
                    if !d.trim().is_empty() {
                        output.push_str(&format!(
                            "  {:<width$}  {}\n",
                            display_name,
                            d.bright_black(),
                            width = max_name_len
                        ));
                    } else {
                        output.push_str(&format!(
                            "  {:<width$}\n",
                            display_name,
                            width = max_name_len
                        ));
                    }
                } else {
                    output.push_str(&format!(
                        "  {:<width$}\n",
                        display_name,
                        width = max_name_len
                    ));
                }
            }
            output.push('\n');
        }

        output
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
            print!("{}", self.build_help_output(graph));
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn on_task_start(&mut self) {
        // Nothing to do on task start for this plugin
    }
}
