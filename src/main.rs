use bodo::GraphManager;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut manager = GraphManager::new();

    // Load config if "bodo.toml" exists
    manager.load_bodo_config(None)?;

    // Build graph from scripts/
    manager.build_graph()?;

    // Print final graph
    manager.debug_graph();

    Ok(())
}
