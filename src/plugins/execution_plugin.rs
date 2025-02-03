/// src/plugins/execution_plugin.rs
// src/plugins/execution_plugin.rs

// ... [Previous code] ...

impl ExecutionPlugin {
    pub fn new() -> Self {
        Self { task_name: None }
    }

    // ... [Other methods] ...

    // Changed the visibility of the method to `pub`
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
}

// ... [Rest of the code] ...
