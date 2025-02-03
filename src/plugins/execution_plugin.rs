// src/plugins/execution_plugin.rs
use colored::Colorize;
use log::debug;
use std::any::Any;
use terminal_size::terminal_size;

use crate::{
    errors::{BodoError, Result},
    graph::{ConcurrentGroupData, Graph, NodeId, NodeKind},
    plugin::{Plugin, PluginConfig},
    process::ProcessManager,
};

pub struct ExecutionPlugin {
    task_name: Option<String>,
}

impl ExecutionPlugin {
    pub fn new() -> Self {
        Self { task_name: None }
    }

    fn get_dependency_subgraph(
        &self,
        graph: &Graph,
        start_node: NodeId,
    ) -> std::collections::HashSet<NodeId> {
        let mut deps = std::collections::HashSet::new();
        let mut stack = vec![start_node];
        while let Some(node_id) = stack.pop() {
            if deps.insert(node_id) {
                for edge in &graph.edges {
                    if edge.to == node_id {
                        stack.push(edge.from);
                    }
                }
            }
        }
        deps
    }

    fn print_command(&self, cmd: &str) {
        let width = terminal_size().map_or(80, |size| size.0 .0 as usize);
        let max_length = width.saturating_sub(7);
        let cmd_line = cmd.lines().next().unwrap_or(cmd);
        let truncated = if cmd_line.len() > max_length {
            format!("{}...", &cmd_line[..max_length.saturating_sub(3)])
        } else {
            cmd_line.to_string()
        };
        println!("{} {}", "$".dimmed(), truncated.green());
    }

    fn get_prefix_settings(
        &self,
        node: &crate::graph::Node,
    ) -> (bool, Option<String>, Option<String>) {
        let prefix_enabled = node
            .metadata
            .get("prefix_enabled")
            .map(|v| v == "true")
            .unwrap_or(false);
        let prefix_label = node.metadata.get("prefix_label").cloned();
        let prefix_color = node.metadata.get("prefix_color").cloned();
        (prefix_enabled, prefix_label, prefix_color)
    }
}

