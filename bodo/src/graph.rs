use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::algo::toposort;
use std::collections::HashMap;

pub struct TaskGraph {
    graph: DiGraph<String, ()>,
    node_map: HashMap<String, NodeIndex>,
}

impl TaskGraph {
    pub fn new() -> Self {
        TaskGraph {
            graph: DiGraph::new(),
            node_map: HashMap::new(),
        }
    }

    pub fn add_task(&mut self, name: &str, dependencies: &[String]) {
        let node_idx = self.get_or_create_node(name);

        for dep in dependencies {
            let dep_idx = self.get_or_create_node(dep);
            self.graph.add_edge(dep_idx, node_idx, ());
        }
    }

    fn get_or_create_node(&mut self, name: &str) -> NodeIndex {
        if let Some(&idx) = self.node_map.get(name) {
            idx
        } else {
            let idx = self.graph.add_node(name.to_string());
            self.node_map.insert(name.to_string(), idx);
            idx
        }
    }

    pub fn get_execution_order(&self) -> Result<Vec<String>, String> {
        toposort(&self.graph, None)
            .map_err(|_| "Cycle detected in task dependencies".to_string())
            .map(|sorted| {
                sorted.into_iter()
                    .map(|idx| self.graph[idx].clone())
                    .collect()
            })
    }

    pub fn validate(&self) -> Result<(), String> {
        self.get_execution_order().map(|_| ())
    }
} 