use async_trait::async_trait;
use colored::{ColoredString, Colorize};
use std::any::Any;
use std::collections::BTreeMap;

use crate::{
    errors::Result,
    graph::{Graph, NodeKind},
    plugin::{Plugin, PluginConfig, PluginExecutionContext},
};

/// Type alias for the task groups map
type TaskGroups = BTreeMap<String, (Option<String>, Vec<(String, Option<String>)>)>;

/// A plugin that prints a "nice help" when `--list` is requested.
/// We gather tasks grouped by `script_source` metadata. NodeKind::Command is
/// ignored in the listing.
#[derive(Default)]
pub struct PrintListPlugin {
    show_help: bool,
}

impl PrintListPlugin {
    pub fn new(show_help: bool) -> Self {
        Self { show_help }
    }

    /// Group tasks by their `script_source` metadata, returning a map:
    ///   script_source -> (script_description, Vec<(task_name, task_desc)>)
    fn gather_tasks(&self, graph: &Graph) -> TaskGroups {
        let mut map = BTreeMap::new();

        for node in &graph.nodes {
            if let NodeKind::Task(task_data) = &node.kind {
                let script_source = node
                    .metadata
                    .get("script_source")
                    .cloned()
                    .unwrap_or_else(|| "Root level tasks".to_string());

                let script_desc = node.metadata.get("script_description").cloned();

                let entry = map.entry(script_source).or_insert((None, Vec::new()));

                if entry.0.is_none() && script_desc.is_some() {
                    entry.0 = script_desc;
                }

                let t_name = task_data.name.clone();
                let t_desc = task_data.description.clone();
                entry.1.push((t_name, t_desc));
            }
        }

        map
    }

    /// Renders the tasks in the "nice help" format requested.
    fn build_help_output(&self, graph: &Graph) -> String {
        let groups = self.gather_tasks(graph);

        if groups.is_empty() {
            return "No tasks found.\n".to_string();
        }

        let mut output = String::new();

        for (script_source, (script_desc, tasks)) in groups {
            if script_source == "Root level tasks" {
                output.push_str(&format!("{}\n", script_source.bright_blue()));
                if let Some(desc) = script_desc {
                    if !desc.trim().is_empty() {
                        output.push_str(&format!("{}\n", desc.bright_black()));
                    }
                }
                output.push('\n');

                for (t_name, t_desc) in tasks {
                    let display_name = if t_name.ends_with("#default") {
                        "(default task)".bright_green().bold()
                    } else {
                        // Remove script# prefix for root tasks
                        let name = t_name.split('#').last().unwrap_or(&t_name);
                        name.bright_green().bold()
                    };
                    output.push_str(&format!("  {}:\n", display_name));
                    if let Some(d) = t_desc {
                        if !d.trim().is_empty() {
                            output.push_str(&format!("    {}\n", d.bright_black()));
                        }
                    }
                    output.push('\n');
                }
            } else {
                let dark_blue_bold: ColoredString = script_source.bright_blue().bold();
                output.push_str(&format!("{}\n", dark_blue_bold));

                if let Some(desc) = script_desc {
                    if !desc.trim().is_empty() {
                        output.push_str(&format!("  {}\n", desc.bright_black()));
                    }
                }
                output.push('\n');

                for (t_name, t_desc) in tasks {
                    output.push_str(&format!("  {}:\n", t_name.bright_green().bold()));
                    if let Some(d) = t_desc {
                        if !d.trim().is_empty() {
                            output.push_str(&format!("    {}\n", d.bright_black()));
                        }
                    }
                    output.push('\n');
                }
            }
        }

        output
    }
}

#[async_trait]
impl Plugin for PrintListPlugin {
    fn name(&self) -> &'static str {
        "PrintListPlugin"
    }

    async fn on_lifecycle_event(&mut self, _ctx: &PluginExecutionContext<'_>) -> Result<()> {
        Ok(())
    }

    async fn on_init(&mut self, _config: &PluginConfig) -> Result<()> {
        Ok(())
    }

    async fn on_graph_build(&mut self, graph: &mut Graph) -> Result<()> {
        if self.show_help {
            let listing = self.build_help_output(graph);
            println!("{}", listing);
        }
        Ok(())
    }

    fn on_task_start(&mut self) {}

    fn as_any(&self) -> &dyn Any {
        self
    }
}
