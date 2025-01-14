use std::fmt;

#[derive(Debug)]
pub enum PluginError {
    // Expand with I/O, parsing, or more specialized errors as needed
    GenericError(String),
    IoError(std::io::Error),
    NotifyError(notify::Error),
}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PluginError::GenericError(msg) => write!(f, "Plugin error: {}", msg),
            PluginError::IoError(e) => write!(f, "I/O error: {}", e),
            PluginError::NotifyError(e) => write!(f, "File watch error: {}", e),
        }
    }
}

impl std::error::Error for PluginError {}

impl From<notify::Error> for PluginError {
    fn from(err: notify::Error) -> Self {
        PluginError::NotifyError(err)
    }
}

pub type Result<T> = std::result::Result<T, PluginError>;
