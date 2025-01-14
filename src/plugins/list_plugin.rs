use crate::errors::PluginError;
use crate::graph::Graph;
use crate::plugin::Plugin;

/// Reads the graph and prints tasks/commands in a user-friendly list.
pub struct ListPlugin;

impl ListPlugin {
    pub fn new() -> Self {
        ListPlugin
    }
}

impl Plugin for ListPlugin {
    fn name(&self) -> &'static str {
        "ListPlugin"
    }

    fn on_graph_build(&mut self, graph: &mut Graph) -> Result<(), PluginError> {
        // Possibly store data about tasks or commands in a local structure
        // or just wait until after it's all built.
        println!("ListPlugin: Graph built with {} nodes.", graph.nodes.len());
        Ok(())
    }

    fn on_after_execute(&mut self, graph: &Graph) -> Result<(), PluginError> {
        // Example: Print final list of tasks/commands to user
        for node in &graph.nodes {
            println!("Node ID: {}, Metadata: {:?}", node.id, node.metadata);
        }
        Ok(())
    }
}impl Default for ListPlugin {
    fn default() -> Self {
        Self::new()