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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_graph() {
        let graph = TaskGraph::new();
        assert!(graph.node_map.is_empty());
        assert_eq!(graph.graph.node_count(), 0);
        assert_eq!(graph.graph.edge_count(), 0);
    }

    #[test]
    fn test_add_task_without_dependencies() {
        let mut graph = TaskGraph::new();
        graph.add_task("task1", &[]);
        
        assert_eq!(graph.node_map.len(), 1);
        assert_eq!(graph.graph.node_count(), 1);
        assert_eq!(graph.graph.edge_count(), 0);
    }

    #[test]
    fn test_add_task_with_dependencies() {
        let mut graph = TaskGraph::new();
        graph.add_task("task1", &[]);
        graph.add_task("task2", &[String::from("task1")]);
        
        assert_eq!(graph.node_map.len(), 2);
        assert_eq!(graph.graph.node_count(), 2);
        assert_eq!(graph.graph.edge_count(), 1);
    }

    #[test]
    fn test_execution_order_simple() {
        let mut graph = TaskGraph::new();
        graph.add_task("task1", &[]);
        graph.add_task("task2", &[String::from("task1")]);
        
        let order = graph.get_execution_order().unwrap();
        assert_eq!(order, vec!["task1", "task2"]);
    }

    #[test]
    fn test_execution_order_complex() {
        let mut graph = TaskGraph::new();
        graph.add_task("task1", &[]);
        graph.add_task("task2", &[String::from("task1")]);
        graph.add_task("task3", &[String::from("task1")]);
        graph.add_task("task4", &[String::from("task2"), String::from("task3")]);
        
        let order = graph.get_execution_order().unwrap();
        assert_eq!(order[0], "task1");
        assert!(order.contains(&"task2"));
        assert!(order.contains(&"task3"));
        assert_eq!(order[3], "task4");
    }

    #[test]
    fn test_cyclic_dependency_detection() {
        let mut graph = TaskGraph::new();
        graph.add_task("task1", &[String::from("task2")]);
        graph.add_task("task2", &[String::from("task1")]);
        
        assert!(graph.get_execution_order().is_err());
    }
} 