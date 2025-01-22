use async_trait::async_trait;
use std::{any::Any, collections::HashMap};

use crate::{
    graph::{Graph, NodeKind},
    plugin::{Plugin, PluginConfig},
    Result,
};

pub struct EnvPlugin {
    global_env: Option<HashMap<String, String>>,
}

impl EnvPlugin {
    pub fn new() -> Self {
        EnvPlugin { global_env: None }
    }
}

#[async_trait]
impl Plugin for EnvPlugin {
    fn name(&self) -> &'static str {
        "EnvPlugin"
    }

    fn priority(&self) -> i32 {
        90 // High priority for environment setup
    }

    async fn on_init(&mut self, config: &PluginConfig) -> Result<()> {
        if let Some(options) = &config.options {
            if let Some(val) = options.get("env") {
                // Parse env object: { "FOO": "bar", "XYZ": "123" }
                if let Some(obj) = val.as_object() {
                    let mut map = HashMap::new();
                    for (k, v) in obj {
                        if let Some(s) = v.as_str() {
                            map.insert(k.clone(), s.to_string());
                        }
                    }
                    self.global_env = Some(map);
                }
            }
        }
        Ok(())
    }

    async fn on_graph_build(&mut self, graph: &mut Graph) -> Result<()> {
        if let Some(ref global_env) = self.global_env {
            for node in &mut graph.nodes {
                match &mut node.kind {
                    NodeKind::Task(task_data) => {
                        for (k, v) in global_env {
                            // Only set if not already set
                            if !task_data.env.contains_key(k) {
                                task_data.env.insert(k.clone(), v.clone());
                            }
                        }
                    }
                    NodeKind::Command(cmd_data) => {
                        for (k, v) in global_env {
                            if !cmd_data.env.contains_key(k) {
                                cmd_data.env.insert(k.clone(), v.clone());
                            }
                        }
                    }
                    NodeKind::ConcurrentGroup(_) => {
                        // No direct env for concurrent groups
                    }
                }
            }
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
