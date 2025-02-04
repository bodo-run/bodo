use crate::config::WatchConfig;
use crate::errors::BodoError;
use crate::Result;
use log::debug;
use std::collections::HashMap;

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
    pub arguments: Vec<crate::config::TaskArgument>,
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, Default)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub task_registry: HashMap<String, NodeId>,
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub from: NodeId,
    pub to: NodeId,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            task_registry: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, kind: NodeKind) -> NodeId {
        let id = self.nodes.len() as NodeId;
        self.nodes.push(Node {
            id,
            kind,
            metadata: HashMap::new(),
        });
        id
    }

    pub fn add_edge(&mut self, from: NodeId, to: NodeId) -> Result<()> {
        if from as usize >= self.nodes.len() || to as usize >= self.nodes.len() {
            return Err(BodoError::PluginError("Invalid node ID".to_string()));
        }
        self.edges.push(Edge { from, to });
        Ok(())
    }

    pub fn print_debug(&self) {
        debug!("\nGraph Debug Info:");
        debug!("Nodes: {}", self.nodes.len());
        for node in &self.nodes {
            match &node.kind {
                NodeKind::Task(task) => {
                    debug!("  Task[{}]: {}", node.id, task.name);
                    if let Some(desc) = &task.description {
                        debug!("    Description: {}", desc);
                    }
                    if let Some(cmd) = &task.command {
                        debug!("    Command: {}", cmd);
                    }
                    if let Some(dir) = &task.working_dir {
                        debug!("    Working Dir: {}", dir);
                    }
                    if !task.env.is_empty() {
                        debug!("    Environment:");
                        for (k, v) in &task.env {
                            debug!("      {}={}", k, v);
                        }
                    }
                }
                NodeKind::Command(cmd) => {
                    debug!("  Command[{}]: {}", node.id, cmd.raw_command);
                }
                NodeKind::ConcurrentGroup(group) => {
                    debug!("  ConcurrentGroup[{}]:", node.id);
                    debug!("    Children: {:?}", group.child_nodes);
                    debug!("    Fail Fast: {}", group.fail_fast);
                    if let Some(max) = group.max_concurrent {
                        debug!("    Max Concurrent: {}", max);
                    }
                    if let Some(timeout) = group.timeout_secs {
                        debug!("    Timeout: {}s", timeout);
                    }
                }
            }
            if !node.metadata.is_empty() {
                debug!("    Metadata:");
                for (k, v) in &node.metadata {
                    debug!("      {}={}", k, v);
                }
            }
        }
        debug!("\nEdges: {}", self.edges.len());
        for edge in &self.edges {
            debug!("  {} -> {}", edge.from, edge.to);
        }
        debug!("");
    }

    pub fn detect_cycle(&self) -> Option<Vec<NodeId>> {
        let mut visited = vec![false; self.nodes.len()];
        let mut stack = vec![false; self.nodes.len()];
        let mut parent = vec![None; self.nodes.len()];

        fn dfs(
            graph: &Graph,
            u: usize,
            visited: &mut [bool],
            stack: &mut [bool],
            parent: &mut [Option<usize>],
        ) -> Option<(usize, usize)> {
            visited[u] = true;
            stack[u] = true;

            for e in &graph.edges {
                if e.from as usize == u {
                    let v = e.to as usize;
                    if !visited[v] {
                        parent[v] = Some(u);
                        if let Some(cycle) = dfs(graph, v, visited, stack, parent) {
                            return Some(cycle);
                        }
                    } else if stack[v] {
                        return Some((u, v));
                    }
                }
            }
            stack[u] = false;
            None
        }

        for i in 0..self.nodes.len() {
            if !visited[i] {
                if let Some((from, to)) = dfs(self, i, &mut visited, &mut stack, &mut parent) {
                    return Some(self.reconstruct_cycle(from, to, &parent));
                }
            }
        }
        None
    }

    fn reconstruct_cycle(
        &self,
        mut from: usize,
        to: usize,
        parent: &[Option<usize>],
    ) -> Vec<NodeId> {
        let mut path = vec![from as NodeId];
        while from != to {
            from = parent[from].unwrap();
            path.push(from as NodeId);
        }
        path.reverse();
        path
    }

    pub fn format_cycle_error(&self, cycle: &[NodeId]) -> String {
        let mut error = String::from("error: found cyclical dependency involving:\n");
        for window in cycle.windows(2) {
            let from = window[0] as usize;
            let to = window[1] as usize;
            let from_name = self.get_node_name(from);
            let to_name = self.get_node_name(to);
            error.push_str(&format!("   --> {} depends on {}\n", from_name, to_name));
        }
        let last = cycle[cycle.len() - 1] as usize;
        let first = cycle[0] as usize;
        let last_name = self.get_node_name(last);
        let first_name = self.get_node_name(first);
        error.push_str(&format!("   --> {} depends on {}\n", last_name, first_name));
        error
    }

    fn get_node_name(&self, node_id: usize) -> String {
        match &self.nodes[node_id].kind {
            NodeKind::Task(task) => {
                if task.script_display_name.is_empty() {
                    task.name.clone()
                } else {
                    format!("{}/{}", task.script_display_name, task.name)
                }
            }
            NodeKind::Command(cmd) => format!("command[{}]", cmd.raw_command),
            NodeKind::ConcurrentGroup(_) => format!("concurrent_group[{}]", node_id),
        }
    }

    pub fn node_name(&self, node_id: usize) -> String {
        self.get_node_name(node_id)
    }

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
