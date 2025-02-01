use crate::{
    errors::BodoError,
    graph::{CommandData, ConcurrentGroupData, Graph, NodeKind, TaskData},
    plugin::Plugin,
    process::ProcessManager,
    Result,
};
use async_trait::async_trait;
use std::{any::Any, future::Future, pin::Pin};

pub struct ExecutionPlugin;

impl ExecutionPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ExecutionPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for ExecutionPlugin {
    fn name(&self) -> &'static str {
        "ExecutionPlugin"
    }

    fn priority(&self) -> i32 {
        95 // Lower than ConcurrentPlugin (100)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    async fn on_graph_build(&mut self, graph: &mut Graph) -> Result<()> {
        execute_graph(graph).await
    }
}

/// Execute a single task node
async fn run_single_task(task_data: &TaskData) -> Result<()> {
    if let Some(cmd) = &task_data.command {
        let mut pm = ProcessManager::new(false); // Single task, no fail-fast needed
        pm.spawn_command(&task_data.name, cmd, None)?;
        pm.run_concurrently()?;
    }
    Ok(())
}

/// Execute a single command node
async fn run_single_command(cmd_data: &CommandData, name: &str) -> Result<()> {
    let mut pm = ProcessManager::new(false);
    pm.spawn_command(name, &cmd_data.raw_command, None)?;
    pm.run_concurrently()?;
    Ok(())
}

/// Execute a concurrent group node
fn run_concurrent_group<'a>(
    group_data: &'a ConcurrentGroupData,
    graph: &'a Graph,
) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
    Box::pin(async move {
        // Create a process manager with the group's fail-fast setting
        let mut pm = ProcessManager::new(group_data.fail_fast);

        // If we have a concurrency limit, we need to chunk the commands
        let max_concurrent = group_data.max_concurrent.unwrap_or(usize::MAX);
        if max_concurrent == 0 {
            return Err(BodoError::PluginError(
                "max_concurrent=0 is not valid".to_string(),
            ));
        }

        let mut running_chunk = vec![];
        for &child_id in &group_data.child_nodes {
            let child_node = &graph.nodes[child_id as usize];

            // If we've hit the concurrency limit, wait for current chunk to finish
            if running_chunk.len() >= max_concurrent {
                pm.run_concurrently()?;
                running_chunk.clear();
                pm = ProcessManager::new(group_data.fail_fast);
            }

            // Spawn the child command based on its type
            match &child_node.kind {
                NodeKind::Task(task) => {
                    if let Some(cmd) = &task.command {
                        pm.spawn_command(&task.name, cmd, None)?;
                        running_chunk.push(task.name.clone());
                    }
                }
                NodeKind::Command(cmd) => {
                    let name = format!("cmd-{}", child_id);
                    pm.spawn_command(&name, &cmd.raw_command, None)?;
                    running_chunk.push(name);
                }
                NodeKind::ConcurrentGroup(sub_group) => {
                    // Handle nested concurrency by running it synchronously within this chunk
                    run_concurrent_group(sub_group, graph).await?;
                }
            }
        }

        // Wait for any remaining commands in the last chunk
        if !running_chunk.is_empty() {
            pm.run_concurrently()?;
        }

        Ok(())
    })
}

pub async fn execute_graph(graph: &mut Graph) -> Result<()> {
    // Get nodes in topological order to respect dependencies
    let sorted = graph.topological_sort()?;

    for node_id in sorted {
        let node = &graph.nodes[node_id as usize];
        match &node.kind {
            NodeKind::Task(task_data) => {
                run_single_task(task_data).await?;
            }
            NodeKind::Command(cmd_data) => {
                let name = format!("cmd-{}", node_id);
                run_single_command(cmd_data, &name).await?;
            }
            NodeKind::ConcurrentGroup(group_data) => {
                run_concurrent_group(group_data, graph).await?;
            }
        }
    }

    Ok(())
}
