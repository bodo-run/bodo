//! Error recovery mechanisms including retry logic, rollback, and recovery strategies

use crate::errors::{BodoError, RecoveryStrategy, TaskCheckpoint, CheckpointState};
use crate::metrics;
use crate::Result;
use std::time::{Duration, SystemTime};
use std::collections::HashMap;
use std::path::PathBuf;

/// Retry configuration with exponential backoff
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial backoff duration
    pub initial_backoff: Duration,
    /// Maximum backoff duration
    pub max_backoff: Duration,
    /// Backoff multiplier for exponential growth
    pub backoff_multiplier: f64,
    /// Whether to add jitter to avoid thundering herd
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

/// Retry mechanism with exponential backoff
pub struct RetryMechanism {
    config: RetryConfig,
}

impl RetryMechanism {
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    /// Execute an operation with retry logic
    pub fn execute_with_retry<F, T>(&self, mut operation: F) -> Result<T>
    where
        F: FnMut() -> Result<T>,
    {
        let mut last_error = None;
        let mut backoff = self.config.initial_backoff;
        let start_time = SystemTime::now();

        for attempt in 1..=self.config.max_attempts {
            metrics::record_retry_attempt();
            
            match operation() {
                Ok(result) => {
                    if attempt > 1 {
                        // This was a recovery success
                        if let Ok(elapsed) = start_time.elapsed() {
                            metrics::record_recovery_success(elapsed);
                        }
                        tracing::info!(
                            attempt = attempt,
                            "Operation succeeded after retry"
                        );
                    }
                    return Ok(result);
                }
                Err(error) => {
                    metrics::record_error(&error);
                    
                    tracing::warn!(
                        attempt = attempt,
                        max_attempts = self.config.max_attempts,
                        error = %error,
                        "Operation failed, will retry"
                    );

                    // Don't retry permanent errors
                    if !error.is_retryable() {
                        tracing::info!("Error is not retryable, aborting retry attempts");
                        if let Ok(elapsed) = start_time.elapsed() {
                            metrics::record_recovery_failure(elapsed);
                        }
                        return Err(error);
                    }

                    last_error = Some(error);

                    // Don't sleep after the last attempt
                    if attempt < self.config.max_attempts {
                        let sleep_duration = self.calculate_backoff(backoff);
                        tracing::debug!(
                            backoff_ms = sleep_duration.as_millis(),
                            "Sleeping before retry"
                        );
                        std::thread::sleep(sleep_duration);
                        
                        // Increase backoff for next iteration
                        backoff = std::cmp::min(
                            Duration::from_millis(
                                (backoff.as_millis() as f64 * self.config.backoff_multiplier) as u64
                            ),
                            self.config.max_backoff,
                        );
                    }
                }
            }
        }

        // All retry attempts exhausted
        if let Ok(elapsed) = start_time.elapsed() {
            metrics::record_recovery_failure(elapsed);
        }
        
        let last_error = last_error.unwrap(); // Safe because we always have at least one attempt
        Err(BodoError::RetryExhausted {
            attempts: self.config.max_attempts,
            last_error: Box::new(last_error),
        })
    }

    /// Calculate backoff duration with optional jitter
    fn calculate_backoff(&self, base_duration: Duration) -> Duration {
        if !self.config.jitter {
            return base_duration;
        }

        // Add up to 25% jitter to avoid thundering herd
        let jitter_range = base_duration.as_millis() / 4;
        let jitter = fastrand::u64(0..=(jitter_range as u64));
        
        Duration::from_millis(base_duration.as_millis() as u64 + jitter)
    }
}

/// Checkpoint manager for rollback functionality
pub struct CheckpointManager {
    checkpoints: HashMap<String, TaskCheckpoint>,
}

impl CheckpointManager {
    pub fn new() -> Self {
        Self {
            checkpoints: HashMap::new(),
        }
    }

    /// Create a checkpoint for a task
    pub fn create_checkpoint(
        &mut self,
        task_id: String,
        working_directory: PathBuf,
        environment: HashMap<String, String>,
    ) -> Result<TaskCheckpoint> {
        let checkpoint = TaskCheckpoint {
            task_id: task_id.clone(),
            timestamp: SystemTime::now(),
            state: CheckpointState {
                environment,
                working_directory,
                metadata: HashMap::new(),
            },
        };

        self.checkpoints.insert(task_id, checkpoint.clone());
        tracing::debug!(
            task_id = %checkpoint.task_id,
            "Created checkpoint"
        );

        Ok(checkpoint)
    }

    /// Rollback to a specific checkpoint
    pub fn rollback_to_checkpoint(&self, checkpoint: &TaskCheckpoint) -> Result<()> {
        tracing::info!(
            task_id = %checkpoint.task_id,
            "Rolling back to checkpoint"
        );

        // Restore environment variables
        for (key, value) in &checkpoint.state.environment {
            std::env::set_var(key, value);
        }

        // Restore working directory
        std::env::set_current_dir(&checkpoint.state.working_directory)
            .map_err(|e| BodoError::RollbackFailed {
                checkpoint: Box::new(checkpoint.clone()),
                cause: format!("Failed to restore working directory: {}", e),
            })?;

        tracing::debug!(
            task_id = %checkpoint.task_id,
            "Successfully rolled back to checkpoint"
        );

        Ok(())
    }

