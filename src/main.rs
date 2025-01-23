use bodo::{
    config::BodoConfig, errors::BodoError, manager::GraphManager,
    plugins::print_list_plugin::PrintListPlugin,
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

    /// Subtask to run
    subtask: Option<String>,

    /// Additional arguments passed to the task
    #[arg(last = true)]
    args: Vec<String>,
}

#[tokio::main]
async fn main() {
    let result = async {
        let args = Args::parse();

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
            if let Some(subtask) = args.subtask {
                format!("{} {}", task, subtask)
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

        // Check if task exists
        if !graph_manager.task_exists(&task_name) {
            return Err(BodoError::TaskNotFound(task_name));
        }

        // Run the task
        graph_manager.run_task(&task_name).await?;

        Ok(())
    }
    .await;

    match result {
        Ok(_) => std::process::exit(0),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}
