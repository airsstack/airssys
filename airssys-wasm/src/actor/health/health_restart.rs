//! Health-based restart triggering for ComponentActor.
//!
//! This module integrates ComponentActor health checks with SupervisorNode
//! health monitoring, enabling automatic restart when health checks fail.
//!
//! # Architecture (ADR-WASM-018)
//!
//! Health monitoring operates across layers:
//! - **Layer 2** (ComponentActor): Implements health_check() via Child trait
//! - **Layer 3** (SupervisorNode): Monitors health and triggers restart
//! - **This module**: Configuration and integration layer
//!
//! # Example
//!
//! ```rust,ignore
//! use airssys_wasm::actor::{HealthRestartConfig, ComponentHealthCheck};
//! use std::time::Duration;
//!
//! let config = HealthRestartConfig {
//!     check_interval: Duration::from_secs(30),
//!     failure_threshold: 3,
//!     enabled: true,
//! };
//!
//! // SupervisorNode will automatically check health and restart on failure
//! ```

// Layer 1: Standard library imports
use std::time::Duration;

// Layer 2: Third-party crate imports
// (none)

// Layer 3: Internal module imports
// (none)

/// Health-based restart configuration.
///
/// Defines parameters for periodic health checking and failure threshold management.
///
/// # Design Philosophy
///
/// Following YAGNI principles (§6.1), health monitoring is an optional feature
/// that can be enabled per supervisor instance. This structure encapsulates all
/// health-related configuration.
///
/// # Examples
///
/// ```rust
/// use airssys_wasm::actor::HealthRestartConfig;
/// use std::time::Duration;
///
/// // Default configuration: check every 5 seconds, restart after 3 failures
/// let config = HealthRestartConfig::default();
/// assert_eq!(config.check_interval, Duration::from_secs(5));
/// assert_eq!(config.failure_threshold, 3);
/// assert!(config.enabled);
///
/// // Custom configuration
/// let custom = HealthRestartConfig {
///     check_interval: Duration::from_secs(30),
///     failure_threshold: 5,
///     enabled: true,
/// };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HealthRestartConfig {
    /// Health check interval
    ///
    /// How often to perform health checks on supervised components.
    /// Shorter intervals provide faster failure detection but higher overhead.
    ///
    /// **Recommended:** 5-30 seconds for most applications
    pub check_interval: Duration,

    /// Number of consecutive failures before restart
    ///
    /// Prevents spurious restarts due to transient health check failures.
    /// Higher thresholds are more tolerant but slower to detect real failures.
    ///
    /// **Recommended:** 3-5 failures
    pub failure_threshold: u32,

    /// Whether to enable health-based restarts
    ///
    /// When false, health checks are still performed but won't trigger restarts.
    /// Useful for monitoring-only scenarios or gradual rollout.
    pub enabled: bool,
}

impl HealthRestartConfig {
    /// Create a new health restart configuration.
    ///
    /// # Parameters
    ///
    /// - `check_interval`: How often to perform health checks
    /// - `failure_threshold`: Number of consecutive failures before restart
    /// - `enabled`: Whether health-based restarts are enabled
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::actor::HealthRestartConfig;
    /// use std::time::Duration;
    ///
    /// let config = HealthRestartConfig::new(
    ///     Duration::from_secs(10),  // Check every 10 seconds
    ///     3,                        // Restart after 3 failures
    ///     true,                     // Enable health-based restart
    /// );
    /// ```
    pub fn new(check_interval: Duration, failure_threshold: u32, enabled: bool) -> Self {
        Self {
            check_interval,
            failure_threshold,
            enabled,
        }
    }

    /// Create configuration with health-based restart disabled.
    ///
    /// Health checks will still be performed for monitoring, but won't trigger restarts.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::actor::HealthRestartConfig;
    ///
    /// let config = HealthRestartConfig::disabled();
    /// assert!(!config.enabled);
    /// ```
    pub fn disabled() -> Self {
        Self {
            check_interval: Duration::from_secs(5),
            failure_threshold: 3,
            enabled: false,
        }
    }

    /// Returns true if health-based restarts are enabled.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::actor::HealthRestartConfig;
    ///
    /// let config = HealthRestartConfig::default();
    /// assert!(config.is_enabled());
    ///
    /// let disabled = HealthRestartConfig::disabled();
    /// assert!(!disabled.is_enabled());
    /// ```
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Returns the check interval.
    pub fn interval(&self) -> Duration {
        self.check_interval
    }

    /// Returns the failure threshold.
    pub fn threshold(&self) -> u32 {
        self.failure_threshold
    }
}

impl Default for HealthRestartConfig {
    /// Default configuration: 5s interval, 3 failure threshold, enabled.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::actor::HealthRestartConfig;
    /// use std::time::Duration;
    ///
    /// let config = HealthRestartConfig::default();
    /// assert_eq!(config.check_interval, Duration::from_secs(5));
    /// assert_eq!(config.failure_threshold, 3);
    /// assert!(config.enabled);
    /// ```
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(5),
            failure_threshold: 3,
            enabled: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_restart_config_default() {
        let config = HealthRestartConfig::default();
        assert_eq!(config.check_interval, Duration::from_secs(5));
        assert_eq!(config.failure_threshold, 3);
        assert!(config.enabled);
        assert!(config.is_enabled());
    }

    #[test]
    fn test_health_restart_config_new() {
        let config = HealthRestartConfig::new(Duration::from_secs(30), 5, true);
        assert_eq!(config.check_interval, Duration::from_secs(30));
        assert_eq!(config.failure_threshold, 5);
        assert!(config.enabled);
    }

    #[test]
    fn test_health_restart_config_disabled() {
        let config = HealthRestartConfig::disabled();
        assert!(!config.enabled);
        assert!(!config.is_enabled());
    }

    #[test]
    fn test_health_restart_config_accessors() {
        let config = HealthRestartConfig::new(Duration::from_secs(15), 4, true);
        assert_eq!(config.interval(), Duration::from_secs(15));
        assert_eq!(config.threshold(), 4);
    }

    #[test]
    fn test_health_restart_config_clone() {
        let config = HealthRestartConfig::default();
        let cloned = config;
        assert_eq!(config, cloned);
    }

    #[test]
    fn test_health_restart_config_debug() {
        let config = HealthRestartConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("HealthRestartConfig"));
        assert!(debug_str.contains("check_interval"));
        assert!(debug_str.contains("failure_threshold"));
        assert!(debug_str.contains("enabled"));
    }
}
