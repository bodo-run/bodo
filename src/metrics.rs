//! Error metrics and monitoring for the recovery system

use crate::errors::{BodoError, ErrorCategory};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// Error metrics collector
#[derive(Debug, Default)]
pub struct ErrorMetrics {
    /// Count of errors by category
    pub error_counts: HashMap<ErrorCategory, u64>,
    /// Count of successful recoveries
    pub recovery_successes: u64,
    /// Count of failed recoveries
    pub recovery_failures: u64,
    /// Total retry attempts
    pub retry_attempts: u64,
    /// Time spent in recovery operations
    pub recovery_time: Duration,
    /// Last error timestamp
    pub last_error_time: Option<SystemTime>,
}

impl ErrorMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an error occurrence
    pub fn record_error(&mut self, error: &BodoError) {
        let category = error.categorize();
        *self.error_counts.entry(category).or_insert(0) += 1;
        self.last_error_time = Some(SystemTime::now());
        
        tracing::debug!(
            category = ?category,
            error = %error,
            "Recorded error in metrics"
        );
    }

    /// Record a successful recovery
    pub fn record_recovery_success(&mut self, recovery_time: Duration) {
        self.recovery_successes += 1;
        self.recovery_time += recovery_time;
        
        tracing::debug!(
            recovery_time_ms = recovery_time.as_millis(),
            total_successes = self.recovery_successes,
            "Recorded successful recovery"
        );
    }

    /// Record a failed recovery
    pub fn record_recovery_failure(&mut self, recovery_time: Duration) {
        self.recovery_failures += 1;
        self.recovery_time += recovery_time;
        
        tracing::debug!(
            recovery_time_ms = recovery_time.as_millis(),
            total_failures = self.recovery_failures,
            "Recorded failed recovery"
        );
    }

    /// Record a retry attempt
    pub fn record_retry_attempt(&mut self) {
        self.retry_attempts += 1;
        
        tracing::trace!(
            total_attempts = self.retry_attempts,
            "Recorded retry attempt"
        );
    }

    /// Get error count for a specific category
    pub fn get_error_count(&self, category: &ErrorCategory) -> u64 {
        self.error_counts.get(category).copied().unwrap_or(0)
    }

    /// Get total error count
    pub fn total_errors(&self) -> u64 {
        self.error_counts.values().sum()
    }

    /// Get recovery success rate
    pub fn recovery_success_rate(&self) -> f64 {
        let total_recoveries = self.recovery_successes + self.recovery_failures;
        if total_recoveries == 0 {
            0.0
        } else {
            self.recovery_successes as f64 / total_recoveries as f64
        }
    }

    /// Get average recovery time
    pub fn average_recovery_time(&self) -> Duration {
        let total_recoveries = self.recovery_successes + self.recovery_failures;
        if total_recoveries == 0 {
            Duration::ZERO
        } else {
            self.recovery_time / total_recoveries as u32
        }
    }

    /// Generate a summary report
    pub fn summary_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("Error Metrics Summary:\n");
        report.push_str(&format!("  Total Errors: {}\n", self.total_errors()));
        
        for (category, count) in &self.error_counts {
            report.push_str(&format!("    {:?}: {}\n", category, count));
        }
        
        report.push_str(&format!("  Recovery Successes: {}\n", self.recovery_successes));
        report.push_str(&format!("  Recovery Failures: {}\n", self.recovery_failures));
        report.push_str(&format!("  Success Rate: {:.2}%\n", self.recovery_success_rate() * 100.0));
        report.push_str(&format!("  Total Retry Attempts: {}\n", self.retry_attempts));
        report.push_str(&format!("  Average Recovery Time: {:?}\n", self.average_recovery_time()));
        
        if let Some(last_error) = self.last_error_time {
            if let Ok(elapsed) = last_error.elapsed() {
                report.push_str(&format!("  Time Since Last Error: {:?}\n", elapsed));
            }
        }
        
        report
    }

    /// Reset all metrics
    pub fn reset(&mut self) {
        self.error_counts.clear();
        self.recovery_successes = 0;
        self.recovery_failures = 0;
        self.retry_attempts = 0;
        self.recovery_time = Duration::ZERO;
        self.last_error_time = None;
        
        tracing::info!("Error metrics reset");
    }
}

