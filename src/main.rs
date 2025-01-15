use clap::{Parser, Subcommand};
use std::error::Error;

use bodo::{
    manager::GraphManager, plugin::PluginConfig, plugins::print_list_plugin::PrintListPlugin,
    script_loader::BodoConfig,
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
    println!("After load_bodo_config: {:?}", manager.config);

    // Set up config
    manager.config = BodoConfig {
        scripts_dir: Some("scripts".to_string()),
        scripts_glob: Some("script.yaml".to_string()),
    };
    println!("After config setup: {:?}", manager.config);

    // 2) Build graph from discovered scripts
    manager.build_graph().await?;
    println!("Graph nodes: {}", manager.graph.nodes.len());

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
