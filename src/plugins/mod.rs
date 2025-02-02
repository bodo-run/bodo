mod plugin;

pub use self::plugin::{Plugin, PluginConfig, PluginManager};

mod concurrent_plugin;
mod env_plugin;
mod execution_plugin;
mod path_plugin; 
mod prefix_plugin;
mod print_list_plugin;
mod timeout_plugin;