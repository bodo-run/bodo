use bodo::{
    config::BodoConfig, manager::GraphManager, plugins::print_list_plugin::PrintListPlugin, Result,
};
use clap::Parser;
use std::collections::HashMap;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// List all available tasks
    #[arg(short, long)]
    list: bool,

    /// Task to run (defaults to default_task)
    task: Option<String>,

    /// Additional arguments passed to the task
    #[arg(last = true)]
    args: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Load configuration
    let config = BodoConfig {
        root_script: Some("scripts/basic.yaml".into()),
        scripts_dirs: Some(vec!["scripts/".into()]),
        tasks: HashMap::new(),
    };

    let mut graph_manager = GraphManager::new();
    graph_manager.build_graph(config).await?;

    if args.list {
        graph_manager.register_plugin(Box::new(PrintListPlugin));
        graph_manager.run_plugins(None).await?;
        return Ok(());
    }

    // Run specified task
    let task_name = args.task.unwrap_or_else(|| "default".to_string());
    graph_manager.run_task(&task_name).await?;

    Ok(())
}
