use bodo::plugin::{Plugin, PluginManager};

struct DummyPlugin {
    pub order: i32,
}

impl Plugin for DummyPlugin {
    fn name(&self) -> &'static str {
        "DummyPlugin"
    }
    fn priority(&self) -> i32 {
        self.order
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[test]
fn test_plugin_manager_ordering() {
    let mut manager = PluginManager::new();
    // Register three dummy plugins with distinct priority values.
    manager.register(Box::new(DummyPlugin { order: 10 }));
    manager.register(Box::new(DummyPlugin { order: 20 }));
    manager.register(Box::new(DummyPlugin { order: 15 }));
    manager.sort_plugins();
    let plugins = manager.get_plugins();
    // Expect the plugins sorted in descending order (highest priority first).
    let orders: Vec<_> = plugins.iter().map(|p| p.priority()).collect();
    assert_eq!(orders, vec![20, 15, 10]);
}
