use async_trait::async_trait;
use std::any::Any;
use std::future::Future;
use std::pin::Pin;

use crate::{
    errors::{BodoError, Result},
    graph::{CommandData, ConcurrentGroupData, Graph, NodeId, NodeKind, TaskData},
    plugin::Plugin,
    process::ProcessManager,
};

/// ExecutionPlugin is the final step in the plugin chain:
/// it takes the fully built & transformed graph (topological order)
/// and actually runs the tasks/commands.
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

    /// Set a fairly low priority so it's called near the end,
    /// after concurrency, env, path, watch, etc.
    fn priority(&self) -> i32 {
        95
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    /// We do our "full run" during on_after_run, so that:
    /// - on_graph_build can finalize node transformations
    /// - watchers or other allocations can be set up in earlier phases
    /// - we only actually "run" at the last step in the lifecycle.
    async fn on_after_run(&mut self, graph: &mut Graph) -> Result<()> {
        // 1) Sort the graph topologically so we respect dependencies
        let sorted = graph.topological_sort()?;

        // 2) Walk each node in topological order
        for node_id in sorted {
            let node = &graph.nodes[node_id as usize];
            match &node.kind {
                NodeKind::Task(task_data) => {
                    run_single_task(task_data)?;
                }
                NodeKind::Command(cmd_data) => {
                    run_single_command(cmd_data, node_id)?;
                }
                NodeKind::ConcurrentGroup(group_data) => {
                    run_concurrent_group(group_data, graph).await?;
                }
            }
        }
        Ok(())
    }
}

/// Runs a single Task node by spawning a process with ProcessManager.
fn run_single_task(task_data: &TaskData) -> Result<()> {
    // If there's no `command`, there's nothing to run; skip
    if let Some(cmd) = &task_data.command {
        let mut pm = ProcessManager::new(false); // Single node => fail_fast false
        pm.spawn_command(&task_data.name, cmd)
            .map_err(|e| BodoError::PluginError(format!("{}", e)))?;
        pm.run_concurrently()
            .map_err(|e| BodoError::PluginError(format!("{}", e)))?;
    }
    Ok(())
}

/// Runs a single Command node by spawning a process with ProcessManager.
fn run_single_command(cmd_data: &CommandData, node_id: NodeId) -> Result<()> {
    let mut pm = ProcessManager::new(false);
    // Give it a label like "cmd-0" or "cmd-5"
    let label = format!("cmd-{}", node_id);
    pm.spawn_command(&label, &cmd_data.raw_command)
        .map_err(|e| BodoError::PluginError(format!("{}", e)))?;
    pm.run_concurrently()
        .map_err(|e| BodoError::PluginError(format!("{}", e)))?;
    Ok(())
}

/// Runs a concurrency group by spawning all child nodes in parallel,
/// respecting the group's `fail_fast` setting and optional concurrency limit.
fn run_concurrent_group<'a>(
    group_data: &'a ConcurrentGroupData,
    graph: &'a Graph,
) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
    Box::pin(async move {
        let mut pm = ProcessManager::new(group_data.fail_fast);
        let max_concurrent = group_data.max_concurrent.unwrap_or(usize::MAX);

        let mut pending = vec![];
        for &child_id in &group_data.child_nodes {
            // If we've hit the concurrency limit, run the chunk now and reset
            if pending.len() >= max_concurrent {
                pm.run_concurrently()
                    .map_err(|e| BodoError::PluginError(format!("{}", e)))?;
                pm = ProcessManager::new(group_data.fail_fast);
                pending.clear();
            }

            let child_node = &graph.nodes[child_id as usize];
            match &child_node.kind {
                NodeKind::Task(t) => {
                    if let Some(cmd) = &t.command {
                        pm.spawn_command(&t.name, cmd)
                            .map_err(|e| BodoError::PluginError(format!("{}", e)))?;
                        pending.push(t.name.clone());
                    }
                }
                NodeKind::Command(cmd) => {
                    let label = format!("cmd-{}", child_node.id);
                    pm.spawn_command(&label, &cmd.raw_command)
                        .map_err(|e| BodoError::PluginError(format!("{}", e)))?;
                    pending.push(label);
                }
                NodeKind::ConcurrentGroup(sub_group) => {
                    // Nested concurrency group:
                    // run it immediately in a sub-block
                    run_concurrent_group(sub_group, graph).await?;
                }
            }
        }

        // Run any leftover chunk
        if !pending.is_empty() {
            pm.run_concurrently()
                .map_err(|e| BodoError::PluginError(format!("{}", e)))?;
        }

        Ok(())
    })
}
