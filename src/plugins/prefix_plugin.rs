use std::any::Any;

use crate::plugin::Plugin;

pub struct PrefixPlugin;

impl PrefixPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PrefixPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for PrefixPlugin {
    fn name(&self) -> &'static str {
        "PrefixPlugin"
    }

    fn priority(&self) -> i32 {
        90 // Lower than ExecutionPlugin (95)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
