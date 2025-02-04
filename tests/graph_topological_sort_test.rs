use bodo::graph::{Graph, NodeKind, TaskData};
use std::collections::HashMap;

#[test]
fn test_topological_sort_order() {
    let mut graph = Graph::new();

    let a = graph.add_node(NodeKind::Task(TaskData {
        name: "A".to_string(),
        description: None,
        command: Some("echo A".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "".to_string(),
        script_display_name: "".to_string(),
        watch: None,
    }));

    let b = graph.add_node(NodeKind::Task(TaskData {
        name: "B".to_string(),
        description: None,
        command: Some("echo B".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "".to_string(),
        script_display_name: "".to_string(),
        watch: None,
    }));

    // a -> b
    graph.add_edge(a, b).unwrap();
    let sorted = graph.topological_sort().unwrap();
    assert_eq!(sorted, vec![a, b]);
}
