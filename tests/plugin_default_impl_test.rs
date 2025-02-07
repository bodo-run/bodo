extern crate bodo;
use bodo::graph::Graph;
use bodo::plugin::{Plugin, PluginConfig};
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
    // Use default implementations for on_init, on_graph_build, on_after_run and on_run.
}

#[test]
fn test_default_plugin_methods() -> Result<()> {
    let mut plugin = DefaultPlugin;
    let config = PluginConfig::default();
    let mut graph = Graph::new();

    // Default implementations should simply return Ok(())
    plugin.on_init(&config)?;
    plugin.on_graph_build(&mut graph)?;
    plugin.on_after_run(&mut graph)?;
    plugin.on_run(0, &mut graph)?;
    Ok(())
}
