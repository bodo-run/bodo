use bodo::graph::Graph;
use bodo::plugin::Plugin;
use bodo::Result;

struct DummyOnRunPlugin {
    pub run_called: bool,
}

impl DummyOnRunPlugin {
    fn new() -> Self {
        Self { run_called: false }
    }
}

impl Plugin for DummyOnRunPlugin {
    fn name(&self) -> &'static str {
        "DummyOnRunPlugin"
    }
    fn priority(&self) -> i32 {
        5
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn on_run(&mut self, node_id: usize, _graph: &mut Graph) -> Result<()> {
        self.run_called = true;
        if node_id == 0 {
            return Err(bodo::errors::BodoError::PluginError(
                "node_id is zero".to_string(),
            ));
        }
        Ok(())
    }
}

#[test]
fn test_dummy_plugin_on_run() -> Result<()> {
    let mut plugin = DummyOnRunPlugin::new();
    let mut graph = Graph::new();
    // Create a dummy task node; its id will be non-zero.
    let _node_id = graph.add_node(bodo::graph::NodeKind::Task(bodo::graph::TaskData {
        name: "dummy".to_string(),
        description: None,
        command: Some("echo dummy".to_string()),
        working_dir: None,
        env: std::collections::HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "dummy".to_string(),
        script_display_name: "dummy".to_string(),
        watch: None,
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
    }));
    // Call on_run with node id 1 (the first added node has id 0, next one is 1)
    // For this test, we simulate calling on_run with id 1.
    plugin.on_run(1, &mut graph)?;
    assert!(plugin.run_called);
    Ok(())
}
