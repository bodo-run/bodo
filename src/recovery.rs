use crate::errors::{BodoError, ErrorCategory, Result};
use std::time::{Duration, Instant};
use tracing::{debug, warn, info};

/// Recovery strategy for handling failures
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// Retry with exponential backoff
    Retry { 
        max_attempts: u32, 
        initial_delay: Duration,
        max_delay: Duration,
        backoff_multiplier: f64,
    },
    /// Rollback to a previous state
    Rollback { 
        checkpoint: String 
    },
    /// Continue execution, skipping failed tasks
    Continue { 
        skip_failed: bool 
    },
    /// Abort execution immediately
    Abort,
}

impl Default for RecoveryStrategy {
    fn default() -> Self {
        RecoveryStrategy::Retry {
            max_attempts: 3,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
        }
    }
}

/// Configuration for retry behavior
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
        }
    }
}

/// Retry executor with exponential backoff
pub struct RetryExecutor {
    config: RetryConfig,
}

impl RetryExecutor {
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    /// Execute a function with retry logic
    pub async fn execute<F, T, Fut>(&self, mut operation: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut attempts = 0;
        let mut delay = self.config.initial_delay;

        loop {
            attempts += 1;
            let start_time = Instant::now();
            
            debug!("Executing operation, attempt {}/{}", attempts, self.config.max_attempts);
            
            match operation().await {
                Ok(result) => {
                    if attempts > 1 {
                        info!("Operation succeeded after {} attempts", attempts);
                    }
                    return Ok(result);
                }
                Err(error) => {
                    let _duration = start_time.elapsed();
                    
                    if attempts >= self.config.max_attempts {
                        warn!("Operation failed after {} attempts, giving up", attempts);
                        return Err(BodoError::RetryExhausted { attempts });
                    }
                    
                    if !error.is_retryable() {
                        warn!("Non-retryable error encountered: {}", error);
                        return Err(error);
                    }
                    
                    warn!(
                        "Operation failed (attempt {}/{}), retrying in {:?}. Error: {}",
                        attempts, self.config.max_attempts, delay, error
                    );
                    
                    // Sleep before retry
                    tokio::time::sleep(delay).await;
                    
                    // Calculate next delay with exponential backoff
                    delay = Duration::from_millis(
                        (delay.as_millis() as f64 * self.config.backoff_multiplier) as u64
                    ).min(self.config.max_delay);
                }
            }
        }
    }

    /// Execute a synchronous function with retry logic
    pub fn execute_sync<F, T>(&self, mut operation: F) -> Result<T>
    where
        F: FnMut() -> Result<T>,
    {
        let mut attempts = 0;
        let mut delay = self.config.initial_delay;

        loop {
            attempts += 1;
            let start_time = Instant::now();
            
            debug!("Executing operation, attempt {}/{}", attempts, self.config.max_attempts);
            
            match operation() {
                Ok(result) => {
                    if attempts > 1 {
                        info!("Operation succeeded after {} attempts", attempts);
                    }
                    return Ok(result);
                }
                Err(error) => {
                    let _duration = start_time.elapsed();
                    
                    if attempts >= self.config.max_attempts {
                        warn!("Operation failed after {} attempts, giving up", attempts);
                        return Err(BodoError::RetryExhausted { attempts });
                    }
                    
                    if !error.is_retryable() {
                        warn!("Non-retryable error encountered: {}", error);
                        return Err(error);
                    }
                    
                    warn!(
                        "Operation failed (attempt {}/{}), retrying in {:?}. Error: {}",
                        attempts, self.config.max_attempts, delay, error
                    );
                    
                    // Sleep before retry
                    std::thread::sleep(delay);
                    
                    // Calculate next delay with exponential backoff
                    delay = Duration::from_millis(
                        (delay.as_millis() as f64 * self.config.backoff_multiplier) as u64
                    ).min(self.config.max_delay);
                }
            }
        }
    }
}

/// Determine recovery strategy based on error type
pub fn determine_recovery_strategy(error: &BodoError) -> RecoveryStrategy {
    match error.category() {
        ErrorCategory::Transient => RecoveryStrategy::Retry {
            max_attempts: 3,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
        },
        ErrorCategory::Resource => RecoveryStrategy::Retry {
            max_attempts: 5,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 1.5,
        },
        ErrorCategory::Timeout => RecoveryStrategy::Retry {
            max_attempts: 2,
            initial_delay: Duration::from_secs(2),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 3.0,
        },
        ErrorCategory::Permanent => RecoveryStrategy::Abort,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_retry_success_after_failure() {
        use std::sync::{Arc, Mutex};
        
        let config = RetryConfig {
            max_attempts: 3,
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            backoff_multiplier: 2.0,
        };
        
        let executor = RetryExecutor::new(config);
        let call_count = Arc::new(Mutex::new(0));
        let call_count_clone = call_count.clone();
        
        let result = executor.execute(move || {
            let call_count = call_count_clone.clone();
            async move {
                let mut count = call_count.lock().unwrap();
                *count += 1;
                if *count < 3 {
                    Err(BodoError::PluginError("Temporary failure".to_string()))
                } else {
                    Ok("Success")
                }
            }
        }).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Success");
        assert_eq!(*call_count.lock().unwrap(), 3);
    }

    #[test]
    fn test_error_categorization() {
        let timeout_error = BodoError::TimeoutError { duration: Duration::from_secs(30) };
        assert_eq!(timeout_error.category(), ErrorCategory::Timeout);
        assert!(timeout_error.is_retryable());
        
        let validation_error = BodoError::ValidationError("Invalid input".to_string());
        assert_eq!(validation_error.category(), ErrorCategory::Permanent);
        assert!(!validation_error.is_retryable());
    }

    #[test]
    fn test_recovery_strategy_determination() {
        let transient_error = BodoError::PluginError("Network glitch".to_string());
        let strategy = determine_recovery_strategy(&transient_error);
        
        match strategy {
            RecoveryStrategy::Retry { max_attempts, .. } => {
                assert_eq!(max_attempts, 3);
            }
            _ => panic!("Expected retry strategy"),
        }
    }
}