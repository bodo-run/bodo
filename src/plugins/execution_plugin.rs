use async_trait::async_trait;
use std::any::Any;
use tokio::{sync::mpsc, time::timeout};

use crate::{
    errors::BodoError,
    graph::{ConcurrentGroupData, Graph, NodeKind},
    plugin::{Plugin, PluginManager},
    Result,
};

pub struct ExecutionPlugin;

#[async_trait]
impl Plugin for ExecutionPlugin {
    fn name(&self) -> &'static str {
        "ExecutionPlugin"
    }

    fn priority(&self) -> i32 {
        50
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub async fn execute_graph(manager: &mut PluginManager, graph: &mut Graph) -> Result<()> {
    let (tx, mut rx) = mpsc::channel::<Result<()>>(32);
    let mut done = vec![false; graph.nodes.len()];

    while done.iter().any(|&x| !x) {
        for node_idx in 0..graph.nodes.len() {
            if done[node_idx] {
                continue;
            }

            let node_kind = graph.nodes[node_idx].kind.clone();
            match node_kind {
                NodeKind::Task(task_data) => {
                    if let Some(cmd) = &task_data.command {
                        let timeout_seconds = graph.nodes[node_idx]
                            .metadata
                            .get("timeout_seconds")
                            .and_then(|s| s.parse::<u64>().ok())
                            .map(tokio::time::Duration::from_secs);

                        let mut command = tokio::process::Command::new("sh");
                        command.arg("-c").arg(cmd);
                        command.stdout(std::process::Stdio::inherit());
                        command.stderr(std::process::Stdio::inherit());

                        let mut child = command.spawn().map_err(|e| {
                            BodoError::PluginError(format!("Failed to spawn command: {}", e))
                        })?;

                        let result = match timeout_seconds {
                            Some(duration) => match timeout(duration, child.wait()).await {
                                Ok(r) => r.map_err(|e| e.into()),
                                Err(_) => {
                                    let _ = child.kill().await;
                                    Err(BodoError::PluginError(format!(
                                        "Task '{}' timed out",
                                        task_data.name
                                    )))
                                }
                            },
                            None => child.wait().await.map_err(|e| e.into()),
                        };

                        match result {
                            Ok(status) if status.success() => {}
                            Ok(status) => {
                                return Err(BodoError::PluginError(format!(
                                    "Task '{}' failed with exit code {}",
                                    task_data.name,
                                    status.code().unwrap_or(1)
                                )));
                            }
                            Err(e) => return Err(e),
                        }
                    }
                    done[node_idx] = true;
                }
                NodeKind::Command(cmd_data) => {
                    let mut command = tokio::process::Command::new("sh");
                    command.arg("-c").arg(&cmd_data.raw_command);
                    command.stdout(std::process::Stdio::inherit());
                    command.stderr(std::process::Stdio::inherit());

                    let status = command.status().await.map_err(|e| {
                        BodoError::PluginError(format!("Failed to execute command: {}", e))
                    })?;

                    if !status.success() {
                        return Err(BodoError::PluginError(format!(
                            "Command failed with exit code {}",
                            status.code().unwrap_or(1)
                        )));
                    }
                    done[node_idx] = true;
                }
                NodeKind::ConcurrentGroup(ConcurrentGroupData {
                    child_nodes,
                    fail_fast,
                    max_concurrent,
                    timeout_secs,
                }) => {
                    let mut all_done = true;
                    for task_id in child_nodes {
                        if !done[task_id as usize] {
                            all_done = false;
                            break;
                        }
                    }
                    if all_done {
                        done[node_idx] = true;
                    }
                }
            }
        }
    }

    Ok(())
}
