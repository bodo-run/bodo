use crate::config::WatchConfig;

pub struct DependencyGraph {
    pub watch_config: WatchConfig,
}

impl DependencyGraph {
    pub fn new(watch_config: WatchConfig) -> Self {
        DependencyGraph { watch_config }
    }
}
