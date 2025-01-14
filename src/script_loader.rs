use serde::Deserialize;
use std::path::PathBuf;

use crate::{errors::Result, graph::Graph};

#[derive(Debug, Deserialize, Default, Clone)]
pub struct BodoConfig {
    pub scripts_dir: Option<String>,
    pub scripts_glob: Option<String>,
}

pub fn load_bodo_config(_config_path: Option<&str>) -> Result<BodoConfig> {
    Ok(BodoConfig::default())
}

pub fn load_scripts(_paths: &[PathBuf], _graph: &mut Graph) -> Result<()> {
    Ok(())
}
