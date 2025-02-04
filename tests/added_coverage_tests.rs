use bodo::{
    errors::BodoError,
    graph::{Graph, NodeKind, TaskData},
    plugin::{Plugin, PluginConfig, PluginManager},
    plugins::execution_plugin::ExecutionPlugin,
    plugins::timeout_plugin::TimeoutPlugin,
};
use std::collections::HashMap;

// Test printing debug in Graph (capture logs using env_logger)
#[test]
fn test_graph_print_debug() {
    let mut graph = Graph::new();
    let _ = graph.add_node(NodeKind::Task(TaskData {
        name: "dummy".to_string(),
        description: Some("Dummy task".to_string()),
        command: Some("echo dummy".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
    }));
    let _ = env_logger::builder().is_test(true).try_init();
    graph.print_debug();
}

// Dummy plugin for PluginManager testing.
struct DummyPlugin {
    init: bool,
    graph_build: bool,
    after_run: bool,
}

impl DummyPlugin {
    fn new() -> Self {
        Self {
            init: false,
            graph_build: false,
            after_run: false,
        }
    }
}

impl Plugin for DummyPlugin {
    fn name(&self) -> &'static str {
        "DummyPlugin"
    }
    fn priority(&self) -> i32 {
        0
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn on_init(&mut self, _config: &PluginConfig) -> Result<(), BodoError> {
        self.init = true;
        Ok(())
    }
    fn on_graph_build(&mut self, _graph: &mut Graph) -> Result<(), BodoError> {
        self.graph_build = true;
        Ok(())
    }
    fn on_after_run(&mut self, _graph: &mut Graph) -> Result<(), BodoError> {
        self.after_run = true;
        Ok(())
    }
}

#[test]
fn test_plugin_manager_lifecycle() {
    let mut manager = PluginManager::new();
    let plugin = Box::new(DummyPlugin::new());
    manager.register(plugin);
    let mut graph = Graph::new();
    let config = PluginConfig::default();
    manager.run_lifecycle(&mut graph, Some(config)).unwrap();
    let dummy = manager.get_plugins()[0]
        .as_any()
        .downcast_ref::<DummyPlugin>()
        .unwrap();
    assert!(dummy.init);
    assert!(dummy.graph_build);
    assert!(dummy.after_run);
}

// Test ExecutionPlugin expand_env_vars edge cases.
#[test]
fn test_expand_env_vars_edge_cases() {
    let plugin = ExecutionPlugin::new();
    let env_map: HashMap<String, String> = vec![
        ("VAR".to_string(), "val".to_string()),
        ("EMPTY".to_string(), "".to_string()),
    ]
    .into_iter()
    .collect();
    let input = "echo $UNSET";
    assert_eq!(plugin.expand_env_vars(input, &env_map), "echo $UNSET");

    let input = "echo $$ and $VAR";
    assert_eq!(plugin.expand_env_vars(input, &env_map), "echo $ and val");

    let input = "echo ${VAR}";
    assert_eq!(plugin.expand_env_vars(input, &env_map), "echo val");

    let input = "hello $VAR, ${EMPTY}, end";
    assert_eq!(
        plugin.expand_env_vars(input, &env_map),
        "hello val, ${EMPTY}, end"
    );
}

// Test TimeoutPlugin parse_timeout.
#[test]
fn test_timeout_plugin_parse_timeout_valid() {
    let secs = TimeoutPlugin::parse_timeout("45s").unwrap();
    assert_eq!(secs, 45);
}

#[test]
fn test_timeout_plugin_parse_timeout_invalid() {
    let res = TimeoutPlugin::parse_timeout("bad");
    assert!(res.is_err());
}

// Test TimeoutPlugin on_graph_build does not set timeout_seconds if not provided.
#[test]
fn test_timeout_plugin_no_timeout() {
    let mut plugin = TimeoutPlugin::new();
    let mut graph = Graph::new();
    let node_id = graph.add_node(NodeKind::Task(TaskData {
        name: "no_timeout".to_string(),
        description: Some("No timeout set".to_string()),
        command: Some("echo no timeout".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
    }));
    plugin.on_graph_build(&mut graph).unwrap();
    let node = &graph.nodes[node_id as usize];
    assert!(node.metadata.get("timeout_seconds").is_none());
}
