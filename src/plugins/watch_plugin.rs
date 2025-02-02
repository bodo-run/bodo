use crate::config::WatchConfig;

pub struct WatchPlugin {
    config: WatchConfig,
}

impl WatchPlugin {
    pub fn new(config: WatchConfig) -> Self {
        WatchPlugin { config }
    }
}

fn graph_manager_config_snapshot() -> Result<crate::config::BodoConfig, Box<dyn std::error::Error>>
{
    Ok(crate::config::BodoConfig {
        timeout: 30,
        tasks: vec![],
        watch: None,
    })
}
