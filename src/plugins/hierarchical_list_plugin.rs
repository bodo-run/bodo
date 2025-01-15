use async_trait::async_trait;
use colored::{ColoredString, Colorize};
use std::any::Any;
use std::collections::BTreeMap;

use crate::{
    errors::Result,
    graph::{Graph, NodeKind},
    plugin::{Plugin, PluginConfig, PluginExecutionContext},
};

/// A plugin that prints tasks grouped by their `script_source` metadata.
/// It ignores NodeKind::Command. It never mentions "node".
///
/// **Expected metadata** on each Task node:
///   - "script_source" => "scripts/code_quality.yaml" (or omitted => "Root level tasks")
///   - "script_description" => "Code quality commands" (optional)
#[derive(Default)]
pub struct HierarchicalListPlugin;

// Type alias for the complex return type
type TaskGroups = BTreeMap<String, (Option<String>, Vec<(String, Option<String>)>)>;

impl HierarchicalListPlugin {
    pub fn new() -> Self {
        Self
    }

    /// Gather tasks, grouped by script_source. Returns a map:
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

    /// Build a single string listing tasks by script.
    pub fn build_hierarchical_list(&self, graph: &Graph) -> String {
        let mut output = String::new();
        let groups = self.gather_tasks(graph);

        if groups.is_empty() {
            output.push_str("No tasks found.\n");
            return output;
        }

        for (script_source, (script_desc, tasks)) in groups {
            if script_source == "Root level tasks" {
                output.push_str(&format!("{}\n\n", script_source.dimmed()));
            } else {
                let dark_blue_bold: ColoredString = script_source.bold().truecolor(0, 0, 139);
                output.push_str(&format!("{}\n", dark_blue_bold));
            }

            if let Some(desc) = script_desc {
                if !desc.trim().is_empty() {
                    output.push_str(&format!("  {}\n\n", desc.dimmed()));
                } else {
                    output.push('\n');
                }
            } else {
                output.push('\n');
            }

            for (t_name, t_desc) in tasks {
                output.push_str(&format!("  {}:\n", t_name.bold()));

                if let Some(desc) = t_desc {
                    if !desc.trim().is_empty() {
                        output.push_str(&format!("    {}\n\n", desc.dimmed()));
                    } else {
                        output.push('\n');
                    }
                } else {
                    output.push('\n');
                }
            }

            output.push('\n');
        }

        output
    }
}

#[async_trait]
impl Plugin for HierarchicalListPlugin {
    fn name(&self) -> &'static str {
        "HierarchicalListPlugin"
    }

    async fn on_init(&mut self, _config: &PluginConfig) -> Result<()> {
        Ok(())
    }

    async fn on_graph_build(&mut self, graph: &mut Graph) -> Result<()> {
        let output = self.build_hierarchical_list(graph);
        println!("{}", output);
        Ok(())
    }

    fn on_task_start(&mut self) {}

    fn as_any(&self) -> &dyn Any {
        self
    }

    async fn on_lifecycle_event(&mut self, _ctx: &PluginExecutionContext<'_>) -> Result<()> {
        Ok(())
    }
}
