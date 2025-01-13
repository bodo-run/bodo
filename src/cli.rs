use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "bodo")]
#[command(author = "Mohsen")]
#[command(version = "0.1.0")]
#[command(about = "Task runner in Rust", long_about = None)]
pub struct BodoCli {
    /// Task to run
    #[arg(index = 1)]
    pub task: String,

    /// Watch for changes
    #[arg(short, long)]
    pub watch: bool,

    /// Target environment
    #[arg(short, long)]
    pub target: Option<String>,

    /// Subtask arguments
    #[arg(last = true)]
    pub args: Vec<String>,
}

impl BodoCli {
    pub fn new() -> Self {
        Self::parse()
    }
} 