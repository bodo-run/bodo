use std::{error::Error, fmt, io};
use validator::{ValidationError, ValidationErrors};

#[derive(Debug)]
pub enum BodoError {
    IoError(io::Error),
    WatcherError(String),
    TaskNotFound(String),
    PluginError(String),
    SerdeError(serde_json::Error),
    YamlError(serde_yaml::Error),
    NoTaskSpecified,
    ValidationError(String),
}

impl fmt::Display for BodoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BodoError::IoError(err) => write!(f, "{}", err),
            BodoError::WatcherError(err) => write!(f, "{}", err),
            BodoError::TaskNotFound(_) => write!(f, "not found"),
            BodoError::PluginError(err) => write!(f, "Plugin error: {}", err),
            BodoError::SerdeError(err) => write!(f, "{}", err),
            BodoError::YamlError(err) => write!(f, "{}", err),
            BodoError::NoTaskSpecified => {
                write!(f, "No task specified and no scripts/script.yaml found")
            }
            BodoError::ValidationError(err) => write!(f, "Validation error: {}", err),
        }
    }
}

impl Error for BodoError {}

impl From<io::Error> for BodoError {
    fn from(err: io::Error) -> Self {
        BodoError::IoError(err)
    }
}

impl From<notify::Error> for BodoError {
    fn from(err: notify::Error) -> Self {
        BodoError::WatcherError(err.to_string())
    }
}

impl From<serde_json::Error> for BodoError {
    fn from(err: serde_json::Error) -> Self {
        BodoError::SerdeError(err)
    }
}

impl From<serde_yaml::Error> for BodoError {
    fn from(err: serde_yaml::Error) -> Self {
        BodoError::YamlError(err)
    }
}

impl From<ValidationError> for BodoError {
    fn from(err: ValidationError) -> Self {
        BodoError::ValidationError(err.to_string())
    }
}

impl From<ValidationErrors> for BodoError {
    fn from(err: ValidationErrors) -> Self {
        BodoError::ValidationError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, BodoError>;
