use std::any::Any;
use std::collections::HashMap;

use crate::{
    errors::{BodoError, Result},
    graph::{Graph, NodeKind},
    plugin::{DryRun, DryRunReport, Plugin, PluginConfig, SimulatedAction},
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

        #[allow(clippy::type_complexity)]
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

impl DryRun for ExecutionPlugin {
    fn dry_run_simulate(&self, graph: &Graph, _config: &PluginConfig) -> Result<DryRunReport> {
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
        let mut simulated_actions = Vec::new();
        let mut dependencies = Vec::new();
        let mut warnings = Vec::new();

        #[allow(clippy::too_many_arguments)]
        #[allow(clippy::type_complexity)]
        #[allow(clippy::type_complexity)]
        fn simulate_node(
            node_id: usize,
            graph: &Graph,
            visited: &mut std::collections::HashSet<usize>,
            simulated_actions: &mut Vec<SimulatedAction>,
            dependencies: &mut Vec<String>,
            warnings: &mut Vec<String>,
            expand_env_vars_fn: &dyn Fn(&str, &std::collections::HashMap<String, String>) -> String,
            #[allow(clippy::only_used_in_recursion)] _get_prefix_settings_fn: &dyn Fn(
                &crate::graph::Node,
            )
                -> (
                bool,
                Option<String>,
                Option<String>,
            ),
        ) -> Result<()> {
            if visited.contains(&node_id) {
                return Ok(());
            }
            visited.insert(node_id);

            let node = &graph.nodes[node_id];
            match &node.kind {
                NodeKind::Task(task_data) => {
                    // Add task as dependency
                    dependencies.push(task_data.name.clone());

                    // Run pre dependencies
                    for edge in &graph.edges {
                        if edge.to == node_id as u64 {
                            simulate_node(
                                edge.from as usize,
                                graph,
                                visited,
                                simulated_actions,
                                dependencies,
                                warnings,
                                expand_env_vars_fn,
                                _get_prefix_settings_fn,
                            )?;
                        }
                    }
                    // Simulate task command execution
                    if let Some(cmd) = &task_data.command {
                        let expanded_cmd = expand_env_vars_fn(cmd, &task_data.env);

                        // Check for unresolved environment variables
                        if expanded_cmd.contains("${")
                            || expanded_cmd.contains("$") && expanded_cmd.contains(" ")
                        {
                            warnings.push(format!(
                                "Task '{}' may have unresolved environment variables",
                                task_data.name
                            ));
                        }

                        let mut details = std::collections::HashMap::new();
                        details.insert("command".to_string(), expanded_cmd.clone());
                        if let Some(cwd) = &task_data.working_dir {
                            details.insert("working_directory".to_string(), cwd.clone());
                        }

                        simulated_actions.push(SimulatedAction {
                            action_type: "command".to_string(),
                            description: format!("Execute task '{}'", task_data.name),
                            details,
                            node_id: Some(node_id),
                        });
                    }
                }
                NodeKind::Command(cmd_data) => {
                    let expanded_cmd = expand_env_vars_fn(&cmd_data.raw_command, &cmd_data.env);

                    // Check for unresolved environment variables
                    if expanded_cmd.contains("${")
                        || expanded_cmd.contains("$") && expanded_cmd.contains(" ")
                    {
                        warnings
                            .push("Command may have unresolved environment variables".to_string());
                    }

                    let mut details = std::collections::HashMap::new();
                    details.insert("command".to_string(), expanded_cmd.clone());
                    if let Some(cwd) = &cmd_data.working_dir {
                        details.insert("working_directory".to_string(), cwd.clone());
                    }

                    simulated_actions.push(SimulatedAction {
                        action_type: "command".to_string(),
                        description: "Execute command".to_string(),
                        details,
                        node_id: Some(node_id),
                    });
                }
                NodeKind::ConcurrentGroup(group_data) => {
                    // Simulate concurrent group execution
                    for &child_id in &group_data.child_nodes {
                        let child_node = &graph.nodes[child_id as usize];
                        match &child_node.kind {
                            NodeKind::Task(task_data) => {
                                dependencies.push(task_data.name.clone());

                                if let Some(cmd) = &task_data.command {
                                    let expanded_cmd = expand_env_vars_fn(cmd, &task_data.env);

                                    // Check for unresolved environment variables
                                    if expanded_cmd.contains("${")
                                        || expanded_cmd.contains("$") && expanded_cmd.contains(" ")
                                    {
                                        warnings.push(format!("Concurrent task '{}' may have unresolved environment variables", task_data.name));
                                    }

                                    let mut details = std::collections::HashMap::new();
                                    details.insert("command".to_string(), expanded_cmd.clone());
                                    if let Some(cwd) = &task_data.working_dir {
                                        details
                                            .insert("working_directory".to_string(), cwd.clone());
                                    }
                                    details.insert(
                                        "execution_mode".to_string(),
                                        "concurrent".to_string(),
                                    );

                                    simulated_actions.push(SimulatedAction {
                                        action_type: "command".to_string(),
                                        description: format!(
                                            "Execute concurrent task '{}'",
                                            task_data.name
                                        ),
                                        details,
                                        node_id: Some(child_id as usize),
                                    });
                                }
                            }
                            NodeKind::Command(cmd_data) => {
                                let expanded_cmd =
                                    expand_env_vars_fn(&cmd_data.raw_command, &cmd_data.env);

                                // Check for unresolved environment variables
                                if expanded_cmd.contains("${")
                                    || expanded_cmd.contains("$") && expanded_cmd.contains(" ")
                                {
                                    warnings.push("Concurrent command may have unresolved environment variables".to_string());
                                }

                                let mut details = std::collections::HashMap::new();
                                details.insert("command".to_string(), expanded_cmd.clone());
                                if let Some(cwd) = &cmd_data.working_dir {
                                    details.insert("working_directory".to_string(), cwd.clone());
                                }
                                details
                                    .insert("execution_mode".to_string(), "concurrent".to_string());

                                simulated_actions.push(SimulatedAction {
                                    action_type: "command".to_string(),
                                    description: "Execute concurrent command".to_string(),
                                    details,
                                    node_id: Some(child_id as usize),
                                });
                            }
                            _ => {}
                        }
                    }
                }
            }
            Ok(())
        }

        simulate_node(
            task_id as usize,
            graph,
            &mut visited,
            &mut simulated_actions,
            &mut dependencies,
            &mut warnings,
            &|cmd, env| self.expand_env_vars(cmd, env),
            &|node| self.get_prefix_settings(node),
        )?;

        let mut metadata = std::collections::HashMap::new();
        metadata.insert(
            "total_actions".to_string(),
            simulated_actions.len().to_string(),
        );
        metadata.insert(
            "total_dependencies".to_string(),
            dependencies.len().to_string(),
        );

        Ok(DryRunReport {
            plugin_name: "ExecutionPlugin".to_string(),
            simulated_actions,
            dependencies,
            warnings,
            metadata,
        })
    }
}
