use bodo::{
    CommandExecPlugin, ConcurrencyPlugin, EnvVarPlugin, GraphManager, ListPlugin, WatchPlugin,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = GraphManager::new();

    // Register plugins
    manager.register_plugin(Box::new(EnvVarPlugin::new()));
    manager.register_plugin(Box::new(ConcurrencyPlugin::new()));
    manager.register_plugin(Box::new(WatchPlugin::new()));
    manager.register_plugin(Box::new(CommandExecPlugin::new()));
    manager.register_plugin(Box::new(ListPlugin::new()));

    // Load bodo.toml if it exists
    manager.load_bodo_config::<&str>(None)?;

    // Initialize plugins
    let plugin_configs = vec![]; // load from YAML or other sources
    manager.init_plugins(&plugin_configs)?;

    // Build the graph
    manager.build_graph()?;

    // Execute the graph
    manager.execute()?;

    // Debug final graph
    manager.debug_graph();

    Ok(())
}
