use async_trait::async_trait;
use std::any::Any;
use std::time::Duration;
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
        50 // Lowest priority for core plugins
    }

    async fn on_before_run(&mut self, graph: &mut Graph) -> Result<()> {
        if graph.has_cycle() {
            return Err(BodoError::PluginError(
                "Cannot run with cycles in the graph".to_string(),
            ));
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Execute the graph in topological order, handling concurrent groups
pub async fn execute_graph(manager: &mut PluginManager, graph: &mut Graph) -> Result<()> {
    let order = graph.topological_sort()?;
    let mut done = vec![false; graph.nodes.len()];

    for node_id in order {
        let node_idx = node_id as usize;
        if done[node_idx] {
            continue;
        }

        manager.on_run_node(node_id, graph).await?;

        // Clone the node kind to avoid borrow issues
        let node_kind = graph.nodes[node_idx].kind.clone();
        match node_kind {
            NodeKind::Task(task_data) => {
                println!("Running task: {}", task_data.name);
                if let Some(cmd) = &task_data.command {
                    // Here we'd use tokio::process::Command to actually run the command
                    // For now we just print
                    println!("Would execute: {}", cmd);
                }
                done[node_idx] = true;
            }
            NodeKind::Command(cmd_data) => {
                println!("Running command: {}", cmd_data.raw_command);
                // Here we'd use tokio::process::Command
                done[node_idx] = true;
            }
            NodeKind::ConcurrentGroup(group_data) => {
                execute_concurrent_group(manager, graph, node_id, &group_data, &mut done).await?;
            }
        }
    }

    Ok(())
}

async fn execute_concurrent_group(
    _manager: &mut PluginManager,
    graph: &mut Graph,
    group_id: u64,
    group_data: &ConcurrentGroupData,
    done: &mut [bool],
) -> Result<()> {
    let children = &group_data.child_nodes;
    if children.is_empty() {
        done[group_id as usize] = true;
        return Ok(());
    }

    let (tx, mut rx) = mpsc::unbounded_channel::<Result<()>>();
    let mut handles = Vec::new();

    // Spawn tasks up to max_concurrent limit
    let max_concurrent = group_data.max_concurrent.unwrap_or(children.len());
    let mut active = 0;
    let mut next_child = 0;

    while active > 0 || next_child < children.len() {
        // Spawn new tasks if under limit
        while active < max_concurrent && next_child < children.len() {
            let child_id = children[next_child];
            let tx = tx.clone();
            let child_node = graph.nodes[child_id as usize].kind.clone();

            let handle = tokio::spawn(async move {
                match child_node {
                    NodeKind::Task(t) => {
                        println!("(Concurrent) Running task: {}", t.name);
                        if let Some(cmd) = t.command {
                            println!("Would execute: {}", cmd);
                        }
                        tx.send(Ok(())).unwrap_or_default();
                    }
                    NodeKind::Command(c) => {
                        println!("(Concurrent) Running command: {}", c.raw_command);
                        tx.send(Ok(())).unwrap_or_default();
                    }
                    NodeKind::ConcurrentGroup(_) => {
                        tx.send(Err(BodoError::PluginError(
                            "Nested concurrency not supported".to_string(),
                        )))
                        .unwrap_or_default();
                    }
                }
            });

            handles.push(handle);
            active += 1;
            next_child += 1;
        }

        // Wait for a task to complete
        if let Some(result) = rx.recv().await {
            active -= 1;
            match result {
                Ok(_) => {}
                Err(e) => {
                    if group_data.fail_fast {
                        // Cancel remaining tasks
                        for handle in handles {
                            handle.abort();
                        }
                        return Err(e);
                    }
                }
            }
        }
    }

    // Apply timeout if specified
    if let Some(timeout_secs) = group_data.timeout_secs {
        match timeout(
            Duration::from_secs(timeout_secs),
            futures::future::join_all(handles),
        )
        .await
        {
            Ok(_) => {}
            Err(_) => {
                return Err(BodoError::PluginError(format!(
                    "Concurrent group timed out after {} seconds",
                    timeout_secs
                )));
            }
        }
    } else {
        futures::future::join_all(handles).await;
    }

    done[group_id as usize] = true;
    Ok(())
}
