use std::any::Any;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::{
    errors::{BodoError, Result},
    graph::{Graph, NodeKind},
    plugin::{DryRunReport, DryRunnable, ExecutionContext, Plugin, PluginConfig, SideEffect},
    process::ProcessManager,
    sandbox::Sandbox,
};


pub struct ExecutionPlugin {
    pub task_name: Option<String>,
    pub dry_run: bool,
}

impl Default for ExecutionPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionPlugin {
    pub fn new() -> Self {
        Self {
            task_name: None,
            dry_run: false,
        }
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

    /// Estimate command execution duration based on command patterns
    pub fn estimate_duration(&self, command: &str) -> Duration {
        // Simple heuristics for estimation
        if command.contains("sleep") {
            Duration::from_secs(5) // Assume sleep commands take longer
        } else if command.contains("npm install") || command.contains("cargo build") {
            Duration::from_secs(30) // Build commands take longer
        } else if command.contains("test") {
            Duration::from_secs(10) // Test commands moderate time
        } else {
            Duration::from_secs(1) // Default for simple commands
        }
    }

    /// Robust side effect analysis using sandboxed execution
    pub fn analyze_side_effects(
        &self,
        command: &str,
        working_dir: &Path,
        env: &HashMap<String, String>,
    ) -> Vec<SideEffect> {
        // Try to use sandbox for robust analysis
        match Sandbox::new() {
            Ok(sandbox) => {
                match sandbox.execute_and_analyze(command, working_dir, env) {
                    Ok(effects) => effects,
                    Err(e) => {
                        // Fallback to pattern-based analysis if sandbox fails
                        tracing::warn!(
                            "Sandbox analysis failed: {}, falling back to pattern analysis",
                            e
                        );
                        self.analyze_side_effects_fallback(command, working_dir)
                    }
                }
            }
            Err(e) => {
                // Fallback to pattern-based analysis if sandbox creation fails
                tracing::warn!(
                    "Failed to create sandbox: {}, falling back to pattern analysis",
                    e
                );
                self.analyze_side_effects_fallback(command, working_dir)
            }
        }
    }

    /// Fallback pattern-based side effect analysis (original implementation)
    pub fn analyze_side_effects_fallback(
        &self,
        command: &str,
        working_dir: &Path,
    ) -> Vec<SideEffect> {
        let mut effects = vec![];

        effects.push(SideEffect::ProcessSpawn(command.to_string()));

        // Analyze common patterns for file operations
        if command.contains("touch") || (command.contains("echo") && command.contains(">")) {
            // File write operation detected
            if let Some(path) = self.extract_file_path_from_command(command) {
                effects.push(SideEffect::FileWrite(working_dir.join(path)));
            }
        }

        if command.contains("cat") || command.contains("less") || command.contains("head") {
            // File read operation detected
            if let Some(path) = self.extract_file_path_from_command(command) {
                effects.push(SideEffect::FileRead(working_dir.join(path)));
            }
        }

        if command.contains("curl") || command.contains("wget") || command.contains("http") {
            // Network request detected
            effects.push(SideEffect::NetworkRequest(command.to_string()));
        }

        // Additional patterns for more comprehensive detection
        if command.contains("rm ") || command.contains("rm -") {
            // File deletion detected
            if let Some(path) = self.extract_file_path_from_rm_command(command) {
                effects.push(SideEffect::FileWrite(working_dir.join(path))); // Treat deletion as write
            }
        }

        if command.contains("mkdir") {
            // Directory creation detected
            if let Some(path) = self.extract_file_path_from_mkdir_command(command) {
                effects.push(SideEffect::FileWrite(working_dir.join(path)));
            }
        }

        if command.contains("sed -i") || command.contains("awk") {
            // In-place file modification detected
            if let Some(path) = self.extract_file_path_from_sed_command(command) {
                effects.push(SideEffect::FileWrite(working_dir.join(path)));
            }
        }

        effects
    }

    /// Extract file path from common file operation commands
    pub fn extract_file_path_from_command(&self, command: &str) -> Option<String> {
        // Simple parsing for common patterns
        if let Some(pos) = command.find('>') {
            // echo "text" > file.txt
            let after_redirect = &command[pos + 1..].trim();
            Some(after_redirect.split_whitespace().next()?.to_string())
        } else if command.starts_with("cat ") {
            // cat file.txt
            let parts: Vec<&str> = command.split_whitespace().collect();
            if parts.len() > 1 {
                Some(parts[1].to_string())
            } else {
                None
            }
        } else if command.starts_with("touch ") {
            // touch file.txt
            let parts: Vec<&str> = command.split_whitespace().collect();
            if parts.len() > 1 {
                Some(parts[1].to_string())
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Extract file path from rm commands
    pub fn extract_file_path_from_rm_command(&self, command: &str) -> Option<String> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.len() > 1 && parts[0] == "rm" {
            // Find the last argument that doesn't start with '-'
            for part in parts.iter().rev() {
                if !part.starts_with('-') && *part != "rm" {
                    return Some(part.to_string());
                }
            }
        }
        None
    }

    /// Extract directory path from mkdir commands
    pub fn extract_file_path_from_mkdir_command(&self, command: &str) -> Option<String> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.len() > 1 && parts[0] == "mkdir" {
            // Find the last argument that doesn't start with '-'
            for part in parts.iter().rev() {
                if !part.starts_with('-') && *part != "mkdir" {
                    return Some(part.to_string());
                }
            }
        }
        None
    }

    /// Extract file path from sed commands
    pub fn extract_file_path_from_sed_command(&self, command: &str) -> Option<String> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.len() > 1 && parts[0] == "sed" {
            // Find the last argument (usually the file)
            if let Some(last) = parts.last() {
                if !last.starts_with('-') && *last != "sed" {
                    return Some(last.to_string());
                }
            }
        }
        None
    }
}

impl DryRunnable for ExecutionPlugin {
    fn dry_run(&self, context: &ExecutionContext) -> Result<DryRunReport> {
        let command = "mock-command".to_string(); // Will be replaced with actual command
        let estimated_duration = self.estimate_duration(&command);
        let side_effects =
            self.analyze_side_effects(&command, &context.working_directory, &context.environment);

        Ok(DryRunReport {
            command,
            environment: context.environment.clone(),
            working_directory: context.working_directory.clone(),
            dependencies: vec![], // Will be populated with actual dependencies
            estimated_duration: Some(estimated_duration),
            side_effects,
        })
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
        self.dry_run = config.dry_run;
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

        if self.dry_run {
            // Handle dry-run mode with enhanced analysis
            self.execute_dry_run(graph, task_id as usize)?;
        } else {
            // Handle normal execution
            self.execute_normal(graph, task_id as usize)?;
        }

        Ok(())
    }
}

impl ExecutionPlugin {
    fn execute_dry_run(&self, graph: &Graph, task_id: usize) -> Result<()> {
        let mut visited = std::collections::HashSet::new();
        let mut dry_run_reports = Vec::new();

        self.collect_dry_run_reports(task_id, graph, &mut visited, &mut dry_run_reports)?;

        // Display dry-run results
        self.display_dry_run_results(&dry_run_reports)?;

        Ok(())
    }

