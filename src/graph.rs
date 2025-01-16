use crate::errors::BodoError;
use crate::Result;
use std::collections::HashMap;

/// Unique identifier for a node in the graph.
pub type NodeId = u64;

/// Represents the type of a node in the graph.
#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    Task(TaskData),
    Command(CommandData),
    ConcurrentGroup(ConcurrentGroupData),
}

/// Represents data for a Task node.
#[derive(Debug, Clone, PartialEq)]
pub struct TaskData {
    /// The name of the task
    pub name: String,
    /// The description of the task
    pub description: Option<String>,
    /// The command to run
    pub command: Option<String>,
    /// The working directory for the task
    pub working_dir: Option<String>,
    /// Environment variables for the task
    pub env: HashMap<String, String>,
}

/// Represents data for a Command node.
#[derive(Debug, Clone, PartialEq)]
pub struct CommandData {
    pub raw_command: String,
    pub description: Option<String>,
    pub working_dir: Option<String>,
    pub env: HashMap<String, String>,
}

/// Represents data for a ConcurrentGroup node.
#[derive(Debug, Clone, PartialEq)]
pub struct ConcurrentGroupData {
    /// The nodes to run in parallel
    pub child_nodes: Vec<NodeId>,

    /// If true, fail the entire group as soon as one child fails
    pub fail_fast: bool,

    /// Optional concurrency limit
    pub max_concurrent: Option<usize>,

    /// Optional timeout in seconds
    pub timeout_secs: Option<u64>,
}

/// A node in the graph
#[derive(Debug)]
pub struct Node {
    pub id: NodeId,
    pub kind: NodeKind,
    /// Arbitrary metadata for plugins
    pub metadata: HashMap<String, String>,
}

/// A directed edge in the graph (dependency or order).
#[derive(Debug)]
pub struct Edge {
    pub from: NodeId,
    pub to: NodeId,
}

/// Core Graph structure.
#[derive(Debug, Default)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Add a node and return its NodeId.
    pub fn add_node(&mut self, kind: NodeKind) -> NodeId {
        let id = self.nodes.len() as NodeId;
        self.nodes.push(Node {
            id,
            kind,
            metadata: HashMap::new(),
        });
        id
    }

    /// Add an edge.
    pub fn add_edge(&mut self, from: NodeId, to: NodeId) -> Result<()> {
        if from as usize >= self.nodes.len() || to as usize >= self.nodes.len() {
            return Err(BodoError::PluginError("Invalid node ID".to_string()));
        }
        self.edges.push(Edge { from, to });
        Ok(())
    }

    /// Detects cycles in the graph using DFS
    pub fn has_cycle(&self) -> bool {
        let mut visited = vec![false; self.nodes.len()];
        let mut stack = vec![false; self.nodes.len()];

        fn dfs(graph: &Graph, u: usize, visited: &mut [bool], stack: &mut [bool]) -> bool {
            if stack[u] {
                return true;
            }
            if visited[u] {
                return false;
            }
            visited[u] = true;
            stack[u] = true;
            for e in &graph.edges {
                if e.from as usize == u {
                    if dfs(graph, e.to as usize, visited, stack) {
                        return true;
                    }
                }
            }
            stack[u] = false;
            false
        }

        for i in 0..self.nodes.len() {
            if dfs(self, i, &mut visited, &mut stack) {
                return true;
            }
        }
        false
    }

    /// Returns a topological sort of the graph nodes
    pub fn topological_sort(&self) -> Result<Vec<NodeId>> {
        let mut in_degree = vec![0; self.nodes.len()];
        for e in &self.edges {
            in_degree[e.to as usize] += 1;
        }

        let mut queue = std::collections::VecDeque::new();
        for (i, &deg) in in_degree.iter().enumerate() {
            if deg == 0 {
                queue.push_back(i);
            }
        }

        let mut sorted = Vec::new();
        while let Some(u) = queue.pop_front() {
            sorted.push(u as u64);
            for e in &self.edges {
                if e.from as usize == u {
                    in_degree[e.to as usize] -= 1;
                    if in_degree[e.to as usize] == 0 {
                        queue.push_back(e.to as usize);
                    }
                }
            }
        }

        if sorted.len() != self.nodes.len() {
            return Err(BodoError::PluginError(
                "Graph has cycles or is disconnected".to_string(),
            ));
        }

        Ok(sorted)
    }
}
