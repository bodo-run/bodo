use bodo::graph::Graph;
use bodo::plugin::{Plugin, PluginConfig, PluginManager};
use bodo::Result;
use std::any::Any;

struct MockPlugin {
    initialized: bool,
}

impl Plugin for MockPlugin {
    fn name(&self) -> &'static str {
        "MockPlugin"
    }
    fn priority(&self) -> i32 {
        100
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn on_init(&mut self, _: &PluginConfig) -> Result<()> {
        self.initialized = true;
        Ok(())
    }
}

#[test]
fn test_plugin_lifecycle() -> Result<()> {
    let mut manager = PluginManager::new();
    manager.register(Box::new(MockPlugin { initialized: false }));

    let mut graph = Graph::new();
    manager.run_lifecycle(&mut graph, None)?;

    let mock_plugin = manager
        .plugins
        .iter_mut()
        .find_map(|p| p.as_any_mut().downcast_mut::<MockPlugin>())
        .expect("MockPlugin not found");

    assert!(mock_plugin.initialized);
    Ok(())
}