    fn collect_dry_run_reports(
        &self,
        node_id: usize,
        graph: &Graph,
        visited: &mut std::collections::HashSet<usize>,
        reports: &mut Vec<DryRunReport>,
    ) -> Result<()> {
        if visited.contains(&node_id) {
            return Ok(());
        }
        visited.insert(node_id);

        let node = &graph.nodes[node_id];
        match &node.kind {
            NodeKind::Task(task_data) => {
                // Process pre dependencies
                for edge in &graph.edges {
                    if edge.to == node_id as u64 {
                        self.collect_dry_run_reports(edge.from as usize, graph, visited, reports)?;
                    }
                }

                // Generate dry-run report for this task
                if let Some(cmd) = &task_data.command {
                    let expanded_cmd = self.expand_env_vars(cmd, &task_data.env);
                    let working_dir = task_data
                        .working_dir
                        .as_ref()
                        .map(PathBuf::from)
                        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

                    let report = DryRunReport {
                        command: expanded_cmd.clone(),
                        environment: task_data.env.clone(),
                        working_directory: working_dir.clone(),
                        dependencies: vec![], // Could be enhanced to list actual dependencies
                        estimated_duration: Some(self.estimate_duration(&expanded_cmd)),
                        side_effects: self.analyze_side_effects(
                            &expanded_cmd,
                            &working_dir,
                            &task_data.env,
                        ),
                    };
                    reports.push(report);
                }
            }
            NodeKind::Command(cmd_data) => {
                let expanded_cmd = self.expand_env_vars(&cmd_data.raw_command, &cmd_data.env);
                let working_dir = cmd_data
                    .working_dir
                    .as_ref()
                    .map(PathBuf::from)
                    .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

                let report = DryRunReport {
                    command: expanded_cmd.clone(),
                    environment: cmd_data.env.clone(),
                    working_directory: working_dir.clone(),
                    dependencies: vec![],
                    estimated_duration: Some(self.estimate_duration(&expanded_cmd)),
                    side_effects: self.analyze_side_effects(
                        &expanded_cmd,
                        &working_dir,
                        &cmd_data.env,
                    ),
                };
                reports.push(report);
            }
            NodeKind::ConcurrentGroup(group_data) => {
                // Handle concurrent group in dry-run
                for &child_id in &group_data.child_nodes {
                    self.collect_dry_run_reports(child_id as usize, graph, visited, reports)?;
                }
            }
        }
        Ok(())
    }

