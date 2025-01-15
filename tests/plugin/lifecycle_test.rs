use async_trait::async_trait;
use std::any::Any;
use std::sync::Arc;
use std::sync::Mutex;

use bodo::{
    errors::Result,
    graph::Graph,
    plugin::{Plugin, PluginConfig, PluginExecutionContext, PluginExecutionPhase},
};

#[derive(Default)]
struct LifecycleTestPlugin {
    events: Arc<Mutex<Vec<(PluginExecutionPhase, usize, Vec<String>)>>>,
}

impl LifecycleTestPlugin {
    fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn get_events(&self) -> Vec<(PluginExecutionPhase, usize, Vec<String>)> {
        self.events.lock().unwrap().clone()
    }
}

#[async_trait]
impl Plugin for LifecycleTestPlugin {
    fn name(&self) -> &'static str {
        "LifecycleTestPlugin"
    }

    async fn on_lifecycle_event(&mut self, ctx: &PluginExecutionContext<'_>) -> Result<()> {
        self.events.lock().unwrap().push((
            ctx.phase,
            ctx.current_plugin_index,
            ctx.all_plugin_names.to_vec(),
        ));
        Ok(())
    }

    async fn on_init(&mut self, _config: &PluginConfig) -> Result<()> {
        Ok(())
    }

    async fn on_graph_build(&mut self, _graph: &mut Graph) -> Result<()> {
        Ok(())
    }

    fn on_task_start(&mut self) {}

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[tokio::test]
async fn test_lifecycle_events() {
    use bodo::plugin::PluginManager;

    let mut manager = PluginManager::new();
    let plugin = Box::new(LifecycleTestPlugin::new());
    let events = plugin.events.clone();
    manager.register(plugin);

    // Run through the lifecycle
    manager
        .init_plugins(&PluginConfig::default())
        .await
        .unwrap();
    manager.on_graph_build(&mut Graph::new()).await.unwrap();
    manager.on_task_start();

    // Check the events were recorded in order
    let recorded_events = events.lock().unwrap();
    let expected_phases = vec![
        PluginExecutionPhase::InitStart,
        PluginExecutionPhase::InitEnd,
        PluginExecutionPhase::GraphBuildStart,
        PluginExecutionPhase::GraphBuildEnd,
        PluginExecutionPhase::TaskStartBegin,
        PluginExecutionPhase::TaskStartEnd,
    ];

    assert_eq!(recorded_events.len(), expected_phases.len());
    for (i, (phase, _, _)) in recorded_events.iter().enumerate() {
        assert_eq!(*phase, expected_phases[i]);
    }
}

#[tokio::test]
async fn test_plugin_order_awareness() {
    use bodo::plugin::PluginManager;

    let mut manager = PluginManager::new();

    // Register two plugins with different events trackers
    let first_plugin = Box::new(LifecycleTestPlugin::new());
    let first_events = first_plugin.events.clone();
    manager.register(first_plugin);

    let second_plugin = Box::new(LifecycleTestPlugin::new());
    let second_events = second_plugin.events.clone();
    manager.register(second_plugin);

    // Run init to trigger lifecycle events
    manager
        .init_plugins(&PluginConfig::default())
        .await
        .unwrap();

    // Check first plugin's events
    let first_events = first_events.lock().unwrap();
    assert!(!first_events.is_empty());
    for (_, idx, names) in first_events.iter() {
        assert_eq!(*idx, 0); // First plugin should always see itself at index 0
        assert_eq!(names.len(), 2); // Should see both plugins
    }

    // Check second plugin's events
    let second_events = second_events.lock().unwrap();
    assert!(!second_events.is_empty());
    for (_, idx, names) in second_events.iter() {
        assert_eq!(*idx, 1); // Second plugin should always see itself at index 1
        assert_eq!(names.len(), 2); // Should see both plugins
    }
}
