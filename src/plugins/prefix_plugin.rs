use colored::Color;
use std::any::Any;

use crate::{
    graph::{Graph, NodeKind},
    plugin::Plugin,
    Result,
};

const DEFAULT_COLORS: &[Color] = &[
    Color::Blue,
    Color::Green,
    Color::Magenta,
    Color::Cyan,
    Color::Yellow,
    Color::BrightRed,
];

pub struct PrefixPlugin {
    color_index: usize,
}

impl PrefixPlugin {
    pub fn new() -> Self {
        Self { color_index: 0 }
    }

    fn next_color(&mut self) -> String {
        let c = DEFAULT_COLORS[self.color_index % DEFAULT_COLORS.len()];
        self.color_index += 1;
        format!("{:?}", c).to_lowercase()
    }
}

impl Default for PrefixPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for PrefixPlugin {
    fn name(&self) -> &'static str {
        "PrefixPlugin"
    }

    fn priority(&self) -> i32 {
        90
    }

    fn on_graph_build(&mut self, graph: &mut Graph) -> Result<()> {
        for node in &mut graph.nodes {
            let label = match &node.kind {
                NodeKind::Task(task) => Some(task.name.clone()),
                NodeKind::Command(cmd) => cmd.description.clone().or_else(|| {
                    cmd.raw_command
                        .split_whitespace()
                        .next()
                        .map(|s| s.to_string())
                }),
                NodeKind::ConcurrentGroup(_) => None,
            };

            if let Some(label_str) = label {
                let color = self.next_color();
                node.metadata
                    .insert("prefix_enabled".to_string(), "true".to_string());
                node.metadata.insert("prefix_label".to_string(), label_str);
                node.metadata.insert("prefix_color".to_string(), color);
            }
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
