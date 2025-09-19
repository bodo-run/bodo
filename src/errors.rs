use std::{error::Error, fmt, io, time::Duration};
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
    TimeoutError { duration: Duration },
    RetryExhausted { attempts: u32 },
    RollbackFailed { reason: String },
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
            BodoError::TimeoutError { duration } => write!(f, "Task timed out after {:?}", duration),
            BodoError::RetryExhausted { attempts } => write!(f, "Failed after {} retry attempts", attempts),
            BodoError::RollbackFailed { reason } => write!(f, "Rollback failed: {}", reason),
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

/// Error category for recovery strategy determination
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorCategory {
    /// Temporary errors that might succeed on retry
    Transient,
    /// Configuration or input errors that won't be fixed by retry
    Permanent,
    /// Resource errors (file system, network, etc.)
    Resource,
    /// Timeout errors
    Timeout,
}

impl BodoError {
    /// Categorize error for recovery strategy
    pub fn category(&self) -> ErrorCategory {
        match self {
            BodoError::TimeoutError { .. } => ErrorCategory::Timeout,
            BodoError::PluginError(_) => ErrorCategory::Transient,
            BodoError::IoError(_) => ErrorCategory::Resource,
            BodoError::ValidationError(_) => ErrorCategory::Permanent,
            BodoError::TaskNotFound(_) => ErrorCategory::Permanent,
            BodoError::NoTaskSpecified => ErrorCategory::Permanent,
            BodoError::WatcherError(_) => ErrorCategory::Resource,
            BodoError::RetryExhausted { .. } => ErrorCategory::Permanent,
            BodoError::RollbackFailed { .. } => ErrorCategory::Permanent,
            BodoError::SerdeError(_) => ErrorCategory::Permanent,
            BodoError::YamlError(_) => ErrorCategory::Permanent,
        }
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(self.category(), ErrorCategory::Transient | ErrorCategory::Resource | ErrorCategory::Timeout)
    }
}
