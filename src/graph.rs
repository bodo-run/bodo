use petgraph::graph::DiGraph;
use std::collections::HashMap;

pub struct TaskGraph {
    graph: DiGraph<String, ()>,
    node_map: HashMap<String, petgraph::graph::NodeIndex>,
}

impl TaskGraph {
    pub fn new() -> Self {
        TaskGraph {
            graph: DiGraph::new(),
            node_map: HashMap::new(),
        }
    }

    pub fn add_task(&mut self, task: String) {
        if !self.node_map.contains_key(&task) {
            let node_idx = self.graph.add_node(task.clone());
            self.node_map.insert(task, node_idx);
        }
    }

    pub fn add_dependency(&mut self, task: String, dependency: String) {
        self.add_task(task.clone());
        self.add_task(dependency.clone());

        let task_idx = self.node_map[&task];
        let dep_idx = self.node_map[&dependency];

        if !self.graph.contains_edge(dep_idx, task_idx) {
            self.graph.add_edge(dep_idx, task_idx, ());
        }
    }

    pub fn get_execution_order(&self) -> Vec<String> {
        let mut order = Vec::new();
        let mut visited = HashMap::new();

        for node in self.graph.node_indices() {
            if !visited.contains_key(&node) {
                self.visit_node(node, &mut visited, &mut order);
            }
        }

        order.iter().map(|&node| self.graph[node].clone()).collect()
    }

    fn visit_node(
        &self,
        node: petgraph::graph::NodeIndex,
        visited: &mut HashMap<petgraph::graph::NodeIndex, bool>,
        order: &mut Vec<petgraph::graph::NodeIndex>,
    ) {
        visited.insert(node, true);

        for neighbor in self
            .graph
            .neighbors_directed(node, petgraph::Direction::Incoming)
        {
            if !visited.contains_key(&neighbor) {
                self.visit_node(neighbor, visited, order);
            }
        }

        order.push(node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_graph() {
        let mut graph = TaskGraph::new();

        // Add tasks
        graph.add_task("task1".to_string());
        graph.add_task("task2".to_string());
        graph.add_task("task3".to_string());

        // Add dependencies
        graph.add_dependency("task2".to_string(), "task1".to_string());
        graph.add_dependency("task3".to_string(), "task2".to_string());

        // Get execution order
        let order = graph.get_execution_order();

        // Check order
        assert!(order.contains(&"task1".to_string()));
        assert!(order.contains(&"task2".to_string()));
        assert!(order.contains(&"task3".to_string()));
    }
}