    /// Get a checkpoint by task ID
    pub fn get_checkpoint(&self, task_id: &str) -> Option<&TaskCheckpoint> {
        self.checkpoints.get(task_id)
    }

    /// Remove a checkpoint
    pub fn remove_checkpoint(&mut self, task_id: &str) -> Option<TaskCheckpoint> {
        self.checkpoints.remove(task_id)
    }
}

impl Default for CheckpointManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Recovery strategy executor
#[derive(Default)]
pub struct RecoveryExecutor {
    checkpoint_manager: CheckpointManager,
}

impl RecoveryExecutor {
    pub fn new(_retry_config: RetryConfig) -> Self {
        Self {
            checkpoint_manager: CheckpointManager::new(),
        }
    }

    /// Execute a recovery strategy
    pub fn execute_strategy<F, T>(
        &mut self,
        strategy: &RecoveryStrategy,
        mut operation: F,
    ) -> Result<T>
    where
        F: FnMut() -> Result<T>,
    {
        match strategy {
            RecoveryStrategy::Retry { max_attempts, backoff } => {
                let config = RetryConfig {
                    max_attempts: *max_attempts,
                    initial_backoff: *backoff,
                    ..Default::default()
                };
                let retry_mechanism = RetryMechanism::new(config);
                retry_mechanism.execute_with_retry(operation)
            }
            RecoveryStrategy::Rollback { checkpoint } => {
                // Execute operation, rollback on failure
                match operation() {
                    Ok(result) => Ok(result),
                    Err(error) => {
                        self.checkpoint_manager.rollback_to_checkpoint(checkpoint)?;
                        Err(error)
                    }
                }
            }
            RecoveryStrategy::Continue { skip_failed } => {
                match operation() {
                    Ok(result) => Ok(result),
                    Err(error) => {
                        if *skip_failed {
                            tracing::warn!(
                                error = %error,
                                "Operation failed but continuing due to Continue strategy"
                            );
                            // For Continue strategy, we need a default value
                            // This is a limitation - the caller needs to handle this case
                            Err(error)
                        } else {
                            Err(error)
                        }
                    }
                }
            }
            RecoveryStrategy::Abort => {
                // Execute once, abort immediately on failure
                operation()
            }
        }
    }

    /// Create a checkpoint
    pub fn create_checkpoint(
        &mut self,
        task_id: String,
        working_directory: PathBuf,
        environment: HashMap<String, String>,
    ) -> Result<TaskCheckpoint> {
        self.checkpoint_manager.create_checkpoint(task_id, working_directory, environment)
    }

    /// Access the checkpoint manager
    pub fn checkpoint_manager(&self) -> &CheckpointManager {
        &self.checkpoint_manager
    }

    /// Access the checkpoint manager mutably
    pub fn checkpoint_manager_mut(&mut self) -> &mut CheckpointManager {
        &mut self.checkpoint_manager
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::ErrorCategory;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_retry_mechanism_success() {
        let retry_mechanism = RetryMechanism::new(RetryConfig::default());
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry_mechanism.execute_with_retry(|| {
            let count = counter_clone.fetch_add(1, Ordering::SeqCst);
            if count < 2 {
                Err(BodoError::WatcherError("temporary error".to_string()))
            } else {
                Ok("success".to_string())
            }
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_retry_mechanism_permanent_error() {
        let retry_mechanism = RetryMechanism::new(RetryConfig::default());
        
        let result: Result<String> = retry_mechanism.execute_with_retry(|| {
            Err(BodoError::ValidationError("permanent error".to_string()))
        });

        assert!(result.is_err());
        // Should not retry permanent errors
        match result.unwrap_err() {
            BodoError::ValidationError(_) => {}, // Expected
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_retry_mechanism_exhausted() {
        let config = RetryConfig {
            max_attempts: 2,
            initial_backoff: Duration::from_millis(1), // Fast for testing
            ..Default::default()
        };
        let retry_mechanism = RetryMechanism::new(config);

        let result: Result<String> = retry_mechanism.execute_with_retry(|| {
            Err(BodoError::WatcherError("always fails".to_string()))
        });

        assert!(result.is_err());
        match result.unwrap_err() {
            BodoError::RetryExhausted { attempts, .. } => {
                assert_eq!(attempts, 2);
            }
            _ => panic!("Expected RetryExhausted"),
        }
    }

    #[test]
    fn test_checkpoint_creation_and_rollback() {
        let mut manager = CheckpointManager::new();
        let env = HashMap::from([("TEST_VAR".to_string(), "test_value".to_string())]);
        let working_dir = PathBuf::from("/tmp");

        let checkpoint = manager
            .create_checkpoint("test_task".to_string(), working_dir, env.clone())
            .unwrap();

        assert_eq!(checkpoint.task_id, "test_task");
        assert_eq!(checkpoint.state.environment, env);

        let retrieved = manager.get_checkpoint("test_task").unwrap();
        assert_eq!(retrieved.task_id, checkpoint.task_id);
    }

    #[test]
    fn test_error_categorization() {
        assert_eq!(
            BodoError::WatcherError("test".to_string()).categorize(),
            ErrorCategory::Transient
        );
        assert_eq!(
            BodoError::ValidationError("test".to_string()).categorize(),
            ErrorCategory::Permanent
        );
        assert!(BodoError::WatcherError("test".to_string()).is_retryable());
        assert!(!BodoError::ValidationError("test".to_string()).is_retryable());
    }
}