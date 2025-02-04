use bodo::errors::BodoError;
use bodo::graph::Graph;
use bodo::plugin::Plugin;
use bodo::Result;

struct ErrorOnRunPlugin;

impl Plugin for ErrorOnRunPlugin {
    fn name(&self) -> &'static str {
        "ErrorOnRunPlugin"
    }
    fn priority(&self) -> i32 {
        0
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn on_run(&mut self, _node_id: usize, _graph: &mut Graph) -> Result<()> {
        Err(BodoError::PluginError("Error on run".to_string()))
    }
}

#[test]
fn test_on_run_error() {
    let mut plugin = ErrorOnRunPlugin;
    let mut graph = Graph::new();
    let result = plugin.on_run(0, &mut graph);
    assert!(result.is_err());
    if let Err(BodoError::PluginError(msg)) = result {
        assert_eq!(msg, "Error on run");
    } else {
        panic!("Expected PluginError");
    }
}
