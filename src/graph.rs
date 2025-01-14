use std::collections::HashMap;

/// Unique identifier for a node in the graph.
pub type NodeId = u64;

/// Represents the type of a node in the graph.
#[derive(Debug, PartialEq)]
pub enum NodeKind {
    Task(TaskData),
    Command(CommandData),
}

/// Represents data for a Task node.
#[derive(Debug, PartialEq)]
pub struct TaskData {
    pub name: String,
    pub description: Option<String>,
}

/// Represents data for a Command node.
#[derive(Debug, PartialEq)]
pub struct CommandData {
    pub raw_command: String,
    pub description: Option<String>,
}

/// A node in the graph
#[derive(Debug)]
pub struct Node {
    pub id: NodeId,
    pub kind: NodeKind,
    pub metadata: HashMap<String, String>,
}

/// A directed edge in the graph (dependency or order).
#[derive(Debug, PartialEq)]
pub struct Edge {
    pub from: NodeId,
    pub to: NodeId,
}

/// Core Graph structure.
#[derive(Debug)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
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
    pub fn add_edge(&mut self, from: NodeId, to: NodeId) {
        self.edges.push(Edge { from, to });
    }

    /// Debugging function to print the graph structure.
    pub fn print_debug(&self) {
        println!("Graph Debug:");
        println!("  Nodes: {}", self.nodes.len());
        for node in &self.nodes {
            println!("    Node {} -> {:?}", node.id, node.kind);
        }
        println!("  Edges: {}", self.edges.len());
        for edge in &self.edges {
            println!("    {} -> {}", edge.from, edge.to);
        }
    }
}
