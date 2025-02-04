use crate::errors::{BodoError, Result};
use crate::manager::GraphManager;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// List all available tasks
    #[arg(short, long)]
    pub list: bool,

    /// Watch mode – rerun task on file changes
    #[arg(short, long)]
    pub watch: bool,

    /// Auto watch mode – if tasks specify auto_watch, enable it even if --watch was not passed
    #[arg(long)]
    pub auto_watch: bool,

    /// Disable watch mode completely, even if tasks define auto_watch.
    #[arg(long, default_value_t = false)]
    pub no_watch: bool,

    /// Enable debug logs
    #[arg(long)]
    pub debug: bool,

    /// Task to run (defaults to default_task)
    pub task: Option<String>,

    /// Subtask to run (appended to the task name)
    pub subtask: Option<String>,

    /// Additional arguments passed to the task
    #[arg(last = true)]
    pub args: Vec<String>,
}

pub fn get_task_name(args: &Args, manager: &GraphManager) -> Result<String> {
    if let Some(task) = &args.task {
        let name = if let Some(subtask) = &args.subtask {
            format!("{} {}", task, subtask)
        } else {
            task.clone()
        };
        if manager.task_exists(&name) {
            Ok(name)
        } else {
            Err(BodoError::TaskNotFound(name))
        }
    } else if manager.task_exists("default") {
        Ok("default".to_string())
    } else {
        Err(BodoError::NoTaskSpecified)
    }
}
