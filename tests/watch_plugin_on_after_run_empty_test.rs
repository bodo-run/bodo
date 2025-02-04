use bodo::graph::Graph;
use bodo::plugin::Plugin;
use bodo::plugins::watch_plugin::WatchPlugin;
use std::error::Error;

#[test]
fn test_on_after_run_empty_when_no_watch_entries() -> Result<(), Box<dyn Error>> {
    // Create a WatchPlugin instance with watch_mode false (or true) but with no watch entries.
    let mut plugin = WatchPlugin::new(false, false);
    // Create an empty graph.
    let mut graph = Graph::new();
    // Call on_after_run; since there are no watch entries, it should return Ok immediately.
    plugin.on_after_run(&mut graph)?;
    Ok(())
}
