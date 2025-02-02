use bodo::graph::{Graph, NodeKind};
use bodo::BodoError;

#[test]
fn test_graph_operations() {
    let mut graph = Graph::new();
    let n1 = graph.add_node(NodeKind::Command(Default::default()));
    let n2 = graph.add_node(NodeKind::Command(Default::default()));
    graph.add_edge(n1, n2).unwrap();

    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.edges.len(), 1);
}

#[test]
fn test_cycle_detection() {
    let mut graph = Graph::new();
    let n1 = graph.add_node(NodeKind::Command(Default::default()));
    let n2 = graph.add_node(NodeKind::Command(Default::default()));
    graph.add_edge(n1, n2).unwrap();
    graph.add_edge(n2, n1).unwrap();

    assert!(graph.detect_cycle().is_some());
}

#[test]
fn test_topological_sort() -> Result<(), BodoError> {
    let mut graph = Graph::new();
    let n1 = graph.add_node(NodeKind::Command(Default::default()));
    let n2 = graph.add_node(NodeKind::Command(Default::default()));
    graph.add_edge(n1, n2)?;

    let sorted = graph.topological_sort()?;
    assert_eq!(sorted, vec![n1, n2]);
    Ok(())
}
