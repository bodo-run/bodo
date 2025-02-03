use bodo::{
    cli::{get_task_name, Args},
    config::BodoConfig,
    manager::GraphManager,
    plugin::PluginConfig,
    plugins::{
        concurrent_plugin::ConcurrentPlugin, env_plugin::EnvPlugin,
        execution_plugin::ExecutionPlugin, path_plugin::PathPlugin, prefix_plugin::PrefixPlugin,
        print_list_plugin::PrintListPlugin, timeout_plugin::TimeoutPlugin,
        watch_plugin::WatchPlugin,
    },
    BodoError,
};
use clap::Parser;
use log::{error, LevelFilter};
use std::io::Read;
use std::process::Command;
use std::process::Stdio;
use std::{collections::HashMap, path::PathBuf, process::exit};
use tempfile::tempdir;

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
}

pub fn get_task_name(args: &Args, graph_manager: &GraphManager) -> Result<String, BodoError> {
    let task_name = if let Some(task) = args.task.clone() {
        if let Some(subtask) = args.subtask.clone() {
            format!("{} {}", task, subtask)
        } else {
            task
        }
    } else {
        if !graph_manager.task_exists("default") {
            return Err(BodoError::NoTaskSpecified);
        }
        "default".to_string()
    };

    if !graph_manager.task_exists(&task_name) {
        return Err(BodoError::TaskNotFound(task_name));
    }

    Ok(task_name)
}

#[test]
fn test_bodo_default() {
    // Create a temporary directory
    let temp_dir = tempdir().unwrap();
    let scripts_dir = temp_dir.path().join("scripts");
    fs::create_dir(&scripts_dir).unwrap();

    // Write scripts/script.yaml
    let script_yaml = r#"
default_task:
  command: echo "Hello from Bodo root!"
  description: "Default greeting when running `bodo` with no arguments."

tasks:
  example:
    description: "An example task."
    command: echo "Example task"
"#;

    fs::write(scripts_dir.join("script.yaml"), script_yaml).unwrap();

    // Run 'bodo' with 'default' argument in temp_dir
    let bodo_executable = env!("CARGO_BIN_EXE_bodo");
    let mut child = Command::new(bodo_executable)
        .arg("default")
        .current_dir(temp_dir.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn 'bodo' process");

    // Capture stdout and stderr
    let mut stdout = String::new();
    let mut stderr = String::new();

    child
        .stdout
        .as_mut()
        .unwrap()
        .read_to_string(&mut stdout)
        .unwrap();
    child
        .stderr
        .as_mut()
        .unwrap()
        .read_to_string(&mut stderr)
        .unwrap();

    // Wait for the process to exit with a timeout
    let result = child.wait();
    match result {
        Ok(status) => {
            assert!(
                status.success(),
                "Command exited with non-zero status: {}",
                status
            );
        }
        Err(e) => panic!("Failed to wait on child process: {}", e),
    }

    // Check the output
    assert!(
        stdout.contains("Hello from Bodo root!"),
        "Output does not contain expected message 'Hello from Bodo root!'"
    );
}
