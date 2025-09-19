use std::{error::Error, fmt, io, time::Duration};
use validator::{ValidationError, ValidationErrors};

/// Error categorization for recovery strategies
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorCategory {
    /// Transient errors that may resolve with retry
    Transient,
    /// Permanent errors that won't resolve with retry
    Permanent, 
    /// Unknown error category
    Unknown,
}

/// Recovery strategy for error handling
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// Retry with exponential backoff
    Retry { max_attempts: u32, backoff: Duration },
    /// Rollback to previous checkpoint
    Rollback { checkpoint: TaskCheckpoint },
    /// Continue execution, skipping failed operations
    Continue { skip_failed: bool },
    /// Abort execution immediately  
    Abort,
}

/// Checkpoint for rollback functionality
#[derive(Debug, Clone)]
pub struct TaskCheckpoint {
    /// Task identifier
    pub task_id: String,
    /// Timestamp when checkpoint was created
    pub timestamp: std::time::SystemTime,
    /// State data for rollback
    pub state: CheckpointState,
}

/// State information captured at checkpoint
#[derive(Debug, Clone)]
pub struct CheckpointState {
    /// Environment variables at checkpoint
    pub environment: std::collections::HashMap<String, String>,
    /// Working directory at checkpoint
    pub working_directory: std::path::PathBuf,
    /// Any additional state data
    pub metadata: std::collections::HashMap<String, String>,
}

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
    /// Error during retry operations
    RetryExhausted { attempts: u32, last_error: Box<BodoError> },
    /// Error during rollback operations  
    RollbackFailed { checkpoint: TaskCheckpoint, cause: String },
    /// Recovery strategy error
    RecoveryFailed { strategy: String, cause: String },
}

impl BodoError {
    /// Categorize error for recovery strategy selection
    pub fn categorize(&self) -> ErrorCategory {
        match self {
            BodoError::IoError(err) => {
                match err.kind() {
                    io::ErrorKind::TimedOut | 
                    io::ErrorKind::Interrupted |
                    io::ErrorKind::ConnectionRefused |
                    io::ErrorKind::ConnectionAborted => ErrorCategory::Transient,
                    io::ErrorKind::NotFound |
                    io::ErrorKind::PermissionDenied |
                    io::ErrorKind::InvalidInput => ErrorCategory::Permanent,
                    _ => ErrorCategory::Unknown,
                }
            }
            BodoError::WatcherError(_) => ErrorCategory::Transient,
            BodoError::TaskNotFound(_) => ErrorCategory::Permanent,
            BodoError::PluginError(_) => ErrorCategory::Unknown,
            BodoError::SerdeError(_) | BodoError::YamlError(_) => ErrorCategory::Permanent,
            BodoError::NoTaskSpecified => ErrorCategory::Permanent,
            BodoError::ValidationError(_) => ErrorCategory::Permanent,
            BodoError::RetryExhausted { .. } => ErrorCategory::Permanent,
            BodoError::RollbackFailed { .. } => ErrorCategory::Permanent,
            BodoError::RecoveryFailed { .. } => ErrorCategory::Permanent,
        }
    }
    
    /// Check if error is retryable based on its category
    pub fn is_retryable(&self) -> bool {
        matches!(self.categorize(), ErrorCategory::Transient)
    }
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
            BodoError::RetryExhausted { attempts, .. } => {
                write!(f, "Retry exhausted after {} attempts", attempts)
            }
            BodoError::RollbackFailed { checkpoint, cause } => {
                write!(f, "Rollback failed for task '{}': {}", checkpoint.task_id, cause)
            }
            BodoError::RecoveryFailed { strategy, cause } => {
                write!(f, "Recovery strategy '{}' failed: {}", strategy, cause)
            }
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