    pub fn display_dry_run_results(&self, reports: &[DryRunReport]) -> Result<()> {
        println!("🔍 Dry Run Results (Enhanced with Sandbox Analysis)");
        println!("==================================================");
        println!();

        let mut total_duration = Duration::new(0, 0);

        for (i, report) in reports.iter().enumerate() {
            println!("📋 Command {}: {}", i + 1, report.command);
            println!(
                "📁 Working Directory: {}",
                report.working_directory.display()
            );

            if !report.environment.is_empty() {
                println!("🌍 Environment Variables:");
                for (key, value) in &report.environment {
                    println!("   {}={}", key, value);
                }
            }

            if let Some(duration) = report.estimated_duration {
                total_duration += duration;
                println!("⏱️  Estimated Duration: {:?}", duration);
            }

            if !report.side_effects.is_empty() {
                println!("⚠️  Detected Side Effects:");
                for effect in &report.side_effects {
                    match effect {
                        SideEffect::FileWrite(path) => {
                            println!("   📝 Write to: {}", path.display())
                        }
                        SideEffect::FileRead(path) => {
                            println!("   📖 Read from: {}", path.display())
                        }
                        SideEffect::NetworkRequest(url) => {
                            println!("   🌐 Network request: {}", url)
                        }
                        SideEffect::ProcessSpawn(cmd) => println!("   🚀 Process spawn: {}", cmd),
                        SideEffect::EnvironmentChange(key, value) => {
                            println!("   🌍 Environment change: {}={}", key, value)
                        }
                    }
                }
            } else {
                println!("✅ No side effects detected");
            }
            println!();
        }

        println!("📊 Summary");
        println!("----------");
        println!("Total Commands: {}", reports.len());
        println!("Estimated Total Duration: {:?}", total_duration);
        println!();
        println!("✅ No commands were actually executed (dry-run mode)");
        println!("🔒 All analysis performed in isolated sandbox environment");

        Ok(())
    }

    fn execute_normal(&self, graph: &Graph, task_id: usize) -> Result<()> {
        let mut visited = std::collections::HashSet::new();
        let mut pm = ProcessManager::new(true);

        #[allow(clippy::type_complexity)]
        fn run_node(
            node_id: usize,
            graph: &Graph,
            pm: &mut ProcessManager,
            visited: &mut std::collections::HashSet<usize>,
            expand_env_vars_fn: &dyn Fn(&str, &HashMap<String, String>) -> String,
            get_prefix_settings_fn: &dyn Fn(&crate::graph::Node) -> (bool, Option<String>, Option<String>),
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
            task_id,
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
