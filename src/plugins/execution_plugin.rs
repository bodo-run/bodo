use async_trait::async_trait;
use std::any::Any;
use std::collections::HashSet;
use std::future::Future;
use std::pin::Pin;

use crate::{
    errors::{BodoError, Result},
    graph::{CommandData, ConcurrentGroupData, Graph, NodeId, NodeKind, TaskData},
    plugin::{Plugin, PluginConfig},
    process::ProcessManager,
};

/// ExecutionPlugin is the final step in the plugin chain:
/// it takes the fully built & transformed graph (topological order)
/// and actually runs the tasks/commands.
pub struct ExecutionPlugin {
    task_name: Option<String>,
}

impl ExecutionPlugin {
    pub fn new() -> Self {
        Self { task_name: None }
    }

    /// Helper function to find all dependencies of a given node (including the node itself)
    fn get_dependency_subgraph(&self, graph: &Graph, start_node: NodeId) -> HashSet<NodeId> {
        let mut deps = HashSet::new();
        let mut stack = vec![start_node];

        while let Some(node_id) = stack.pop() {
            if deps.insert(node_id) {
                // For each edge that points TO this node, add its source
                for edge in &graph.edges {
                    if edge.to == node_id {
                        stack.push(edge.from);
                    }
                }
            }
        }

        deps
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

    async fn on_init(&mut self, config: &PluginConfig) -> Result<()> {
        if let Some(options) = &config.options {
            if let Some(task) = options.get("task") {
                if let Some(task_name) = task.as_str() {
                    self.task_name = Some(task_name.to_string());
                }
            }
        }
        Ok(())
    }

    /// We do our "full run" during on_after_run, so that:
    /// - on_graph_build can finalize node transformations
    /// - watchers or other allocations can be set up in earlier phases
    /// - we only actually "run" at the last step in the lifecycle.
    async fn on_after_run(&mut self, graph: &mut Graph) -> Result<()> {
        // Get the requested task from stored task_name
        let task_name = self
            .task_name
            .as_deref()
            .ok_or_else(|| BodoError::PluginError("No task specified for execution".to_string()))?;

        // Find the task node
        let task_node_id = *graph
            .task_registry
            .get(task_name)
            .ok_or_else(|| BodoError::TaskNotFound(task_name.to_string()))?;

        // Get all dependencies of this task
        let deps = self.get_dependency_subgraph(graph, task_node_id);

        // 1) Sort the graph topologically so we respect dependencies
        let sorted = graph.topological_sort()?;

        // 2) Walk each node in topological order, but only process nodes in our dependency subgraph
        for node_id in sorted {
            if !deps.contains(&node_id) {
                continue;
            }

            let node = &graph.nodes[node_id as usize];

            // Skip children that were marked by the concurrency plugin
            if node.metadata.get("skip_main_pass") == Some(&"true".to_string()) {
                continue;
            }

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
