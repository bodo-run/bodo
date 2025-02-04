use bodo::graph::{Graph, NodeKind, TaskData};
use bodo::plugin::{Plugin, PluginConfig, PluginManager};
use std::collections::HashMap;

// Dummy plugin to exercise default implementations
struct DummyPlugin;

impl Plugin for DummyPlugin {
    fn name(&self) -> &'static str {
        "DummyPlugin"
    }
    fn priority(&self) -> i32 {
        10
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    // Use default implementations for on_init, on_graph_build, on_after_run, on_run.
}

#[test]
fn test_plugin_on_run_default() {
    // Test that default on_run returns Ok(())
    let mut plugin = DummyPlugin;
    let mut graph = Graph::new();
    let node_id = graph.add_node(NodeKind::Task(TaskData {
        name: "dummy".to_string(),
        description: None,
        command: Some("echo dummy".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "".to_string(),
        script_display_name: "".to_string(),
        watch: None,
    }));
    let res = plugin.on_run(node_id as usize, &mut graph);
    assert!(res.is_ok());
}

#[test]
fn test_plugin_manager_run_lifecycle_default_config() {
    let mut pm = PluginManager::new();
    pm.register(Box::new(DummyPlugin));
    let mut graph = Graph::new();
    let res = pm.run_lifecycle(&mut graph, None);
    assert!(res.is_ok());
}

#[test]
fn test_set_default_paths_in_path_plugin() {
    use bodo::plugins::path_plugin::PathPlugin;
    let mut pp = PathPlugin::new();
    let paths = vec!["/bin".to_string(), "/usr/bin".to_string()];
    pp.set_default_paths(paths.clone());
    assert_eq!(pp.get_default_paths(), &paths);
}

#[test]
fn test_set_preserve_path_in_path_plugin() {
    use bodo::plugins::path_plugin::PathPlugin;
    let mut pp = PathPlugin::new();
    pp.set_preserve_path(true);
    assert!(pp.get_preserve_path());
    pp.set_preserve_path(false);
    assert!(!pp.get_preserve_path());
}

#[test]
fn test_prefix_plugin_as_any() {
    use bodo::plugins::prefix_plugin::PrefixPlugin;
    let pp = PrefixPlugin::new();
    let any_ref = pp.as_any();
    // Check it is of type PrefixPlugin
    assert!(any_ref.is::<PrefixPlugin>());
}

#[test]
fn test_env_plugin_as_any() {
    use bodo::plugins::env_plugin::EnvPlugin;
    let ep = EnvPlugin::new();
    let any_ref = ep.as_any();
    assert!(any_ref.is::<EnvPlugin>());
}

#[test]
fn test_timeout_plugin_as_any() {
    use bodo::plugins::timeout_plugin::TimeoutPlugin;
    let tp = TimeoutPlugin::new();
    let any_ref = tp.as_any();
    assert!(any_ref.is::<TimeoutPlugin>());
}

#[test]
fn test_print_list_plugin_as_any() {
    use bodo::plugins::print_list_plugin::PrintListPlugin;
    let plp = PrintListPlugin;
    let any_ref = plp.as_any();
    // Check its name via the trait method.
    assert_eq!(plp.name(), "PrintListPlugin");
    // We cannot further check inner type but this ensures as_any works.
}

#[test]
fn test_execution_plugin_as_any() {
    use bodo::plugins::execution_plugin::ExecutionPlugin;
    let ep = ExecutionPlugin::new();
    let any_ref = ep.as_any();
    assert!(any_ref.is::<ExecutionPlugin>());
}

#[test]
fn test_concurrent_plugin_as_any() {
    use bodo::plugins::concurrent_plugin::ConcurrentPlugin;
    let cp = ConcurrentPlugin::new();
    let any_ref = cp.as_any();
    assert!(any_ref.is::<ConcurrentPlugin>());
}

#[test]
fn test_plugin_config_defaults() {
    use bodo::plugin::PluginConfig;
    let pc = PluginConfig::default();
    assert!(!pc.fail_fast);
    assert!(!pc.watch);
    assert!(!pc.list);
    assert!(pc.options.is_none());
}

#[test]
fn test_on_init_for_plugins_default_behavior() {
    // Test that calling on_init on default implementations does not error.
    let mut graph = Graph::new();
    // Create a dummy plugin that does nothing special.
    struct NoOpPlugin;
    impl Plugin for NoOpPlugin {
        fn name(&self) -> &'static str {
            "NoOpPlugin"
        }
        fn priority(&self) -> i32 {
            0
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }
    let mut plugin = NoOpPlugin;
    let config = PluginConfig::default();
    assert!(plugin.on_init(&config).is_ok());
    assert!(plugin.on_graph_build(&mut graph).is_ok());
    assert!(plugin.on_after_run(&mut graph).is_ok());
}
