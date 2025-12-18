//! Supervision configuration for ComponentActor restart policies.
//!
//! This module provides the configuration infrastructure for component supervision,
//! enabling automatic restart based on failure patterns and policies.
//!
//! # Overview
//!
//! Components can be supervised with different restart policies:
//! - **Permanent**: Always restart, regardless of exit reason
//! - **Transient**: Restart only on abnormal exit (with error)
//! - **Temporary**: Never restart (expected to complete)
//!
//! This follows the Erlang OTP supervision model.
//!
//! # Example
//!
//! ```rust,ignore
//! use airssys_wasm::actor::{SupervisorConfig, RestartPolicy, BackoffStrategy};
//! use std::time::Duration;
//!
//! // Create a permanent configuration (always restart)
//! let config = SupervisorConfig::permanent()
//!     .with_max_restarts(5)
//!     .with_time_window(Duration::from_secs(120));
//!
//! assert_eq!(config.restart_policy, RestartPolicy::Permanent);
//! assert_eq!(config.max_restarts, 5);
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Restart policy for supervised components (Erlang OTP style).
///
/// Determines whether a component should be restarted when it crashes or exits.
///
/// # Variants
///
/// - **Permanent**: Component should always restart, regardless of exit reason
///   - Use for critical components that must always be running
///   - Examples: Database connections, API servers, core services
///   - Restart on: Both normal exit and error
///
/// - **Transient**: Component should restart only on abnormal exit
///   - Use for workers that may complete successfully
///   - Examples: Job processors, temporary services, batch tasks
///   - Restart on: Error exits only, not normal completion
///
/// - **Temporary**: Component should never restart
///   - Use for one-shot tasks or temporary processes
///   - Examples: Configuration loading, migration runners, one-time jobs
///   - Never restart
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RestartPolicy {
    /// Always restart the component, regardless of exit reason
    Permanent,

    /// Restart only if component exits abnormally (with error)
    Transient,

    /// Never restart the component
    Temporary,
}

impl RestartPolicy {
    /// Returns true if this policy should trigger restart for given exit condition.
    ///
    /// # Parameters
    ///
    /// * `is_error` - Whether the component exited with an error (true) or normally (false)
    ///
    /// # Returns
    ///
    /// `true` if the component should be restarted, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use airssys_wasm::actor::RestartPolicy;
    ///
    /// let permanent = RestartPolicy::Permanent;
    /// assert!(permanent.should_restart(true));   // Error → restart
    /// assert!(permanent.should_restart(false));  // Normal exit → restart
    ///
    /// let transient = RestartPolicy::Transient;
    /// assert!(transient.should_restart(true));   // Error → restart
    /// assert!(!transient.should_restart(false)); // Normal exit → no restart
    ///
    /// let temporary = RestartPolicy::Temporary;
    /// assert!(!temporary.should_restart(true));  // Error → no restart
    /// assert!(!temporary.should_restart(false)); // Normal exit → no restart
    /// ```
    pub fn should_restart(&self, is_error: bool) -> bool {
        match self {
            RestartPolicy::Permanent => true,
            RestartPolicy::Transient => is_error,
            RestartPolicy::Temporary => false,
        }
    }
}

impl std::fmt::Display for RestartPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RestartPolicy::Permanent => write!(f, "Permanent"),
            RestartPolicy::Transient => write!(f, "Transient"),
            RestartPolicy::Temporary => write!(f, "Temporary"),
        }
    }
}

