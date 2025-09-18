use crate::errors::BodoError;
use crate::manager::GraphManager;
use clap::Parser;
use std::fmt::Debug;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// List all available tasks
    #[arg(short, long)]
    pub list: bool,

    /// Watch mode - rerun task on file changes
    #[arg(short, long)]
    pub watch: bool,

    /// Auto watch mode - automatically enable watch if specified
    #[arg(long)]
    pub auto_watch: bool,

    /// Enable debug logs
    #[arg(long)]
    pub debug: bool,

    /// Task to run (defaults to default_task)
    pub task: Option<String>,

    /// Subtask to run
    pub subtask: Option<String>,

    /// Additional arguments passed to the task
    #[arg(last = true)]
    pub args: Vec<String>,

    /// Dry-run mode - simulate execution without running commands
    #[arg(long)]
    pub dry_run: bool,
}

pub fn get_task_name(args: &Args, graph_manager: &GraphManager) -> Result<String, BodoError> {
    let task_name = if let Some(task) = args.task.clone() {
        if let Some(subtask) = args.subtask.clone() {
            format!("{} {}", task, subtask)
        } else {
            task
        }
    } else {
        // Check for default task in the task registry
        if graph_manager.task_exists("default") {
            "default".to_string()
        } else {
            return Err(BodoError::NoTaskSpecified);
        }
    };

    if !graph_manager.task_exists(&task_name) {
        return Err(BodoError::TaskNotFound(task_name));
    }

    Ok(task_name)
}
