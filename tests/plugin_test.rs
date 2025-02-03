use bodo::graph::Graph;
use bodo::plugin::{Plugin, PluginConfig, PluginManager};
use bodo::Result;

struct TestPlugin {
    pub init_called: bool,
    pub build_called: bool,
    pub after_run_called: bool,
}

impl Plugin for TestPlugin {
    fn name(&self) -> &'static str {
        "TestPlugin"
    }

    fn priority(&self) -> i32 {
        0
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn on_init(&mut self, _config: &PluginConfig) -> Result<()> {
        self.init_called = true;
        Ok(())
    }

    fn on_graph_build(&mut self, _graph: &mut Graph) -> Result<()> {
        self.build_called = true;
        Ok(())
    }

    fn on_after_run(&mut self, _graph: &mut Graph) -> Result<()> {
        self.after_run_called = true;
        Ok(())
    }
}

#[test]
fn test_plugin_manager() {
    let mut manager = PluginManager::new();
    let plugin = Box::new(TestPlugin {
        init_called: false,
        build_called: false,
        after_run_called: false,
    });
    manager.register(plugin);
    let mut graph = Graph::new();
    manager.run_lifecycle(&mut graph, None).unwrap();

    let plugin = manager.plugins[0]
        .as_any()
        .downcast_ref::<TestPlugin>()
        .unwrap();
    assert!(plugin.init_called);
    assert!(plugin.build_called);
    assert!(plugin.after_run_called);
}