impl Default for ExecutionPlugin {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple expansion of `$KEY` with the environment map's values. Not robust for all shells.
pub fn expand_env_vars(
    original: &str,
    env_map: &std::collections::HashMap<String, String>,
) -> String {
    let mut result = String::new();
    let mut chars = original.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '$' {
            if let Some(&next_ch) = chars.peek() {
                if next_ch == '$' {
                    // handle $$ as a literal $
                    result.push('$');
                    chars.next(); // consume the second '$'
                } else {
                    // attempt to read a variable name
                    let mut var_name = String::new();
                    while let Some(peek_ch) = chars.peek() {
                        if peek_ch.is_alphanumeric() || *peek_ch == '_' {
                            var_name.push(*peek_ch);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    if !var_name.is_empty() {
                        if let Some(val) = env_map.get(&var_name) {
                            result.push_str(val);
                        } else {
                            // no match in env, leave as is
                            result.push('$');
                            result.push_str(&var_name);
                        }
                    } else {
                        // '$' followed by non-alphanumeric, just output '$'
                        result.push('$');
                    }
                }
            } else {
                // '$' at end of string
                result.push('$');
            }
        } else {
            result.push(ch);
        }
    }
    result
}

impl Plugin for ExecutionPlugin {
    fn name(&self) -> &'static str {
        "ExecutionPlugin"
    }

    fn priority(&self) -> i32 {
        95
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn on_init(&mut self, config: &PluginConfig) -> Result<()> {
        if let Some(options) = &config.options {
            if let Some(task) = options.get("task") {
                if let Some(task_name) = task.as_str() {
                    debug!("Initializing ExecutionPlugin with task: {}", task_name);
                    self.task_name = Some(task_name.to_string());
                }
            }
        }
        Ok(())
    }

    fn on_after_run(&mut self, graph: &mut Graph) -> Result<()> {
        let task_name = self
            .task_name
            .as_deref()
            .ok_or_else(|| BodoError::PluginError("No task specified for execution".to_string()))?;

        debug!("Starting execution of task: {}", task_name);

        let task_node_id = *graph
            .task_registry
            .get(task_name)
            .ok_or_else(|| BodoError::TaskNotFound(task_name.to_string()))?;

        let deps = self.get_dependency_subgraph(graph, task_node_id);
        let sorted = graph.topological_sort()?;

        for node_id in sorted {
            if !deps.contains(&node_id) {
                continue;
            }
            let node = &graph.nodes[node_id as usize];
            if node.metadata.get("skip_main_pass") == Some(&"true".to_string()) {
                debug!("Skipping node {} (skip_main_pass=true)", node_id);
                continue;
            }
            match &node.kind {
                NodeKind::Task(task_data) => {
                    if let Some(cmd) = &task_data.command {
                        debug!("Executing task: {}", task_data.name);
                        let final_cmd = expand_env_vars(cmd, &task_data.env);
                        self.print_command(&final_cmd);
                        let mut pm = ProcessManager::new(false);
                        pm.spawn_command(
                            &task_data.name,
                            &final_cmd,
                            true,
                            None,
                            None,
                            task_data.working_dir.as_deref(),
                        )
                        .map_err(|e| BodoError::PluginError(format!("{}", e)))?;
                        pm.run_concurrently()
                            .map_err(|e| BodoError::PluginError(format!("{}", e)))?;
                    }
                }
                NodeKind::Command(cmd_data) => {
                    debug!("Executing command node: {}", node_id);
                    let final_cmd = expand_env_vars(&cmd_data.raw_command, &cmd_data.env);
                    self.print_command(&final_cmd);
                    let mut pm = ProcessManager::new(false);
                    let label = format!("cmd-{}", node_id);
                    pm.spawn_command(
                        &label,
                        &final_cmd,
                        true,
                        None,
                        None,
                        cmd_data.working_dir.as_deref(),
                    )
                    .map_err(|e| BodoError::PluginError(format!("{}", e)))?;
                    pm.run_concurrently()
                        .map_err(|e| BodoError::PluginError(format!("{}", e)))?;
                }
                NodeKind::ConcurrentGroup(group_data) => {
                    debug!("Executing concurrent group: {}", node_id);
                    run_concurrent_group(group_data, graph, self)?;
                }
            }
        }
        Ok(())
    }
}

fn run_concurrent_group(
    group_data: &ConcurrentGroupData,
    graph: &Graph,
    plugin: &ExecutionPlugin,
) -> Result<()> {
    let mut pm = ProcessManager::new(group_data.fail_fast);
    let max_concurrent = group_data.max_concurrent.unwrap_or(usize::MAX);

    debug!(
        "Running concurrent group with max_concurrent={}, fail_fast={}",
        max_concurrent, group_data.fail_fast
    );
    let mut pending = vec![];

    for &child_id in &group_data.child_nodes {
        if pending.len() >= max_concurrent {
            pm.run_concurrently()
                .map_err(|e| BodoError::PluginError(format!("{}", e)))?;
            pm = ProcessManager::new(group_data.fail_fast);
            pending.clear();
        }
        let child_node = &graph.nodes[child_id as usize];
        let (prefix_enabled, prefix_label, prefix_color) = plugin.get_prefix_settings(child_node);
        match &child_node.kind {
            NodeKind::Task(t) => {
                if let Some(cmd) = &t.command {
                    let final_cmd = expand_env_vars(cmd, &t.env);
                    pm.spawn_command(
                        &t.name,
                        &final_cmd,
                        prefix_enabled,
                        prefix_label.clone(),
                        prefix_color.clone(),
                        t.working_dir.as_deref(),
                    )
                    .map_err(|e| BodoError::PluginError(format!("{}", e)))?;
                    pending.push(t.name.clone());
                }
            }
            NodeKind::Command(cmd) => {
                let label = format!("cmd-{}", child_id);
                let final_cmd = expand_env_vars(&cmd.raw_command, &cmd.env);
                pm.spawn_command(
                    &label,
                    &final_cmd,
                    prefix_enabled,
                    prefix_label.clone(),
                    prefix_color.clone(),
                    cmd.working_dir.as_deref(),
                )
                .map_err(|e| BodoError::PluginError(format!("{}", e)))?;
                pending.push(label);
            }
            NodeKind::ConcurrentGroup(sub_group) => {
                run_concurrent_group(sub_group, graph, plugin)?;
            }
        }
    }

    if !pending.is_empty() {
        pm.run_concurrently()
            .map_err(|e| BodoError::PluginError(format!("{}", e)))?;
    }
    Ok(())
}
