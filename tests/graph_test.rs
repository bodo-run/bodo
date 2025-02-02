use crate::graph::{Graph, NodeId, NodeKind};
use crate::errors::BodoError;

#[test]
fn test_graph_construction() {
    let mut graph = Graph::new();
    let node1 = graph.add_node(NodeKind::Task(Default::default()));
    let node2 = graph.add_node(NodeKind::Command(Default::default()));
    graph.add_edge(node1, node2).unwrap();
    
    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.edges.len(), 1);
}

#[test]
fn test_cycle_detection() {
    let mut graph = Graph::new();
    let n1 = graph.add_node(NodeKind::Task(Default::default()));
    let n2 = graph.add_node(NodeKind::Task(Default::default()));
    graph.add_edge(n1, n2).unwrap();
    graph.add_edge(n2, n1).unwrap();
    
    let cycle = graph.detect_cycle();
    assert!(cycle.is_some());
    assert_eq!(cycle.unwrap().len(), 3); // n1 -> n2 -> n1
}

#[test]
fn test_topological_sort() {
    let mut graph = Graph::new();
    let n1 = graph.add_node(NodeKind::Task(Default::default()));
    let n2 = graph.add_node(NodeKind::Command(Default::default()));
    let n3 = graph.add_node(NodeKind::Task(Default::default()));
    
    graph.add_edge(n1, n2).unwrap();
    graph.add_edge(n1, n3).unwrap();
    graph.add_edge(n2, n3).unwrap();
    
    let sorted = graph.topological_sort().unwrap();
    assert_eq!(sorted, vec![n1, n2, n3]);
}

#[test]
fn test_error_handling() {
    let mut graph = Graph::new();
    let invalid_node = NodeId::MAX;
    assert!(graph.add_edge(invalid_node, 0).is_err());
}

#[test]
fn test_get_node_name() {
    let mut graph = Graph::new();
    let node_id = graph.add_node(NodeKind::Task(Default::default()));
    assert!(!graph.get_node_name(node_id as usize).is_empty());
}