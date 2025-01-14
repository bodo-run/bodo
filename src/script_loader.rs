use crate::{errors::Result, graph::Graph};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Default, Clone)]
pub struct BodoConfig {
    pub scripts_dir: Option<String>,
    pub scripts_glob: Option<String>,
}

pub fn load_bodo_config(config_path: Option<&str>) -> Result<BodoConfig> {
    // Implementation...
    Ok(BodoConfig::default())
}

pub fn load_scripts(paths: &[PathBuf], graph: &mut Graph) -> Result<()> {
    // Implementation...
    Ok(())
}
