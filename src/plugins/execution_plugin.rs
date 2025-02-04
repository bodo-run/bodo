use std::any::Any;
use std::collections::HashMap;

use crate::{
    errors::{BodoError, Result},
    graph::{Graph, NodeKind},
    plugin::{Plugin, PluginConfig},
    process::ProcessManager,
};

pub struct ExecutionPlugin {
    pub task_name: Option<String>,
}

impl Default for ExecutionPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionPlugin {
    pub fn new() -> Self {
        Self { task_name: None }
    }

    /// Changed the visibility of the method to `pub`
    pub fn get_prefix_settings(
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

    pub fn expand_env_vars(&self, cmd: &str, env: &HashMap<String, String>) -> String {
        let mut result = String::new();
        let mut chars = cmd.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '$' {
                // Handle environment variable
                if let Some(peek) = chars.peek() {
                    if *peek == '$' {
                        result.push('$');
                        chars.next();
                    } else if *peek == '{' {
                        // ${VAR}
                        chars.next(); // Consume '{'
                        let mut var_name = String::new();
                        while let Some(&ch) = chars.peek() {
                            if ch == '}' {
                                chars.next(); // Consume '}'
                                break;
                            } else {
                                var_name.push(ch);
                                chars.next();
                            }
                        }
                        if let Some(var_value) = env.get(&var_name) {
                            result.push_str(var_value);
                        } else {
                            // If the variable is not in env, keep it as is
                            result.push_str(&format!("${{{}}}", var_name));
                        }
                    } else {
                        // $VAR
                        let mut var_name = String::new();
                        while let Some(&ch) = chars.peek() {
                            if ch.is_alphanumeric() || ch == '_' {
                                var_name.push(ch);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                        if let Some(var_value) = env.get(&var_name) {
                            result.push_str(var_value);
                        } else {
                            // If the variable is not in env, keep it as is
                            result.push_str(&format!("${}", var_name));
                        }
                    }
                } else {
                    result.push('$');
                }
            } else {
                result.push(c);
            }
        }
        result
    }
}

impl Plugin for ExecutionPlugin {
    fn name(&self) -> &'static str {
        "ExecutionPlugin"
    }

    fn priority(&self) -> i32 {
        100
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn on_init(&mut self, config: &PluginConfig) -> Result<()> {
        if let Some(options) = &config.options {
            if let Some(task) = options.get("task").and_then(|v| v.as_str()) {
                self.task_name = Some(task.to_string());
            }
        }
        Ok(())
    }

    fn on_after_run(&mut self, graph: &mut Graph) -> Result<()> {
        let task_name = if let Some(name) = &self.task_name {
            name.clone()
        } else {
            return Err(BodoError::PluginError("No task specified".to_string()));
        };

        let task_id = *graph
            .task_registry
            .get(&task_name)
            .ok_or_else(|| BodoError::TaskNotFound(task_name.clone()))?;

        let mut visited = std::collections::HashSet::new();

        let mut pm = ProcessManager::new(true);

        fn run_node(
            node_id: usize,
            graph: &Graph,
            pm: &mut ProcessManager,
            visited: &mut std::collections::HashSet<usize>,
            expand_env_vars_fn: &dyn Fn(&str, &HashMap<String, String>) -> String,

            get_prefix_settings_fn: &dyn Fn(
                &crate::graph::Node,
            ) -> (bool, Option<String>, Option<String>),
        ) -> Result<()> {
            if visited.contains(&node_id) {
                return Ok(());
            }
            visited.insert(node_id);

            let node = &graph.nodes[node_id];
            match &node.kind {
                NodeKind::Task(task_data) => {
                    // Run pre dependencies
                    for edge in &graph.edges {
                        if edge.to == node_id as u64 {
                            run_node(
                                edge.from as usize,
                                graph,
                                pm,
                                visited,
                                expand_env_vars_fn,
                                get_prefix_settings_fn,
                            )?;
                        }
                    }
                    // Execute the task command
                    if let Some(cmd) = &task_data.command {
                        let expanded_cmd = expand_env_vars_fn(cmd, &task_data.env);
                        let (prefix_enabled, prefix_label, prefix_color) =
                            get_prefix_settings_fn(node);
                        pm.spawn_command(
                            &task_data.name,
                            &expanded_cmd,
                            prefix_enabled,
                            prefix_label,
                            prefix_color,
                            task_data.working_dir.as_deref(),
                        )?;
                    }
                }
                NodeKind::Command(cmd_data) => {
                    let expanded_cmd = expand_env_vars_fn(&cmd_data.raw_command, &cmd_data.env);
                    let (prefix_enabled, prefix_label, prefix_color) = get_prefix_settings_fn(node);
                    pm.spawn_command(
                        "command",
                        &expanded_cmd,
                        prefix_enabled,
                        prefix_label,
                        prefix_color,
                        cmd_data.working_dir.as_deref(),
                    )?;
                }
                NodeKind::ConcurrentGroup(group_data) => {
                    // Handle concurrent group execution
                    let mut group_pm = ProcessManager::new(group_data.fail_fast);
                    for &child_id in &group_data.child_nodes {
                        let child_node = &graph.nodes[child_id as usize];
                        match &child_node.kind {
                            NodeKind::Task(task_data) => {
                                if let Some(cmd) = &task_data.command {
                                    let expanded_cmd = expand_env_vars_fn(cmd, &task_data.env);
                                    let (prefix_enabled, prefix_label, prefix_color) =
                                        get_prefix_settings_fn(child_node);
                                    group_pm.spawn_command(
                                        &task_data.name,
                                        &expanded_cmd,
                                        prefix_enabled,
                                        prefix_label,
                                        prefix_color,
                                        task_data.working_dir.as_deref(),
                                    )?;
                                }
                            }
                            NodeKind::Command(cmd_data) => {
                                let expanded_cmd =
                                    expand_env_vars_fn(&cmd_data.raw_command, &cmd_data.env);
                                let (prefix_enabled, prefix_label, prefix_color) =
                                    get_prefix_settings_fn(child_node);
                                group_pm.spawn_command(
                                    "command",
                                    &expanded_cmd,
                                    prefix_enabled,
                                    prefix_label,
                                    prefix_color,
                                    cmd_data.working_dir.as_deref(),
                                )?;
                            }
                            _ => {}
                        }
                    }
                    group_pm.run_concurrently()?;
                }
            }
            Ok(())
        }

        run_node(
            task_id as usize,
            graph,
            &mut pm,
            &mut visited,
            &|cmd, env| self.expand_env_vars(cmd, env),
            &|node| self.get_prefix_settings(node),
        )?;

        pm.run_concurrently()?;

        Ok(())
    }
}
