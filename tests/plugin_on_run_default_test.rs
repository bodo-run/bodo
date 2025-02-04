use bodo::graph::Graph;
use bodo::plugin::Plugin;
use bodo::Result;

struct DefaultPlugin;

impl Plugin for DefaultPlugin {
    fn name(&self) -> &'static str {
        "DefaultPlugin"
    }
    fn priority(&self) -> i32 {
        0
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    // Use default implementations for on_init, on_graph_build, on_after_run, and on_run.
}

#[test]
fn test_on_run_default() -> Result<()> {
    let mut plugin = DefaultPlugin;
    let mut graph = Graph::new();
    // Default implementation for on_run should return Ok(())
    plugin.on_run(0, &mut graph)?;
    Ok(())
}
