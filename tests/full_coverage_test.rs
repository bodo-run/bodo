use clap::Parser;

use bodo::cli::Args;
use bodo::config::BodoConfig;
use bodo::errors::BodoError;
use bodo::graph::Graph;
use bodo::manager::GraphManager;
use bodo::plugin::Plugin;
use bodo::Result;

#[test]
fn test_cli_parser() {
    let args = Args::parse_from([
        "bodo", "--debug", "-l", "mytask", "subtask", "--", "arg1", "arg2",
    ]);
    assert_eq!(args.task, Some("mytask".to_string()));
    assert_eq!(args.subtask, Some("subtask".to_string()));
    assert_eq!(args.args, vec!["arg1".to_string(), "arg2".to_string()]);
    assert!(args.debug);
    assert!(args.list);

    // Test default no-argument invocation.
    let default_args = Args::parse_from(["bodo"]);
    assert_eq!(default_args.task, None);
    assert_eq!(default_args.subtask, None);
    assert!(default_args.args.is_empty());
}

#[test]
fn test_bodo_config_generate_schema() {
    let schema = bodo::config::BodoConfig::generate_schema();
    serde_json::to_string_pretty(&schema).unwrap();
}

#[test]
fn test_graph_print_debug() {
    let graph = Graph::new();
    graph.print_debug();
}

#[test]
fn test_graph_detect_cycle_none() {
    let mut graph = Graph::new();
    let _ = graph.add_node(bodo::graph::NodeKind::Task(bodo::graph::TaskData {
        name: "a".to_string(),
        description: None,
        command: Some("echo a".to_string()),
        working_dir: None,
        env: Default::default(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "".to_string(),
        script_display_name: "".to_string(),
        watch: None,
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
    }));
    assert!(graph.detect_cycle().is_none());
}

#[test]
fn test_graph_detect_cycle_some() {
    let mut graph = Graph::new();
    let (id1, id2) = (
        graph.add_node(bodo::graph::NodeKind::Task(bodo::graph::TaskData {
            name: "a".to_string(),
            description: None,
            command: Some("echo a".to_string()),
            working_dir: None,
            env: Default::default(),
            exec_paths: vec![],
            arguments: vec![],
            is_default: false,
            script_id: "".to_string(),
            script_display_name: "".to_string(),
            watch: None,
            pre_deps: vec![],
            post_deps: vec![],
            concurrently: vec![],
            concurrently_options: Default::default(),
        })),
        graph.add_node(bodo::graph::NodeKind::Task(bodo::graph::TaskData {
            name: "b".to_string(),
            description: None,
            command: Some("echo b".to_string()),
            working_dir: None,
            env: Default::default(),
            exec_paths: vec![],
            arguments: vec![],
            is_default: false,
            script_id: "".to_string(),
            script_display_name: "".to_string(),
            watch: None,
            pre_deps: vec![],
            post_deps: vec![],
            concurrently: vec![],
            concurrently_options: Default::default(),
        })),
    );
    graph.add_edge(id1, id2).unwrap();
    graph.add_edge(id2, id1).unwrap();
    let cycle = graph.detect_cycle();
    assert!(cycle.is_some());
}

#[test]
fn test_graph_topological_sort_order() -> bodo::Result<()> {
    let mut graph = Graph::new();
    let a = graph.add_node(bodo::graph::NodeKind::Task(bodo::graph::TaskData {
        name: "A".to_string(),
        description: None,
        command: Some("echo A".to_string()),
        working_dir: None,
        env: Default::default(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "".to_string(),
        script_display_name: "".to_string(),
        watch: None,
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
    }));
    let b = graph.add_node(bodo::graph::NodeKind::Task(bodo::graph::TaskData {
        name: "B".to_string(),
        description: None,
        command: Some("echo B".to_string()),
        working_dir: None,
        env: Default::default(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "".to_string(),
        script_display_name: "".to_string(),
        watch: None,
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
    }));
    graph.add_edge(a, b).unwrap();
    let sorted = graph.topological_sort()?;
    assert_eq!(sorted, vec![a, b]);
    Ok(())
}
