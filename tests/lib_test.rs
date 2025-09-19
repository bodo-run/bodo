use bodo::{BodoConfig, Graph, GraphManager};

#[test]
fn test_lib_reexports() {
    let config = BodoConfig::default();
    let mut manager = GraphManager::new();
    manager.build_graph(config).unwrap();
    let _graph: Graph = manager.graph;
<<<<<<< HEAD
    // Test passes if no panic occurs
=======
    // Test passed if we reach here without panicking
>>>>>>> origin/main
}
