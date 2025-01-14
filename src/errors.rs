use std::fmt;

#[derive(Debug)]
pub enum PluginError {
    // Expand with I/O, parsing, or more specialized errors as needed
    GenericError(String),
    IoError(std::io::Error),
}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PluginError::GenericError(msg) => write!(f, "Plugin error: {}", msg),
            PluginError::IoError(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl std::error::Error for PluginError {}
