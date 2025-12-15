//! Health monitoring and decision-making for component restart triggers.
//!
//! Coordinates health checks with restart decisions, tracking consecutive failures
//! and triggering restarts based on degradation patterns.
//!
//! # Design
//!
//! - **Health states**: Healthy, Degraded, Unhealthy, Unknown
//! - **Consecutive failure tracking**: Counts failures for pattern detection
//! - **Recovery on success**: Clears failure counter
//! - **Interval checking**: Respects check intervals to avoid excessive checks
//!
//! # Example
//!
//! ```rust,ignore
//! use airssys_wasm::actor::{HealthMonitor, HealthStatus, HealthDecision};
//! use std::time::Duration;
//!
//! let mut monitor = HealthMonitor::new(Duration::from_secs(5));
//!
//! // Evaluate health status
//! let status = HealthStatus::Unhealthy {
//!     message: "Too many errors".to_string(),
//! };
//!
//! match monitor.evaluate_health(status) {
//!     HealthDecision::Unhealthy => println!("Restart needed!"),
//!     HealthDecision::Degraded => println!("Continue monitoring..."),
//!     HealthDecision::Healthy => println!("All good"),
//!     HealthDecision::Unknown => println!("Status unknown"),
//! }
//! ```

// Layer 1: Standard library imports
use std::time::{Duration, Instant};

/// Health status of a component.
///
/// Represents the current health assessment from a health check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    /// Component is operating normally
    Healthy,

    /// Component has minor issues but is still functional
    Degraded { message: String },

    /// Component is in critical state and should restart
    Unhealthy { message: String },

    /// Unable to determine health status
    Unknown,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "healthy"),
            HealthStatus::Degraded { message } => write!(f, "degraded ({})", message),
            HealthStatus::Unhealthy { message } => write!(f, "unhealthy ({})", message),
            HealthStatus::Unknown => write!(f, "unknown"),
        }
    }
}

/// Decision on whether to restart based on health status.
///
/// Produced by HealthMonitor after evaluating health status and trends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthDecision {
    /// Component is healthy, continue normal operation
    Healthy,

    /// Component is degraded, continue monitoring but don't restart yet
    Degraded,

    /// Component is unhealthy, trigger restart
    Unhealthy,

    /// Unable to determine, default to continue operation
    Unknown,
}

impl std::fmt::Display for HealthDecision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthDecision::Healthy => write!(f, "healthy"),
            HealthDecision::Degraded => write!(f, "degraded"),
            HealthDecision::Unhealthy => write!(f, "unhealthy"),
            HealthDecision::Unknown => write!(f, "unknown"),
        }
    }
}

/// Health monitor for component restart coordination.
///
/// Tracks health check results, counts consecutive failures, and determines
/// when restart is warranted based on health trends.
#[derive(Debug)]
pub struct HealthMonitor {
    /// Last recorded health status
    last_status: HealthStatus,

    /// How often health checks should be performed
    check_interval: Duration,

    /// When the last health check was performed
    last_check_time: Option<Instant>,

    /// Count of consecutive unhealthy/degraded checks
    consecutive_failures: u32,

    /// Threshold for unhealthy decision (e.g., 3 consecutive failures)
    failure_threshold: u32,
}

impl HealthMonitor {
    /// Create a new health monitor.
    ///
    /// # Parameters
    ///
    /// - `check_interval`: How often health checks should be performed
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use airssys_wasm::actor::HealthMonitor;
    /// use std::time::Duration;
    ///
    /// let monitor = HealthMonitor::new(Duration::from_secs(5));
    /// ```
    pub fn new(check_interval: Duration) -> Self {
        Self {
            last_status: HealthStatus::Unknown,
            check_interval,
            last_check_time: None,
            consecutive_failures: 0,
            failure_threshold: 3,
        }
    }

    /// Evaluate health status and produce a restart decision.
    ///
    /// Updates internal state and returns decision based on current and
    /// historical health status.
    ///
    /// # Parameters
    ///
    /// - `status`: Current health status from check
    ///
    /// # Returns
    ///
    /// Decision on whether to restart or continue
    pub fn evaluate_health(&mut self, status: HealthStatus) -> HealthDecision {
        // Record the status before pattern matching
        let decision = match &status {
            HealthStatus::Healthy => {
                self.consecutive_failures = 0;
                HealthDecision::Healthy
            }
            HealthStatus::Degraded { .. } => {
                self.consecutive_failures = self.consecutive_failures.saturating_add(1);
                HealthDecision::Degraded
            }
            HealthStatus::Unhealthy { .. } => {
                self.consecutive_failures = self.consecutive_failures.saturating_add(1);

                if self.consecutive_failures >= self.failure_threshold {
                    HealthDecision::Unhealthy
                } else {
                    HealthDecision::Degraded
                }
            }
            HealthStatus::Unknown => HealthDecision::Unknown,
        };
        
        // Record the status after decision is made
        self.record_check_result(status);
        decision
    }

