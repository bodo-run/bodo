use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "bodo")]
#[command(author = "Mohsen")]
#[command(version = "0.1.0")]
#[command(about = "Task runner in Rust", long_about = None)]
pub struct BodoCli {
    /// Task to run
    #[arg(index = 1, required = false)]
    pub task: Option<String>,

    /// Watch for changes
    #[arg(short, long)]
    pub watch: bool,

    /// Target environment
    #[arg(short, long)]
    pub target: Option<String>,

    /// List all available tasks with descriptions
    #[arg(short, long)]
    pub list: bool,

    /// Show verbose output including debug messages
    #[arg(short, long)]
    pub verbose: bool,

    /// Subtask arguments
    #[arg(index = 2, num_args = 0..)]
    pub args: Vec<String>,
}

impl BodoCli {
    pub fn new() -> Self {
        Self::parse()
    }
}

impl Default for BodoCli {
    fn default() -> Self {
        Self::new()
    }
}
