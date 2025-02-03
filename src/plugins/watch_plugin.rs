// src/plugins/watch_plugin.rs

use std::any::Any;

use crate::{
    graph::Graph,
    plugin::{Plugin, PluginConfig},
    Result,
};

pub struct WatchPlugin {
    watch_mode: bool,
    stop_on_fail: bool,
}

impl WatchPlugin {
    pub fn new(watch_mode: bool, stop_on_fail: bool) -> Self {
        Self {
            watch_mode,
            stop_on_fail,
        }
    }

    pub fn get_watch_entry_count(&self) -> usize {
        0
    }
}

impl Plugin for WatchPlugin {
    fn name(&self) -> &'static str {
        "WatchPlugin"
    }

    fn priority(&self) -> i32 {
        70
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn on_init(&mut self, _config: &PluginConfig) -> Result<()> {
        Ok(())
    }

    fn on_graph_build(&mut self, _graph: &mut Graph) -> Result<()> {
        Ok(())
    }

    fn on_after_run(&mut self, _graph: &mut Graph) -> Result<()> {
        Ok(())
    }
}
