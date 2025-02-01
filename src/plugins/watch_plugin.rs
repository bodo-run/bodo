use crate::{
    graph::{CommandData, ConcurrentGroupData, Graph, NodeKind, TaskData},
    plugin::Plugin,
    Result,
};
use async_trait::async_trait;
use std::any::Any;

pub struct WatchPlugin {}

impl Default for WatchPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl WatchPlugin {
    pub fn new() -> Self {
        Self {}
    }

    /// Converts a single task with timeout into a concurrent group containing:
    /// 1) The original task
    /// 2) A timeout command that fails after N seconds
    fn transform_task_into_timeout_group(
        &self,
        graph: &mut Graph,
        node_id: u64,
        timeout_sec: u64,
    ) -> Result<()> {
        // Get the original node
        let old_node = &graph.nodes[node_id as usize];
        let old_kind = old_node.kind.clone();
        let old_metadata = old_node.metadata.clone();

        // Extract task data
        let task_data = match old_kind {
            NodeKind::Task(t) => t,
            _ => return Ok(()), // Should never happen due to our filtering
        };

        // Create new task node with identical data
        let child_task = NodeKind::Task(TaskData {
            name: task_data.name.clone(),
            description: task_data.description.clone(),
            command: task_data.command.clone(),
            working_dir: task_data.working_dir.clone(),
            env: task_data.env.clone(),
            is_default: task_data.is_default,
            script_id: task_data.script_id.clone(),
            script_display_name: task_data.script_display_name.clone(),
            watch: task_data.watch.clone(),
        });
        let child_task_id = graph.add_node(child_task);

        // Create timeout command node
        let timeout_cmd = format!(
            "sleep {} && echo 'Task timeout exceeded after {}s' && exit 1",
            timeout_sec, timeout_sec
        );
        let child_timeout = NodeKind::Command(CommandData {
            raw_command: timeout_cmd,
            description: Some("Timeout enforcer".to_string()),
            working_dir: None,
            env: Default::default(),
            watch: None,
        });
        let child_timeout_id = graph.add_node(child_timeout);

        // Transform original node into a concurrent group
        let concurrency_kind = NodeKind::ConcurrentGroup(ConcurrentGroupData {
            child_nodes: vec![child_task_id, child_timeout_id],
            fail_fast: true, // Kill the task if timeout occurs
            max_concurrent: Some(2),
            timeout_secs: None, // We don't need an additional timeout on the group
        });

        // Update the original node in place
        let new_node = &mut graph.nodes[node_id as usize];
        new_node.kind = concurrency_kind;

        // Clear old metadata but keep some useful info
        new_node.metadata.clear();
        new_node
            .metadata
            .insert("timeout_group".to_string(), "true".to_string());

        // Add edges from group to children
        graph.add_edge(node_id, child_task_id)?;
        graph.add_edge(node_id, child_timeout_id)?;

        // Mark children to skip main execution pass
        let child_task_node = &mut graph.nodes[child_task_id as usize];
        child_task_node
            .metadata
            .insert("skip_main_pass".to_string(), "true".to_string());

        let child_timeout_node = &mut graph.nodes[child_timeout_id as usize];
        child_timeout_node
            .metadata
            .insert("skip_main_pass".to_string(), "true".to_string());

        Ok(())
    }
}

#[async_trait]
impl Plugin for WatchPlugin {
    fn name(&self) -> &'static str {
        "WatchPlugin"
    }

    fn priority(&self) -> i32 {
        90 // Lower than ExecutionPlugin (95)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    async fn on_graph_build(&mut self, graph: &mut Graph) -> Result<()> {
        // Store transformations to apply after iteration
        let mut timeout_transforms = Vec::new();

        // First pass: collect nodes that need timeout transformation
        for node in &graph.nodes {
            // Skip if already a concurrent group
            if let NodeKind::ConcurrentGroup(_) = node.kind {
                continue;
            }

            // Check if it's a Task with watch config
            let task_data = match &node.kind {
                NodeKind::Task(t) if t.watch.is_some() => t,
                _ => continue,
            };

            // Skip if it has explicit concurrency
            if node.metadata.get("concurrently").is_some() {
                continue;
            }

            // Check for timeout_seconds (set by TimeoutPlugin)
            if let Some(timeout_str) = node.metadata.get("timeout_seconds") {
                if let Ok(timeout_sec) = timeout_str.parse::<u64>() {
                    if timeout_sec > 0 {
                        timeout_transforms.push((node.id, timeout_sec));
                    }
                }
            }
        }

        // Second pass: apply transformations
        for (node_id, timeout_sec) in timeout_transforms {
            self.transform_task_into_timeout_group(graph, node_id, timeout_sec)?;
        }

        // Print watch info for tasks
        let watch_tasks: Vec<_> = graph
            .nodes
            .iter()
            .filter_map(|node| {
                if let NodeKind::Task(task) = &node.kind {
                    if task.watch.is_some() {
                        Some(task)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        for task in watch_tasks {
            println!(
                "Would watch task: {} with paths: {:?}",
                task.name, task.watch
            );
        }

        Ok(())
    }
}