/// Backoff strategy for restart attempts.
///
/// Controls the delay between consecutive restart attempts to prevent
/// rapid restart loops that could overwhelm the system.
///
/// # Variants
///
/// - **Immediate**: No delay between restarts (useful for development)
/// - **Linear**: Delay increases linearly with attempt count
/// - **Exponential**: Delay increases exponentially with attempt count (recommended)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackoffStrategy {
    /// No delay between restarts (useful for development/testing).
    ///
    /// Every restart attempt happens immediately after the previous one completes.
    /// **Warning**: Can cause rapid restart loops. Use Temporary or Transient policies
    /// with max_restarts=1 for safety.
    Immediate,

    /// Linear backoff: `base_delay * attempt_count`
    ///
    /// Delay increases linearly with each restart attempt.
    ///
    /// # Examples
    ///
    /// With `base_delay = 100ms`:
    /// - Attempt 1: 100ms delay
    /// - Attempt 2: 200ms delay
    /// - Attempt 3: 300ms delay
    /// - Attempt 4: 400ms delay
    ///
    /// Good for: Moderate load scenarios, predictable delays
    Linear {
        /// Base delay per attempt (e.g., 100ms)
        base_delay: Duration,
    },

    /// Exponential backoff: `base_delay * (multiplier ^ attempt_count)`, capped at max_delay
    ///
    /// Delay grows exponentially with each restart attempt, preventing
    /// system overload when components are failing repeatedly.
    ///
    /// # Examples
    ///
    /// With `base_delay = 100ms`, `multiplier = 2.0`, `max_delay = 30s`:
    /// - Attempt 0: 100ms
    /// - Attempt 1: 200ms
    /// - Attempt 2: 400ms
    /// - Attempt 3: 800ms
    /// - Attempt 4: 1.6s
    /// - Attempt 5: 3.2s
    /// - Attempt 6: 6.4s
    /// - Attempt 7: 12.8s
    /// - Attempt 8: 25.6s
    /// - Attempt 9+: 30s (capped at max_delay)
    ///
    /// Good for: Production systems, handles cascading failures gracefully
    ///
    /// **Recommended**: Use `multiplier = 1.5` for balanced backoff
    Exponential {
        /// Base delay for first attempt (e.g., 100ms)
        base_delay: Duration,

        /// Multiplier per attempt (e.g., 1.5 or 2.0)
        multiplier: f32,

        /// Maximum delay cap (e.g., 30 seconds)
        max_delay: Duration,
    },
}

impl BackoffStrategy {
    /// Calculate delay for a given restart attempt number.
    ///
    /// # Parameters
    ///
    /// * `attempt` - 0-based attempt number (0 = first attempt, 1 = second, etc.)
    ///
    /// # Returns
    ///
    /// The duration to wait before this restart attempt
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use airssys_wasm::actor::BackoffStrategy;
    /// use std::time::Duration;
    ///
    /// let strategy = BackoffStrategy::Exponential {
    ///     base_delay: Duration::from_millis(100),
    ///     multiplier: 2.0,
    ///     max_delay: Duration::from_secs(30),
    /// };
    ///
    /// assert_eq!(strategy.calculate_delay(0), Duration::from_millis(100));
    /// assert_eq!(strategy.calculate_delay(1), Duration::from_millis(200));
    /// assert_eq!(strategy.calculate_delay(8), Duration::from_secs(30)); // Capped
    /// ```
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        match self {
            BackoffStrategy::Immediate => Duration::from_millis(0),

            BackoffStrategy::Linear { base_delay } => base_delay.saturating_mul(attempt.max(1)),

            BackoffStrategy::Exponential {
                base_delay,
                multiplier,
                max_delay,
            } => {
                let calculated =
                    (base_delay.as_millis() as f32 * multiplier.powi(attempt as i32)) as u64;
                Duration::from_millis(calculated).min(*max_delay)
            }
        }
    }
}

