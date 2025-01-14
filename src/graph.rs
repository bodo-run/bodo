use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for a node in the graph
pub type NodeId = u64;

/// Represents the type of a node in the graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeKind {
    /// A structured task with dependencies, environment, etc.
    Task(TaskData),
    /// A simpler stand-alone command
    Command(CommandData),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskData {
    pub name: String,
    pub description: Option<String>,
    pub dependencies: Vec<String>,
    pub env: HashMap<String, String>,
    pub working_dir: Option<String>,
    pub concurrency: Option<usize>,
    pub fail_fast: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandData {
    pub raw_command: String,
    pub description: Option<String>,
    pub env: HashMap<String, String>,
    pub working_dir: Option<String>,
}

/// A node in the graph representing either a task or a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub kind: NodeKind,
    /// Arbitrary metadata for plugins to read/write
    pub metadata: HashMap<String, String>,
}

/// Directed edge representing a dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub from: NodeId,
    pub to: NodeId,
    pub edge_type: EdgeType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeType {
    /// Must run before
    Dependency,
    /// Can run in parallel with
    Parallel,
    /// Must run after
    PostTask,
}

/// The core Graph structure
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub node_lookup: HashMap<String, NodeId>,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            node_lookup: HashMap::new(),
        }
    }

    /// Add a node to the graph and return its NodeId
    pub fn add_node(&mut self, kind: NodeKind) -> NodeId {
        let id = self.nodes.len() as NodeId;
        let node = Node {
            id,
            kind: kind.clone(),
            metadata: HashMap::new(),
        };

        // Add to lookup table if it's a named task
        if let NodeKind::Task(task_data) = kind {
            self.node_lookup.insert(task_data.name, id);
        }

        self.nodes.push(node);
        id
    }

    /// Add a directed edge between two nodes
    pub fn add_edge(&mut self, from: NodeId, to: NodeId, edge_type: EdgeType) {
        self.edges.push(Edge {
            from,
            to,
            edge_type,
        });
    }

    /// Get a node by its name (for tasks)
    pub fn get_node_by_name(&self, name: &str) -> Option<&Node> {
        self.node_lookup
            .get(name)
            .and_then(|&id| self.nodes.get(id as usize))
    }

    /// Get a node by its ID
    pub fn get_node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id as usize)
    }

    /// Get a mutable reference to a node by its ID
    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(id as usize)
    }

    /// Get all edges from a node
    pub fn get_edges_from(&self, from: NodeId) -> Vec<&Edge> {
        self.edges.iter().filter(|e| e.from == from).collect()
    }

    /// Get all edges to a node
    pub fn get_edges_to(&self, to: NodeId) -> Vec<&Edge> {
        self.edges.iter().filter(|e| e.to == to).collect()
    }

    /// Print debug information about the graph
    pub fn print_debug(&self) {
        println!(
            "Graph has {} nodes, {} edges.",
            self.nodes.len(),
            self.edges.len()
        );
        for node in &self.nodes {
            match &node.kind {
                NodeKind::Task(td) => {
                    println!(" Task[{}]: {} - {:?}", node.id, td.name, td.description);
                }
                NodeKind::Command(cd) => {
                    println!(
                        " Cmd[{}]: {} - {:?}",
                        node.id, cd.raw_command, cd.description
                    );
                }
            }
        }
        for edge in &self.edges {
            println!(
                "  Edge: {} -> {} ({:?})",
                edge.from, edge.to, edge.edge_type
            );
        }
    }
}
