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

    /// Disable watch mode completely, even if tasks define auto_watch.
    #[arg(long, default_value_t = false)]
    pub no_watch: bool,

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
}
