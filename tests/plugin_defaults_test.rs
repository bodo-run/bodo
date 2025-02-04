use bodo::graph::Graph;
use bodo::plugin::{Plugin, PluginConfig};

struct DummyPlugin;

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
}

#[test]
fn test_dummy_plugin_defaults() {
    let mut plugin = DummyPlugin;
    let config = PluginConfig::default();
    let mut graph = Graph::new();
    // Default implementations should succeed.
    assert!(plugin.on_init(&config).is_ok());
    assert!(plugin.on_graph_build(&mut graph).is_ok());
    assert!(plugin.on_after_run(&mut graph).is_ok());
    assert!(plugin.on_run(0, &mut graph).is_ok());
}