/// Supervision configuration for ComponentActor.
///
/// Defines how a component should be supervised, including restart policies,
/// maximum restart attempts, and backoff strategies.
///
/// # Default Configuration
///
/// The default configuration uses:
/// - **Restart Policy**: Permanent (always restart)
/// - **Max Restarts**: 3 restarts in 60 seconds
/// - **Backoff Strategy**: Exponential (1.5x multiplier, 30s cap)
/// - **Shutdown Timeout**: 5 seconds
/// - **Startup Timeout**: 10 seconds
///
/// # Example
///
/// ```rust,ignore
/// use airssys_wasm::actor::{SupervisorConfig, RestartPolicy, BackoffStrategy};
/// use std::time::Duration;
///
/// // Critical database component
/// let db_config = SupervisorConfig::permanent()
///     .with_max_restarts(5)
///     .with_time_window(Duration::from_secs(120))
///     .with_shutdown_timeout(Duration::from_secs(10));
///
/// // Worker pool task
/// let worker_config = SupervisorConfig::transient()
///     .with_max_restarts(2)
///     .with_backoff(BackoffStrategy::Linear {
///         base_delay: Duration::from_millis(100),
///     });
///
/// // One-shot initialization
/// let init_config = SupervisorConfig::temporary();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisorConfig {
    /// When to restart this component.
    ///
    /// - Permanent: Always restart
    /// - Transient: Restart on error only
    /// - Temporary: Never restart
    pub restart_policy: RestartPolicy,

    /// Maximum restarts allowed in `time_window` duration.
    ///
    /// Once this limit is exceeded, the component enters RestartLimitExceeded state
    /// and will not be restarted again until the time window expires.
    ///
    /// # Example
    ///
    /// `max_restarts = 3` means "no more than 3 restarts in the time_window"
    pub max_restarts: u32,

    /// Time window for `max_restarts` counting.
    ///
    /// Common value: 60 seconds (Erlang OTP standard)
    ///
    /// Restarts outside this window don't count toward the limit.
    pub time_window: Duration,

    /// Backoff strategy between restart attempts.
    ///
    /// Controls the delay between consecutive restarts to prevent
    /// rapid restart loops.
    pub backoff_strategy: BackoffStrategy,

    /// Maximum time to wait for component shutdown.
    ///
    /// If the component doesn't shut down within this time,
    /// it will be forcefully terminated.
    pub shutdown_timeout: Duration,

    /// Maximum time to wait for component startup.
    ///
    /// If the component doesn't start within this time,
    /// the start attempt will be cancelled.
    pub startup_timeout: Duration,
}

impl Default for SupervisorConfig {
    /// Create default supervision configuration (Permanent, 3 restarts/60s).
    fn default() -> Self {
        Self {
            restart_policy: RestartPolicy::Permanent,
            max_restarts: 3,
            time_window: Duration::from_secs(60),
            backoff_strategy: BackoffStrategy::Exponential {
                base_delay: Duration::from_millis(100),
                multiplier: 1.5,
                max_delay: Duration::from_secs(30),
            },
            shutdown_timeout: Duration::from_secs(5),
            startup_timeout: Duration::from_secs(10),
        }
    }
}

impl SupervisorConfig {
    /// Create a permanent configuration (always restart).
    ///
    /// Use for critical components that must always be running.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use airssys_wasm::actor::SupervisorConfig;
    ///
    /// let config = SupervisorConfig::permanent();
    /// assert_eq!(config.restart_policy, RestartPolicy::Permanent);
    /// ```
    pub fn permanent() -> Self {
        Self {
            restart_policy: RestartPolicy::Permanent,
            ..Default::default()
        }
    }

    /// Create a transient configuration (restart on error only).
    ///
    /// Use for workers that may complete successfully.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use airssys_wasm::actor::SupervisorConfig;
    ///
    /// let config = SupervisorConfig::transient();
    /// assert_eq!(config.restart_policy, RestartPolicy::Transient);
    /// ```
    pub fn transient() -> Self {
        Self {
            restart_policy: RestartPolicy::Transient,
            ..Default::default()
        }
    }

    /// Create a temporary configuration (never restart).
    ///
    /// Use for one-shot tasks or temporary processes.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use airssys_wasm::actor::SupervisorConfig;
    ///
    /// let config = SupervisorConfig::temporary();
    /// assert_eq!(config.restart_policy, RestartPolicy::Temporary);
    /// ```
    pub fn temporary() -> Self {
        Self {
            restart_policy: RestartPolicy::Temporary,
            ..Default::default()
        }
    }

    /// Builder method: Set restart policy.
    pub fn with_restart_policy(mut self, policy: RestartPolicy) -> Self {
        self.restart_policy = policy;
        self
    }

