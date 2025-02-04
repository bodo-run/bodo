use crate::graph::Graph;
use crate::{BodoConfig, Result};
use std::collections::{HashMap, HashSet};

pub struct ScriptLoader;

impl Default for ScriptLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptLoader {
    pub fn new() -> Self {
        ScriptLoader
    }

    pub fn build_graph(&mut self, _config: BodoConfig) -> Result<Graph> {
        // Minimal implementation: return an empty graph.
        Ok(Graph::new())
    }

    pub fn merge_envs(
        global: &HashMap<String, String>,
        script: &HashMap<String, String>,
        task: &HashMap<String, String>,
    ) -> HashMap<String, String> {
        let mut merged = global.clone();
        for (k, v) in script {
            merged.insert(k.clone(), v.clone());
        }
        for (k, v) in task {
            merged.insert(k.clone(), v.clone());
        }
        merged
    }

    pub fn merge_exec_paths(
        global: &Vec<String>,
        script: &Vec<String>,
        task: &Vec<String>,
    ) -> Vec<String> {
        let mut seen = HashSet::new();
        let mut result = Vec::new();
        for path in global.iter().chain(script).chain(task) {
            if seen.insert(path.clone()) {
                result.push(path.clone());
            }
        }
        result
    }
}