    /// Check if a health check should be performed based on interval.
    ///
    /// # Returns
    ///
    /// `true` if `check_interval` has elapsed since last check, `false` otherwise.
    pub fn should_check_health(&self) -> bool {
        match self.last_check_time {
            None => true, // First check should always run
            Some(last_check) => last_check.elapsed() >= self.check_interval,
        }
    }

    /// Reset state when component recovers successfully.
    ///
    /// Clears failure counter and marks recovery point.
    pub fn reset_on_recovery(&mut self) {
        self.consecutive_failures = 0;
        self.last_status = HealthStatus::Healthy;
    }

    /// Record a health check result.
    ///
    /// Updates last_status and last_check_time. Called automatically
    /// by `evaluate_health()`.
    ///
    /// # Parameters
    ///
    /// - `status`: Health status from check
    pub fn record_check_result(&mut self, status: HealthStatus) {
        self.last_status = status;
        self.last_check_time = Some(Instant::now());
    }

    /// Get last recorded health status.
    pub fn last_status(&self) -> &HealthStatus {
        &self.last_status
    }

    /// Get current failure counter.
    pub fn consecutive_failures(&self) -> u32 {
        self.consecutive_failures
    }

    /// Set the failure threshold for unhealthy decision.
    ///
    /// Default is 3 consecutive failures.
    pub fn set_failure_threshold(&mut self, threshold: u32) {
        self.failure_threshold = threshold;
    }

    /// Get the failure threshold.
    pub fn failure_threshold(&self) -> u32 {
        self.failure_threshold
    }
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new(Duration::from_secs(5))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_healthy_transitions() {
        let mut monitor = HealthMonitor::new(Duration::from_secs(5));

        let decision = monitor.evaluate_health(HealthStatus::Healthy);
        assert_eq!(decision, HealthDecision::Healthy);
        assert_eq!(monitor.consecutive_failures(), 0);

        // Multiple healthy checks should stay at 0
        let decision = monitor.evaluate_health(HealthStatus::Healthy);
        assert_eq!(decision, HealthDecision::Healthy);
        assert_eq!(monitor.consecutive_failures(), 0);
    }

    #[test]
    fn test_degraded_handling() {
        let mut monitor = HealthMonitor::new(Duration::from_secs(5));

        let decision = monitor.evaluate_health(HealthStatus::Degraded {
            message: "High latency".to_string(),
        });

        assert_eq!(decision, HealthDecision::Degraded);
        assert_eq!(monitor.consecutive_failures(), 1);

        // Second degraded check
        let decision = monitor.evaluate_health(HealthStatus::Degraded {
            message: "High latency".to_string(),
        });

        assert_eq!(decision, HealthDecision::Degraded); // Still degraded, not at threshold yet
        assert_eq!(monitor.consecutive_failures(), 2);
    }

    #[test]
    fn test_unhealthy_triggers_restart() {
        let mut monitor = HealthMonitor::new(Duration::from_secs(5));
        monitor.set_failure_threshold(2); // Lower threshold for testing

        // First unhealthy
        let decision = monitor.evaluate_health(HealthStatus::Unhealthy {
            message: "Out of memory".to_string(),
        });
        assert_eq!(decision, HealthDecision::Degraded); // Not yet at threshold
        assert_eq!(monitor.consecutive_failures(), 1);

        // Second unhealthy - should now trigger restart
        let decision = monitor.evaluate_health(HealthStatus::Unhealthy {
            message: "Out of memory".to_string(),
        });
        assert_eq!(decision, HealthDecision::Unhealthy); // Reached threshold
        assert_eq!(monitor.consecutive_failures(), 2);
    }