    /// Builder method: Set maximum restart count.
    pub fn with_max_restarts(mut self, max: u32) -> Self {
        self.max_restarts = max;
        self
    }

    /// Builder method: Set time window for max restart counting.
    pub fn with_time_window(mut self, window: Duration) -> Self {
        self.time_window = window;
        self
    }

    /// Builder method: Set backoff strategy.
    pub fn with_backoff(mut self, strategy: BackoffStrategy) -> Self {
        self.backoff_strategy = strategy;
        self
    }

    /// Builder method: Set shutdown timeout.
    pub fn with_shutdown_timeout(mut self, timeout: Duration) -> Self {
        self.shutdown_timeout = timeout;
        self
    }

    /// Builder method: Set startup timeout.
    pub fn with_startup_timeout(mut self, timeout: Duration) -> Self {
        self.startup_timeout = timeout;
        self
    }

    /// Check if maximum restart limit is exceeded.
    ///
    /// Returns true if the component has exceeded the maximum number of restarts
    /// within the configured time window.
    ///
    /// # Parameters
    ///
    /// * `restart_attempts` - Vector of (timestamp, is_error) tuples
    ///
    /// # Returns
    ///
    /// `true` if restart limit exceeded, `false` otherwise
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use airssys_wasm::actor::SupervisorConfig;
    /// use chrono::Utc;
    /// use std::time::Duration;
    ///
    /// let config = SupervisorConfig {
    ///     max_restarts: 3,
    ///     time_window: Duration::from_secs(60),
    ///     ..Default::default()
    /// };
    ///
    /// let now = Utc::now();
    /// let recent_restarts = vec![
    ///     (now - Duration::from_secs(30), true),
    ///     (now - Duration::from_secs(20), true),
    ///     (now - Duration::from_secs(10), true),
    /// ];
    ///
    /// // 3 restarts in 60 seconds = limit exceeded
    /// assert!(config.check_restart_limit(&recent_restarts));
    /// ```
    pub fn check_restart_limit(&self, restart_attempts: &[(DateTime<Utc>, bool)]) -> bool {
        let now = Utc::now();
        let recent_restarts = restart_attempts
            .iter()
            .filter(|(timestamp, _)| {
                now.signed_duration_since(*timestamp).num_seconds()
                    < self.time_window.as_secs() as i64
            })
            .count();

        recent_restarts >= self.max_restarts as usize
    }

