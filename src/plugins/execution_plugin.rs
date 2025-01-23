use crate::{
    errors::BodoError,
    graph::{Graph, NodeKind},
    plugin::{Plugin, PluginConfig},
    Result,
};
use async_trait::async_trait;
use std::any::Any;
use std::process::ExitStatus;
use tokio::time::timeout;

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

    async fn on_graph_build(&mut self, graph: &mut Graph) -> Result<()> {
        for node in &graph.nodes {
            match &node.kind {
                NodeKind::ConcurrentGroup(group) => {
                    let mut handles = Vec::new();
                    let mut failed = false;

                    for task_id in &group.child_nodes {
                        let task_node = &graph.nodes[*task_id as usize];
                        if let NodeKind::Task(task_data) = &task_node.kind {
                            let command = task_data.command.clone();
                            let handle = tokio::spawn(async move {
                                if let Some(cmd) = command {
                                    let mut command = tokio::process::Command::new("sh");
                                    command.arg("-c").arg(cmd);
                                    command.stdout(std::process::Stdio::inherit());
                                    command.stderr(std::process::Stdio::inherit());
                                    command.status().await
                                } else {
                                    Ok(std::process::ExitStatus::default())
                                }
                            });
                            handles.push(handle);
                        }
                    }

                    for handle in handles {
                        match handle.await {
                            Ok(Ok(status)) if status.success() => continue,
                            Ok(Ok(status)) => {
                                failed = true;
                                if group.fail_fast {
                                    return Err(BodoError::PluginError(format!(
                                        "Task failed with exit code {}",
                                        status.code().unwrap_or(1)
                                    )));
                                }
                            }
                            Ok(Err(e)) => {
                                failed = true;
                                if group.fail_fast {
                                    return Err(BodoError::PluginError(format!(
                                        "Task failed: {}",
                                        e
                                    )));
                                }
                            }
                            Err(e) => {
                                failed = true;
                                if group.fail_fast {
                                    return Err(BodoError::PluginError(format!(
                                        "Task failed: {}",
                                        e
                                    )));
                                }
                            }
                        }
                    }

                    if failed {
                        return Err(BodoError::PluginError(
                            "One or more tasks failed".to_string(),
                        ));
                    }
                }
                _ => continue,
            }
        }

        Ok(())
    }
}
