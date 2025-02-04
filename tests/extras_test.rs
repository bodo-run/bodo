use std::sync::Arc;

use bodo::manager::GraphManager;
use bodo::plugin::{Plugin, PluginConfig, PluginManager};
use bodo::process::ProcessManager;

// Test default PluginConfig values.
#[test]
fn test_plugin_config_defaults() {
    let config = PluginConfig::default();
    assert!(!config.fail_fast);
    assert!(!config.watch);
    assert!(!config.list);
    assert!(config.options.is_none());
}

// Dummy plugin to test PluginManager sorting.
struct DummyPlugin {
    priority_val: i32,
}

impl Plugin for DummyPlugin {
    fn name(&self) -> &'static str {
        "DummyPlugin"
    }
    fn priority(&self) -> i32 {
        self.priority_val
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[test]
fn test_plugin_manager_sort() {
    let mut manager = PluginManager::new();
    manager.register(Box::new(DummyPlugin { priority_val: 10 }));
    manager.register(Box::new(DummyPlugin { priority_val: 20 }));
    manager.sort_plugins();
    let plugins = manager.get_plugins();
    // The first plugin should have the higher priority.
    let p0 = plugins[0].priority();
    let p1 = plugins[1].priority();
    assert!(p0 >= p1);
}

// Test GraphManager initialize using the default config.
#[test]
fn test_graph_manager_initialize() {
    let mut manager = GraphManager::new();
    let result = manager.initialize();
    assert!(result.is_ok());
}

// Test ProcessManager kill_all with an empty children list.
#[test]
fn test_process_manager_kill_all_empty() {
    let mut pm = ProcessManager::new(false);
    let result = pm.kill_all();
    assert!(result.is_ok());
}

// A simple test to utilize Arc.
#[test]
fn test_arc_usage() {
    let a = Arc::new(5);
    let b = a.clone();
    assert_eq!(*a, 5);
    assert_eq!(*b, 5);
}

// Test Plugin on_run lifecycle method.
#[test]
fn test_plugin_on_run() {
    struct TestPluginInner;
    impl Plugin for TestPluginInner {
        fn name(&self) -> &'static str {
            "TestPluginInner"
        }
        fn priority(&self) -> i32 {
            0
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn on_run(&mut self, _node_id: usize, _graph: &mut bodo::graph::Graph) -> bodo::Result<()> {
            Ok(())
        }
    }
    let mut p = TestPluginInner;
    let mut graph = bodo::graph::Graph::new();
    p.on_run(0, &mut graph).unwrap();
}
