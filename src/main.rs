use bodo::{
    config::BodoConfig, errors::BodoError, manager::GraphManager,
    plugins::print_list_plugin::PrintListPlugin, Result,
};
use clap::Parser;
use std::collections::HashMap;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// List all available tasks
    #[arg(short, long)]
    list: bool,

    /// Watch mode - rerun task on file changes
    #[arg(short, long)]
    watch: bool,

    /// Task to run (defaults to default_task)
    task: Option<String>,

    /// Additional arguments passed to the task
    #[arg(last = true)]
    args: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = Args::parse();

    // Load configuration
    let config = BodoConfig {
        root_script: None,
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

    // Parse task name and subtask
    let task_name = if let Some(task) = args.task {
        if task.contains('#') {
            task
        } else if !args.args.is_empty() {
            // Check if the first argument is a subtask
            let subtask = args.args[0].clone();
            if !subtask.starts_with('-') && !subtask.starts_with('/') {
                // Remove the subtask from args
                args.args.remove(0);
                // Try both formats to find the task
                let task_with_hash = format!("{}#{}", task, subtask);
                if graph_manager.task_exists(&task_with_hash) {
                    task_with_hash
                } else {
                    // If task not found, restore the arg and return original task
                    args.args.insert(0, subtask);
                    task
                }
            } else {
                task
            }
        } else {
            task
        }
    } else {
        // Check if default task exists
        if !graph_manager.task_exists("default") {
            return Err(BodoError::NoTaskSpecified);
        }
        "default".to_string()
    };

    // Run the task
    graph_manager.run_task(&task_name).await?;

    Ok(())
}
