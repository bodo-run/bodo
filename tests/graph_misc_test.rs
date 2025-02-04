use bodo::graph::Graph;

#[test]
fn test_format_cycle_error_empty() {
    let graph = Graph::new();
    // Calling format_cycle_error with an empty cycle slice may panic; catch the panic.
    let result = std::panic::catch_unwind(|| {
        graph.format_cycle_error(&[]);
    });
    assert!(
        result.is_err(),
        "Expected panic when formatting an empty cycle"
    );
}

#[test]
fn test_print_debug_no_nodes() {
    let graph = Graph::new();
    // Calling print_debug on an empty graph should not panic.
    graph.print_debug();
}