    #[test]
    fn test_consecutive_failure_counting() {
        let mut monitor = HealthMonitor::new(Duration::from_secs(5));
        monitor.set_failure_threshold(4); // Set higher threshold to allow counting without triggering

        // Record sequence: Healthy, Degraded, Degraded, Unhealthy
        assert_eq!(
            monitor.evaluate_health(HealthStatus::Healthy),
            HealthDecision::Healthy
        );
        assert_eq!(monitor.consecutive_failures(), 0);

        assert_eq!(
            monitor.evaluate_health(HealthStatus::Degraded {
                message: "test".to_string()
            }),
            HealthDecision::Degraded
        );
        assert_eq!(monitor.consecutive_failures(), 1);

        assert_eq!(
            monitor.evaluate_health(HealthStatus::Degraded {
                message: "test".to_string()
            }),
            HealthDecision::Degraded
        );
        assert_eq!(monitor.consecutive_failures(), 2);

        assert_eq!(
            monitor.evaluate_health(HealthStatus::Unhealthy {
                message: "test".to_string()
            }),
            HealthDecision::Degraded
        );
        assert_eq!(monitor.consecutive_failures(), 3);
    }

    #[test]
    fn test_recovery_resets_counter() {
        let mut monitor = HealthMonitor::new(Duration::from_secs(5));

        // Build up failures
        for _ in 0..5 {
            let _ = monitor.evaluate_health(HealthStatus::Unhealthy {
                message: "test".to_string(),
            });
        }
        assert_eq!(monitor.consecutive_failures(), 5);

        // Recovery clears counter
        monitor.reset_on_recovery();
        assert_eq!(monitor.consecutive_failures(), 0);
        assert_eq!(monitor.last_status(), &HealthStatus::Healthy);
    }

    #[test]
    fn test_unknown_status_handling() {
        let mut monitor = HealthMonitor::new(Duration::from_secs(5));

        let decision = monitor.evaluate_health(HealthStatus::Unknown);
        assert_eq!(decision, HealthDecision::Unknown);

        // Unknown shouldn't increment failure counter
        assert_eq!(monitor.consecutive_failures(), 0);

        // Can recover from unknown with healthy
        let decision = monitor.evaluate_health(HealthStatus::Healthy);
        assert_eq!(decision, HealthDecision::Healthy);
    }

    #[test]
    fn test_check_interval_enforcement() {
        let mut monitor = HealthMonitor::new(Duration::from_millis(100));

        // Should allow check (first time)
        assert!(monitor.should_check_health());

        // Record a check
        monitor.record_check_result(HealthStatus::Healthy);
        assert!(!monitor.should_check_health()); // Interval not elapsed

        // Wait for interval
        std::thread::sleep(Duration::from_millis(110));
        assert!(monitor.should_check_health()); // Interval elapsed
    }

    #[test]
    fn test_healthy_to_unhealthy_transition() {
        let mut monitor = HealthMonitor::new(Duration::from_secs(5));
        monitor.set_failure_threshold(1); // Trigger on first unhealthy

        // Start healthy
        assert_eq!(
            monitor.evaluate_health(HealthStatus::Healthy),
            HealthDecision::Healthy
        );

        // Go unhealthy - should trigger restart
        assert_eq!(
            monitor.evaluate_health(HealthStatus::Unhealthy {
                message: "Critical".to_string()
            }),
            HealthDecision::Unhealthy
        );
    }

    #[test]
    fn test_default_creation() {
        let monitor = HealthMonitor::default();
        assert_eq!(monitor.consecutive_failures(), 0);
        assert_eq!(monitor.failure_threshold(), 3);
        assert!(matches!(monitor.last_status(), HealthStatus::Unknown));
    }

    #[test]
    fn test_last_status_tracking() {
        let mut monitor = HealthMonitor::new(Duration::from_secs(5));

        monitor.record_check_result(HealthStatus::Healthy);
        assert_eq!(monitor.last_status(), &HealthStatus::Healthy);

        let degraded = HealthStatus::Degraded {
            message: "High load".to_string(),
        };
        monitor.record_check_result(degraded.clone());
        assert_eq!(monitor.last_status(), &degraded);
    }

    #[test]
    fn test_failure_threshold_configuration() {
        let mut monitor = HealthMonitor::new(Duration::from_secs(5));

        assert_eq!(monitor.failure_threshold(), 3);

        monitor.set_failure_threshold(5);
        assert_eq!(monitor.failure_threshold(), 5);

        // Verify threshold is applied
        for _ in 0..4 {
            let decision = monitor.evaluate_health(HealthStatus::Unhealthy {
                message: "test".to_string(),
            });
            assert_eq!(decision, HealthDecision::Degraded);
        }

        // 5th unhealthy should trigger
        let decision = monitor.evaluate_health(HealthStatus::Unhealthy {
            message: "test".to_string(),
        });
        assert_eq!(decision, HealthDecision::Unhealthy);
    }
}
