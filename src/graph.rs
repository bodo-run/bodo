use std::collections::HashMap;

/// Unique identifier for a node in the graph
pub type NodeId = u64;

/// Represents the type of a node in the graph.
/// Task: a structured set of pre/post dependencies, environment, etc.
/// Command: a simpler stand-alone command (like a shell script).
#[derive(Debug)]
pub enum NodeKind {
    Task(TaskData),
    Command(CommandData),
}

#[derive(Debug)]
pub struct TaskData {
    pub name: String,
    pub description: Option<String>,
    // Possibly references to concurrency settings, dependencies, etc.
}

#[derive(Debug)]
pub struct CommandData {
    pub raw_command: String,
    pub description: Option<String>,
}

/// A node in the graph. Could represent either a task or a single command.
#[derive(Debug)]
pub struct Node {
    pub id: NodeId,
    pub kind: NodeKind,
    // Arbitrary metadata for plugins to read/write.
    pub metadata: HashMap<String, String>,
}

/// Directed edge representing a dependency or an execution order constraint.
#[derive(Debug)]
pub struct Edge {
    pub from: NodeId,
    pub to: NodeId,
}

/// The core structure to hold tasks, commands, and the edges between them.
#[derive(Debug)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    // Optionally, track node lookup by name, etc.
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}

impl Graph {
    /// Creates a new, empty graph.
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Add a node to the graph and return its NodeId.
    pub fn add_node(&mut self, kind: NodeKind) -> NodeId {
        let id = self.nodes.len() as NodeId;
        self.nodes.push(Node {
            id,
            kind,
            metadata: HashMap::new(),
        });
        id
    }

    /// Add a directed edge between two nodes.
    pub fn add_edge(&mut self, from: NodeId, to: NodeId) {
        self.edges.push(Edge { from, to });
    }

    /// Debug/print the graph structure.
    pub fn print_debug(&self) {
        println!("Graph Debug:");
        println!("Nodes: {}", self.nodes.len());
        for node in &self.nodes {
            println!("  Node {}: {:?}", node.id, node.kind);
        }
        println!("Edges: {}", self.edges.len());
        for edge in &self.edges {
            println!("  {} -> {}", edge.from, edge.to);
        }
    }
}
