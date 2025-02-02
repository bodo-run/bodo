use std::collections::HashMap;

use crate::config::WatchConfig;
use crate::errors::Result;

pub type NodeId = u64;

#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    Task(TaskData),
    Command(CommandData),
    ConcurrentGroup(ConcurrentGroupData),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TaskData {
    pub name: String,
    pub description: Option<String>,
    pub command: Option<String>,
    pub working_dir: Option<String>,
    pub env: HashMap<String, String>,
    pub exec_paths: Vec<String>,
    pub is_default: bool,
    pub script_id: String,
    pub script_display_name: String,
    pub watch: Option<WatchConfig>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CommandData {
    pub raw_command: String,
    pub description: Option<String>,
    pub working_dir: Option<String>,
    pub env: HashMap<String, String>,
    pub watch: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConcurrentGroupData {
    pub child_nodes: Vec<NodeId>,
    pub fail_fast: bool,
    pub max_concurrent: Option<usize>,
    pub timeout_secs: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub id: NodeId,
    pub kind: NodeKind,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub from: NodeId,
    pub to: NodeId,
}

#[derive(Debug, Clone, Default)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub task_registry: HashMap<String, NodeId>,
}

impl Graph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, kind: NodeKind) -> NodeId {
        let id = self.nodes.len() as u64;
        self.nodes.push(Node {
            id,
            kind,
            metadata: HashMap::new(),
        });
        id
    }

    pub fn add_edge(&mut self, from: NodeId, to: NodeId) -> Result<()> {
        if from >= self.nodes.len() as u64 || to >= self.nodes.len() as u64 {
            return Err(crate::errors::BodoError::PluginError(
                "Invalid node ID".into(),
            ));
        }
        self.edges.push(Edge { from, to });
        Ok(())
    }

    pub fn detect_cycle(&self) -> Option<Vec<NodeId>> {
        let mut visited = vec![false; self.nodes.len()];
        let mut rec_stack = vec![false; self.nodes.len()];
        let mut node_stack = Vec::new();

        for node in 0..self.nodes.len() {
            if self.is_cyclic(node as u64, &mut visited, &mut rec_stack, &mut node_stack) {
                return Some(node_stack);
            }
        }
        None
    }

    fn is_cyclic(
        &self,
        node: NodeId,
        visited: &mut [bool],
        rec_stack: &mut [bool],
        stack: &mut Vec<NodeId>,
    ) -> bool {
        let node_idx = node as usize;
        if !visited[node_idx] {
            visited[node_idx] = true;
            rec_stack[node_idx] = true;
            stack.push(node);

            for edge in &self.edges {
                if edge.from == node {
                    let adjacent = edge.to;
                    if !visited[adjacent as usize]
                        && self.is_cyclic(adjacent, visited, rec_stack, stack)
                        || rec_stack[adjacent as usize]
                    {
                        return true;
                    }
                }
            }

            stack.pop();
            rec_stack[node_idx] = false;
        }
        false
    }

    pub fn format_cycle_error(&self, cycle: &[NodeId]) -> String {
        let mut names = Vec::new();
        for node_id in cycle {
            if let Some(node) = self.nodes.get(*node_id as usize) {
                if let NodeKind::Task(task) = &node.kind {
                    names.push(task.name.clone());
                }
            }
        }
        format!("Circular dependency detected: {}", names.join(" -> "))
    }

    pub fn topological_sort(&self) -> Result<Vec<NodeId>> {
        let mut in_degree = HashMap::new();
        let mut queue = Vec::new();
        let mut result = Vec::new();

        // Initialize in-degree
        for node in 0..self.nodes.len() as u64 {
            in_degree.insert(node, 0);
        }
        for edge in &self.edges {
            *in_degree.entry(edge.to).or_insert(0) += 1;
        }

        // Find nodes with zero in-degree
        for (node, &degree) in &in_degree {
            if degree == 0 {
                queue.push(*node);
            }
        }

        // Kahn's algorithm
        while let Some(node) = queue.pop() {
            result.push(node);
            for edge in &self.edges {
                if edge.from == node {
                    let entry = in_degree.entry(edge.to).or_default();
                    *entry -= 1;
                    if *entry == 0 {
                        queue.push(edge.to);
                    }
                }
            }
        }

        if result.len() != self.nodes.len() {
            Err(crate::errors::BodoError::PluginError(
                "Graph has a cycle".into(),
            ))
        } else {
            Ok(result)
        }
    }
}
