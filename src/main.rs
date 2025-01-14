use bodo::{
    CommandExecPlugin, ConcurrencyPlugin, EnvVarPlugin, GraphManager, ListPlugin, PathPlugin,
    PrefixPlugin, Result, WatchPlugin,
};
use clap::Parser;
use std::path::PathBuf;
use tracing::info;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Directory containing task scripts
    #[arg(short, long, default_value = "scripts")]
    scripts_dir: PathBuf,

    /// Maximum concurrent tasks
    #[arg(short, long, default_value_t = 4)]
    concurrency: usize,

    /// Enable watch mode
    #[arg(short, long)]
    watch: bool,

    /// Fail fast on first error
    #[arg(short, long)]
    fail_fast: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    info!("Starting bodo task runner");

    // Parse command line arguments
    let args = Args::parse();

    // Create and configure the graph manager
    let mut manager = GraphManager::new();

    // Register plugins
    manager.register_plugin(Box::new(EnvVarPlugin::new()));
    manager.register_plugin(Box::new(PathPlugin::new()));
    manager.register_plugin(Box::new(ConcurrencyPlugin::new(
        args.concurrency,
        args.fail_fast,
    )));
    manager.register_plugin(Box::new(PrefixPlugin::new()));
    manager.register_plugin(Box::new(CommandExecPlugin::new()));
    manager.register_plugin(Box::new(ListPlugin::new()));

    // Register watch plugin only if watch mode is enabled
    if args.watch {
        manager.register_plugin(Box::new(WatchPlugin::new()));
    }

    // Initialize plugins (no specific config for now)
    manager.init_plugins(&[]).await?;

    // Load scripts from the specified directory
    manager.load_scripts(&args.scripts_dir).await?;

    // Build the task graph
    manager.build_graph().await?;

    // Print the graph for debugging
    manager.debug_graph().await;

    // Execute the tasks
    manager.execute().await?;

    // Shut down plugins gracefully
    manager.shutdown().await?;

    info!("Task execution completed successfully");
    Ok(())
}
