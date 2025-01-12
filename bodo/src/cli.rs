use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "bodo")]
#[command(author = "Mohsen")]
#[command(version = "0.1.0")]
#[command(about = "Task runner in Rust", long_about = None)]
pub struct BodoCli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(short, long)]
    pub watch: bool,

    #[arg(short, long)]
    pub target: Option<String>,

    #[arg(index = 1)]
    pub task_group: Option<String>,

    #[arg(index = 2)]
    pub subtask: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize a new project
    Init {
        /// Project name
        #[arg(short, long)]
        name: String,
    },
    /// Run a command
    Run {
        /// Command name
        #[arg(short, long)]
        name: String,
    },
}

impl BodoCli {
    pub fn new() -> Self {
        Self::parse()
    }
} 