/// Global error metrics instance
use std::sync::{Mutex, OnceLock};
static GLOBAL_METRICS: OnceLock<Mutex<ErrorMetrics>> = OnceLock::new();

/// Get or initialize the global metrics instance
fn global_metrics() -> &'static Mutex<ErrorMetrics> {
    GLOBAL_METRICS.get_or_init(|| Mutex::new(ErrorMetrics::new()))
}

/// Record an error in global metrics
pub fn record_error(error: &BodoError) {
    if let Ok(mut metrics) = global_metrics().lock() {
        metrics.record_error(error);
    }
}

/// Record a successful recovery in global metrics
pub fn record_recovery_success(recovery_time: Duration) {
    if let Ok(mut metrics) = global_metrics().lock() {
        metrics.record_recovery_success(recovery_time);
    }
}

/// Record a failed recovery in global metrics
pub fn record_recovery_failure(recovery_time: Duration) {
    if let Ok(mut metrics) = global_metrics().lock() {
        metrics.record_recovery_failure(recovery_time);
    }
}

/// Record a retry attempt in global metrics
pub fn record_retry_attempt() {
    if let Ok(mut metrics) = global_metrics().lock() {
        metrics.record_retry_attempt();
    }
}

/// Get a summary report of global metrics
pub fn get_metrics_summary() -> String {
    if let Ok(metrics) = global_metrics().lock() {
        metrics.summary_report()
    } else {
        "Error: Could not acquire metrics lock".to_string()
    }
}

/// Reset global metrics
pub fn reset_metrics() {
    if let Ok(mut metrics) = global_metrics().lock() {
        metrics.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_metrics_basic() {
        let mut metrics = ErrorMetrics::new();
        
        let error = BodoError::WatcherError("test error".to_string());
        metrics.record_error(&error);
        
        assert_eq!(metrics.total_errors(), 1);
        assert_eq!(metrics.get_error_count(&ErrorCategory::Transient), 1);
        assert_eq!(metrics.get_error_count(&ErrorCategory::Permanent), 0);
    }

    #[test]
    fn test_recovery_metrics() {
        let mut metrics = ErrorMetrics::new();
        
        metrics.record_recovery_success(Duration::from_millis(100));
        metrics.record_recovery_failure(Duration::from_millis(200));
        
        assert_eq!(metrics.recovery_successes, 1);
        assert_eq!(metrics.recovery_failures, 1);
        assert_eq!(metrics.recovery_success_rate(), 0.5);
        assert_eq!(metrics.average_recovery_time(), Duration::from_millis(150));
    }

    #[test]
    fn test_retry_attempts() {
        let mut metrics = ErrorMetrics::new();
        
        metrics.record_retry_attempt();
        metrics.record_retry_attempt();
        metrics.record_retry_attempt();
        
        assert_eq!(metrics.retry_attempts, 3);
    }

    #[test]
    fn test_summary_report() {
        let mut metrics = ErrorMetrics::new();
        
        let error = BodoError::ValidationError("test".to_string());
        metrics.record_error(&error);
        metrics.record_recovery_success(Duration::from_millis(50));
        
        let report = metrics.summary_report();
        assert!(report.contains("Total Errors: 1"));
        assert!(report.contains("Recovery Successes: 1"));
        assert!(report.contains("Success Rate: 100.00%"));
    }

    #[test]
    fn test_reset() {
        let mut metrics = ErrorMetrics::new();
        
        let error = BodoError::WatcherError("test".to_string());
        metrics.record_error(&error);
        metrics.record_recovery_success(Duration::from_millis(100));
        
        assert_eq!(metrics.total_errors(), 1);
        assert_eq!(metrics.recovery_successes, 1);
        
        metrics.reset();
        
        assert_eq!(metrics.total_errors(), 0);
        assert_eq!(metrics.recovery_successes, 0);
        assert_eq!(metrics.recovery_failures, 0);
        assert_eq!(metrics.retry_attempts, 0);
    }
}