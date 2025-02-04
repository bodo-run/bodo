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

    pub fn next_color(&mut self) -> String {
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

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn on_graph_build(&mut self, graph: &mut Graph) -> Result<()> {
        let mut updates = Vec::new();
        for node in graph.nodes.iter() {
            if let NodeKind::ConcurrentGroup(group_data) = &node.kind {
                let prefix_output = node
                    .metadata
                    .get("prefix_output")
                    .map(|s| s == "true")
                    .unwrap_or(false);

                if prefix_output {
                    let user_color = node.metadata.get("prefix_color").cloned();
                    for &child_id in &group_data.child_nodes {
                        let child_node = &graph.nodes[child_id as usize];
                        let (label, default_color) = match &child_node.kind {
                            NodeKind::Task(t) => (t.name.clone(), self.next_color()),
                            NodeKind::Command(_) => {
                                (format!("cmd-{}", child_id), self.next_color())
                            }
                            NodeKind::ConcurrentGroup(_) => {
                                (format!("group-{}", child_id), self.next_color())
                            }
                        };
                        let chosen_color = user_color.clone().unwrap_or(default_color);
                        updates.push((child_id as usize, label, chosen_color));
                    }
                }
            }
        }
        for (node_idx, label, color) in updates {
            let node = &mut graph.nodes[node_idx];
            node.metadata
                .insert("prefix_enabled".to_string(), "true".to_string());
            node.metadata.insert("prefix_label".to_string(), label);
            node.metadata.insert("prefix_color".to_string(), color);
        }
        Ok(())
    }
}
