use clap::{Parser, Subcommand};
use std::error::Error;

use bodo::{
    config::BodoConfig, manager::GraphManager, plugin::PluginConfig,
    plugins::print_list_plugin::PrintListPlugin,
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Show list of available tasks
    #[arg(long, short)]
    list: bool,
}

#[derive(Subcommand)]
enum Commands {
    // Add other commands here as needed
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let mut manager = GraphManager::new();

    if cli.list {
        manager.register_plugin(Box::new(PrintListPlugin::new(true)));
    }

    // 1) Load config
    manager.load_bodo_config(None).await?;

    // TODO: if CLI args are provided, override config with them
    manager.config = BodoConfig::default();

    // 2) Build graph from discovered scripts
    manager.build_graph(manager.config.clone()).await?;

    // 3) Initialize plugins
    manager.init_plugins(Some(PluginConfig::default())).await?;

    // 4) Let the plugins transform the graph
    manager.apply_plugins_to_graph().await?;

    // If the user used --list, the plugin has printed it. Exit:
    if cli.list {
        std::process::exit(0);
    }

    match cli.command {
        None => {
            println!("No command specified");
        }
    }

    Ok(())
}