    /// Calculate the delay before next restart attempt.
    ///
    /// # Parameters
    ///
    /// * `attempt_count` - Number of restart attempts so far (0 = first attempt)
    ///
    /// # Returns
    ///
    /// Duration to wait before the next restart attempt
    pub fn calculate_next_restart_delay(&self, attempt_count: u32) -> Duration {
        self.backoff_strategy.calculate_delay(attempt_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // RestartPolicy Tests
    // ============================================================================

    #[test]
    fn test_restart_policy_permanent_always_restarts() {
        let policy = RestartPolicy::Permanent;
        assert!(policy.should_restart(true)); // Error exit
        assert!(policy.should_restart(false)); // Normal exit
    }

    #[test]
    fn test_restart_policy_transient_error_only() {
        let policy = RestartPolicy::Transient;
        assert!(policy.should_restart(true)); // Error exit
        assert!(!policy.should_restart(false)); // Normal exit
    }

    #[test]
    fn test_restart_policy_temporary_never_restarts() {
        let policy = RestartPolicy::Temporary;
        assert!(!policy.should_restart(true)); // Error exit
        assert!(!policy.should_restart(false)); // Normal exit
    }

    #[test]
    fn test_restart_policy_display() {
        assert_eq!(RestartPolicy::Permanent.to_string(), "Permanent");
        assert_eq!(RestartPolicy::Transient.to_string(), "Transient");
        assert_eq!(RestartPolicy::Temporary.to_string(), "Temporary");
    }

    // ============================================================================
    // BackoffStrategy Tests
    // ============================================================================

    #[test]
    fn test_backoff_immediate_no_delay() {
        let strategy = BackoffStrategy::Immediate;
        assert_eq!(strategy.calculate_delay(1), Duration::from_millis(0));
        assert_eq!(strategy.calculate_delay(5), Duration::from_millis(0));
        assert_eq!(strategy.calculate_delay(100), Duration::from_millis(0));
    }

    #[test]
    fn test_backoff_linear_scaling() {
        let strategy = BackoffStrategy::Linear {
            base_delay: Duration::from_millis(100),
        };
        assert_eq!(strategy.calculate_delay(1), Duration::from_millis(100));
        assert_eq!(strategy.calculate_delay(2), Duration::from_millis(200));
        assert_eq!(strategy.calculate_delay(3), Duration::from_millis(300));
        assert_eq!(strategy.calculate_delay(10), Duration::from_millis(1000));
    }

    #[test]
    fn test_backoff_exponential_scaling() {
        let strategy = BackoffStrategy::Exponential {
            base_delay: Duration::from_millis(100),
            multiplier: 2.0,
            max_delay: Duration::from_secs(30),
        };
        // 100 * 2^0 = 100ms
        assert_eq!(strategy.calculate_delay(0), Duration::from_millis(100));
        // 100 * 2^1 = 200ms
        assert_eq!(strategy.calculate_delay(1), Duration::from_millis(200));
        // 100 * 2^2 = 400ms
        assert_eq!(strategy.calculate_delay(2), Duration::from_millis(400));
        // 100 * 2^8 = 25600ms
        assert_eq!(strategy.calculate_delay(8), Duration::from_millis(25600));
        // 100 * 2^9 = 51200ms, capped at 30s
        assert_eq!(strategy.calculate_delay(9), Duration::from_secs(30));
    }

    #[test]
    fn test_backoff_exponential_with_1_5_multiplier() {
        let strategy = BackoffStrategy::Exponential {
            base_delay: Duration::from_millis(100),
            multiplier: 1.5,
            max_delay: Duration::from_secs(30),
        };
        // 100 * 1.5^0 = 100ms
        assert_eq!(strategy.calculate_delay(0), Duration::from_millis(100));
        // 100 * 1.5^1 = 150ms
        assert_eq!(strategy.calculate_delay(1), Duration::from_millis(150));
        // 100 * 1.5^2 = 225ms
        assert_eq!(strategy.calculate_delay(2), Duration::from_millis(225));
    }

    // ============================================================================
    // SupervisorConfig Tests
    // ============================================================================

    #[test]
    fn test_supervisor_config_default() {
        let config = SupervisorConfig::default();
        assert_eq!(config.restart_policy, RestartPolicy::Permanent);
        assert_eq!(config.max_restarts, 3);
        assert_eq!(config.time_window, Duration::from_secs(60));
        assert_eq!(config.shutdown_timeout, Duration::from_secs(5));
        assert_eq!(config.startup_timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_supervisor_config_permanent_builder() {
        let config = SupervisorConfig::permanent();
        assert_eq!(config.restart_policy, RestartPolicy::Permanent);
    }

    #[test]
    fn test_supervisor_config_transient_builder() {
        let config = SupervisorConfig::transient();
        assert_eq!(config.restart_policy, RestartPolicy::Transient);
    }

    #[test]
    fn test_supervisor_config_temporary_builder() {
        let config = SupervisorConfig::temporary();
        assert_eq!(config.restart_policy, RestartPolicy::Temporary);
    }

    #[test]
    fn test_supervisor_config_with_max_restarts() {
        let config = SupervisorConfig::default().with_max_restarts(5);
        assert_eq!(config.max_restarts, 5);
    }

    #[test]
    fn test_supervisor_config_with_time_window() {
        let window = Duration::from_secs(120);
        let config = SupervisorConfig::default().with_time_window(window);
        assert_eq!(config.time_window, window);
    }

    #[test]
    fn test_supervisor_config_with_shutdown_timeout() {
        let timeout = Duration::from_secs(10);
        let config = SupervisorConfig::default().with_shutdown_timeout(timeout);
        assert_eq!(config.shutdown_timeout, timeout);
    }

    #[test]
    fn test_supervisor_config_with_startup_timeout() {
        let timeout = Duration::from_secs(15);
        let config = SupervisorConfig::default().with_startup_timeout(timeout);
        assert_eq!(config.startup_timeout, timeout);
    }

    #[test]
    fn test_supervisor_config_builder_chain() {
        let config = SupervisorConfig::permanent()
            .with_max_restarts(5)
            .with_time_window(Duration::from_secs(120))
            .with_shutdown_timeout(Duration::from_secs(10));

        assert_eq!(config.restart_policy, RestartPolicy::Permanent);
        assert_eq!(config.max_restarts, 5);
        assert_eq!(config.time_window, Duration::from_secs(120));
        assert_eq!(config.shutdown_timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_check_restart_limit_within_window() {
        let config = SupervisorConfig {
            max_restarts: 3,
            time_window: Duration::from_secs(60),
            ..Default::default()
        };

        let now = Utc::now();
        let recent_restarts = vec![
            (now - Duration::from_secs(30), true),
            (now - Duration::from_secs(20), true),
            (now - Duration::from_secs(10), true),
        ];

        // 3 restarts in 60 seconds = limit exceeded
        assert!(config.check_restart_limit(&recent_restarts));
    }

    #[test]
    fn test_check_restart_limit_outside_window() {
        let config = SupervisorConfig {
            max_restarts: 3,
            time_window: Duration::from_secs(60),
            ..Default::default()
        };

        let now = Utc::now();
        let old_restarts = vec![
            (now - Duration::from_secs(90), true),
            (now - Duration::from_secs(80), true),
            (now - Duration::from_secs(70), true),
        ];

        // All restarts outside 60s window = limit not exceeded
        assert!(!config.check_restart_limit(&old_restarts));
    }

    #[test]
    fn test_check_restart_limit_mixed_times() {
        let config = SupervisorConfig {
            max_restarts: 3,
            time_window: Duration::from_secs(60),
            ..Default::default()
        };

        let now = Utc::now();
        let mixed_restarts = vec![
            (now - Duration::from_secs(90), true), // Outside window
            (now - Duration::from_secs(30), true), // Inside window
            (now - Duration::from_secs(20), true), // Inside window
            (now - Duration::from_secs(10), true), // Inside window
        ];

        // 3 restarts in window = limit exceeded
        assert!(config.check_restart_limit(&mixed_restarts));
    }

    #[test]
    fn test_calculate_next_restart_delay_immediate() {
        let config = SupervisorConfig {
            backoff_strategy: BackoffStrategy::Immediate,
            ..Default::default()
        };

        assert_eq!(
            config.calculate_next_restart_delay(0),
            Duration::from_millis(0)
        );
        assert_eq!(
            config.calculate_next_restart_delay(5),
            Duration::from_millis(0)
        );
    }

    #[test]
    fn test_calculate_next_restart_delay_linear() {
        let config = SupervisorConfig {
            backoff_strategy: BackoffStrategy::Linear {
                base_delay: Duration::from_millis(100),
            },
            ..Default::default()
        };

        assert_eq!(
            config.calculate_next_restart_delay(1),
            Duration::from_millis(100)
        );
        assert_eq!(
            config.calculate_next_restart_delay(2),
            Duration::from_millis(200)
        );
    }

    #[test]
    fn test_calculate_next_restart_delay_exponential() {
        let config = SupervisorConfig {
            backoff_strategy: BackoffStrategy::Exponential {
                base_delay: Duration::from_millis(100),
                multiplier: 2.0,
                max_delay: Duration::from_secs(30),
            },
            ..Default::default()
        };

        assert_eq!(
            config.calculate_next_restart_delay(0),
            Duration::from_millis(100)
        );
        assert_eq!(
            config.calculate_next_restart_delay(1),
            Duration::from_millis(200)
        );
    }
}
