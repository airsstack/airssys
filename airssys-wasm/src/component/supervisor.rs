//! Supervisor configuration for WASM component actor fault tolerance.
//!
//! Provides configuration for supervising WASM component actors using the
//! airssys-rt supervisor framework. SupervisorConfig maps WASM-specific
//! supervision needs to airssys-rt types.
//!
//! # Architecture
//!
//! SupervisorConfig is part of Layer 3A (component/ module). It:
//! - Uses types from `airssys-rt` (RestartPolicy, ShutdownPolicy, RestartBackoff)
//! - Provides WASM-appropriate defaults
//! - Is consumed by `system/` module (Layer 4) to construct SupervisorNode instances
//!
//! # Design Note
//!
//! SupervisorConfig does NOT create a `SupervisorNode` directly because
//! `SupervisorNode<S, C, M>` requires three generic type parameters that are
//! determined at construction time by the system/ module. Instead, this config
//! provides accessor methods and conversion methods for the individual settings.
//!
//! # Module Boundary Rules
//!
//! - CAN import: `airssys-rt`
//! - CANNOT import: `runtime/`, `security/`, `system/`
//!
//! # References
//!
//! - ADR-WASM-031: Component & Messaging Module Design
//! - ADR-WASM-023: Module Boundary Enforcement

// Layer 1: Standard library imports
use std::time::Duration;

// Layer 2: Third-party crate imports
use airssys_rt::supervisor::{RestartBackoff, RestartPolicy, ShutdownPolicy};
use thiserror::Error;

// Layer 3: Internal module imports
// (none needed for this module)

/// Errors that can occur when creating or validating supervisor configuration.
#[derive(Debug, Clone, Error)]
#[non_exhaustive]
pub enum SupervisorConfigError {
    /// Maximum restarts must be greater than zero.
    #[error("max_restarts must be > 0, got {0}")]
    InvalidMaxRestarts(u32),

    /// Restart window must be greater than zero.
    #[error("restart_window must be > 0ms, got {0:?}")]
    InvalidRestartWindow(Duration),

    /// Base delay must not exceed max delay.
    #[error("base_delay ({0:?}) must be <= max_delay ({1:?})")]
    InvalidBackoffRange(Duration, Duration),

    /// Shutdown timeout must be reasonable (> 0 for graceful).
    #[error("graceful shutdown timeout must be > 0ms, got {0:?}")]
    InvalidShutdownTimeout(Duration),
}

/// Supervision strategy selection for WASM component actors.
///
/// Maps to airssys-rt supervision strategies. The system/ module will
/// pattern-match on this enum to select the appropriate generic strategy
/// type parameter for SupervisorNode.
///
/// # Strategy Guide
///
/// - **OneForOne**: Use when WASM components are independent (most common)
/// - **OneForAll**: Use when components share state and must restart together
/// - **RestForOne**: Use when components have startup dependencies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum WasmSupervisionStrategy {
    /// Restart only the failed component. Other components continue unaffected.
    ///
    /// This is the default strategy for WASM components since they are
    /// typically independent, sandboxed processes.
    #[default]
    OneForOne,

    /// Restart all components when one fails.
    ///
    /// Use when components share state or have interdependencies that
    /// require a coordinated restart.
    OneForAll,

    /// Restart the failed component and all components started after it.
    ///
    /// Use when components have startup ordering dependencies.
    RestForOne,
}

/// Configuration for restart backoff delays.
///
/// Controls the exponential backoff behavior between restart attempts.
/// Maps to `RestartBackoff::with_delays()` in airssys-rt.
///
/// # Exponential Backoff Formula
///
/// ```text
/// delay = base_delay * 2^(min(restart_count, 10))
/// delay = min(delay, max_delay)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackoffConfig {
    /// Base delay for the first restart attempt (doubles each subsequent restart).
    base_delay: Duration,
    /// Maximum delay cap to prevent excessively long waits.
    max_delay: Duration,
}

impl BackoffConfig {
    /// Creates a new backoff configuration.
    ///
    /// # Arguments
    ///
    /// * `base_delay` - Initial delay (doubles per restart)
    /// * `max_delay` - Maximum delay cap
    ///
    /// # Note
    ///
    /// No validation is performed here (e.g., `base_delay <= max_delay` is not checked).
    /// Validation is deferred to [`SupervisorConfig::validate`] or
    /// [`SupervisorConfigBuilder::build`].
    pub fn new(base_delay: Duration, max_delay: Duration) -> Self {
        Self {
            base_delay,
            max_delay,
        }
    }

    /// Returns the base delay.
    pub fn base_delay(&self) -> Duration {
        self.base_delay
    }

    /// Returns the max delay.
    pub fn max_delay(&self) -> Duration {
        self.max_delay
    }
}

impl Default for BackoffConfig {
    fn default() -> Self {
        Self {
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
        }
    }
}

/// Supervisor configuration for WASM component actors.
///
/// Defines how WASM component actors should be supervised, including
/// restart policies, shutdown behavior, and backoff strategies.
///
/// # WASM-Specific Defaults
///
/// - Strategy: OneForOne (components are independent)
/// - Restart policy: Transient (restart on crash, not on normal exit)
/// - Shutdown: Graceful(5s) (allow WASM cleanup)
/// - Max restarts: 3 in 60s window
/// - Backoff: Exponential 100ms base, 30s max
///
/// # Usage
///
/// ```rust,ignore
/// use airssys_wasm::component::supervisor::SupervisorConfig;
///
/// // Use defaults
/// let config = SupervisorConfig::default();
///
/// // Use new() with custom restart limits
/// let config = SupervisorConfig::new(5, Duration::from_secs(120))?;
///
/// // Use builder for full customization
/// let config = SupervisorConfig::builder()
///     .max_restarts(5)
///     .restart_window(Duration::from_secs(120))
///     .build()?;
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupervisorConfig {
    /// Supervision strategy (default: OneForOne)
    strategy: WasmSupervisionStrategy,
    /// Per-child restart policy (default: Transient)
    restart_policy: RestartPolicy,
    /// Shutdown policy for stopping children (default: Graceful(5s))
    shutdown_policy: ShutdownPolicy,
    /// Maximum restarts within the restart window (default: 3)
    max_restarts: u32,
    /// Sliding window for counting restarts (default: 60s)
    restart_window: Duration,
    /// Backoff delay configuration (default: 100ms base, 30s max)
    backoff: BackoffConfig,
}

impl SupervisorConfig {
    /// Creates a new SupervisorConfig with the given restart limits.
    ///
    /// Uses defaults for strategy, restart_policy, shutdown_policy, and backoff.
    ///
    /// # Errors
    ///
    /// Returns `SupervisorConfigError` if `max_restarts` is 0 or `restart_window` is zero.
    pub fn new(max_restarts: u32, restart_window: Duration) -> Result<Self, SupervisorConfigError> {
        let config = Self {
            max_restarts,
            restart_window,
            ..Self::default()
        };
        config.validate()?;
        Ok(config)
    }

    /// Returns a builder for constructing SupervisorConfig.
    pub fn builder() -> SupervisorConfigBuilder {
        SupervisorConfigBuilder::new()
    }

    /// Validates the configuration.
    ///
    /// # Errors
    ///
    /// Returns `SupervisorConfigError` if any field has an invalid value.
    pub fn validate(&self) -> Result<(), SupervisorConfigError> {
        if self.max_restarts == 0 {
            return Err(SupervisorConfigError::InvalidMaxRestarts(self.max_restarts));
        }
        if self.restart_window.is_zero() {
            return Err(SupervisorConfigError::InvalidRestartWindow(
                self.restart_window,
            ));
        }
        if self.backoff.base_delay > self.backoff.max_delay {
            return Err(SupervisorConfigError::InvalidBackoffRange(
                self.backoff.base_delay,
                self.backoff.max_delay,
            ));
        }
        if let ShutdownPolicy::Graceful(timeout) = self.shutdown_policy {
            if timeout.is_zero() {
                return Err(SupervisorConfigError::InvalidShutdownTimeout(timeout));
            }
        }
        Ok(())
    }

    /// Creates an airssys-rt `RestartBackoff` from this configuration.
    ///
    /// This is the primary integration point with airssys-rt. The system/ module
    /// calls this to get a configured `RestartBackoff` for the supervisor.
    pub fn to_restart_backoff(&self) -> RestartBackoff {
        RestartBackoff::with_delays(
            self.max_restarts,
            self.restart_window,
            self.backoff.base_delay,
            self.backoff.max_delay,
        )
    }

    // --- Accessor methods ---

    /// Returns the supervision strategy.
    pub fn strategy(&self) -> WasmSupervisionStrategy {
        self.strategy
    }

    /// Returns the restart policy.
    pub fn restart_policy(&self) -> RestartPolicy {
        self.restart_policy
    }

    /// Returns the shutdown policy.
    pub fn shutdown_policy(&self) -> ShutdownPolicy {
        self.shutdown_policy
    }

    /// Returns the maximum number of restarts within the window.
    pub fn max_restarts(&self) -> u32 {
        self.max_restarts
    }

    /// Returns the restart window duration.
    pub fn restart_window(&self) -> Duration {
        self.restart_window
    }

    /// Returns the backoff configuration.
    pub fn backoff(&self) -> &BackoffConfig {
        &self.backoff
    }
}

impl Default for SupervisorConfig {
    fn default() -> Self {
        Self {
            strategy: WasmSupervisionStrategy::OneForOne,
            restart_policy: RestartPolicy::Transient,
            shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
            max_restarts: 3,
            restart_window: Duration::from_secs(60),
            backoff: BackoffConfig::default(),
        }
    }
}

/// Builder for constructing SupervisorConfig with fluent API.
///
/// # Examples
///
/// ```rust,ignore
/// let config = SupervisorConfig::builder()
///     .strategy(WasmSupervisionStrategy::OneForAll)
///     .restart_policy(RestartPolicy::Permanent)
///     .max_restarts(5)
///     .restart_window(Duration::from_secs(120))
///     .backoff(BackoffConfig::new(
///         Duration::from_millis(200),
///         Duration::from_secs(60),
///     ))
///     .build()?;
/// ```
#[derive(Debug)]
pub struct SupervisorConfigBuilder {
    strategy: WasmSupervisionStrategy,
    restart_policy: RestartPolicy,
    shutdown_policy: ShutdownPolicy,
    max_restarts: u32,
    restart_window: Duration,
    backoff: BackoffConfig,
}

impl SupervisorConfigBuilder {
    /// Creates a new builder with WASM-appropriate defaults.
    fn new() -> Self {
        let defaults = SupervisorConfig::default();
        Self {
            strategy: defaults.strategy,
            restart_policy: defaults.restart_policy,
            shutdown_policy: defaults.shutdown_policy,
            max_restarts: defaults.max_restarts,
            restart_window: defaults.restart_window,
            backoff: defaults.backoff,
        }
    }

    /// Sets the supervision strategy.
    pub fn strategy(mut self, strategy: WasmSupervisionStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Sets the restart policy.
    pub fn restart_policy(mut self, policy: RestartPolicy) -> Self {
        self.restart_policy = policy;
        self
    }

    /// Sets the shutdown policy.
    pub fn shutdown_policy(mut self, policy: ShutdownPolicy) -> Self {
        self.shutdown_policy = policy;
        self
    }

    /// Sets the maximum number of restarts within the window.
    pub fn max_restarts(mut self, max: u32) -> Self {
        self.max_restarts = max;
        self
    }

    /// Sets the restart window duration.
    pub fn restart_window(mut self, window: Duration) -> Self {
        self.restart_window = window;
        self
    }

    /// Sets the backoff configuration.
    pub fn backoff(mut self, backoff: BackoffConfig) -> Self {
        self.backoff = backoff;
        self
    }

    /// Builds the SupervisorConfig, validating all fields.
    ///
    /// # Errors
    ///
    /// Returns `SupervisorConfigError` if validation fails.
    pub fn build(self) -> Result<SupervisorConfig, SupervisorConfigError> {
        let config = SupervisorConfig {
            strategy: self.strategy,
            restart_policy: self.restart_policy,
            shutdown_policy: self.shutdown_policy,
            max_restarts: self.max_restarts,
            restart_window: self.restart_window,
            backoff: self.backoff,
        };
        config.validate()?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Default and Creation Tests (6)
    // =========================================================================

    #[test]
    fn test_default_config() {
        let config = SupervisorConfig::default();

        assert!(matches!(
            config.strategy(),
            WasmSupervisionStrategy::OneForOne
        ));
        assert!(matches!(config.restart_policy(), RestartPolicy::Transient));
        assert_eq!(
            config.shutdown_policy(),
            ShutdownPolicy::Graceful(Duration::from_secs(5))
        );
        assert_eq!(config.max_restarts(), 3);
        assert_eq!(config.restart_window(), Duration::from_secs(60));
        assert_eq!(config.backoff().base_delay(), Duration::from_millis(100));
        assert_eq!(config.backoff().max_delay(), Duration::from_secs(30));
    }

    #[test]
    fn test_new_config() {
        let result = SupervisorConfig::new(5, Duration::from_secs(120));
        assert!(result.is_ok());
        if let Ok(config) = result {
            assert_eq!(config.max_restarts(), 5);
            assert_eq!(config.restart_window(), Duration::from_secs(120));
            // Other fields should be defaults
            assert!(matches!(
                config.strategy(),
                WasmSupervisionStrategy::OneForOne
            ));
            assert!(matches!(config.restart_policy(), RestartPolicy::Transient));
        }
    }

    #[test]
    fn test_new_rejects_zero_max_restarts() {
        let result = SupervisorConfig::new(0, Duration::from_secs(60));
        assert!(result.is_err());
        if let Err(err) = result {
            assert!(matches!(err, SupervisorConfigError::InvalidMaxRestarts(0)));
        }
    }

    #[test]
    fn test_new_rejects_zero_restart_window() {
        let result = SupervisorConfig::new(3, Duration::ZERO);
        assert!(result.is_err());
        if let Err(err) = result {
            assert!(matches!(
                err,
                SupervisorConfigError::InvalidRestartWindow(_)
            ));
        }
    }

    #[test]
    fn test_default_strategy() {
        let strategy = WasmSupervisionStrategy::default();
        assert!(matches!(strategy, WasmSupervisionStrategy::OneForOne));
    }

    #[test]
    fn test_default_backoff_config() {
        let backoff = BackoffConfig::default();
        assert_eq!(backoff.base_delay(), Duration::from_millis(100));
        assert_eq!(backoff.max_delay(), Duration::from_secs(30));
    }

    // =========================================================================
    // Builder Tests (5)
    // =========================================================================

    #[test]
    fn test_builder_with_defaults() {
        let result = SupervisorConfig::builder().build();
        assert!(result.is_ok());
        if let Ok(config) = result {
            assert_eq!(config, SupervisorConfig::default());
        }
    }

    #[test]
    fn test_builder_with_all_fields() {
        let result = SupervisorConfig::builder()
            .strategy(WasmSupervisionStrategy::OneForAll)
            .restart_policy(RestartPolicy::Permanent)
            .shutdown_policy(ShutdownPolicy::Graceful(Duration::from_secs(10)))
            .max_restarts(5)
            .restart_window(Duration::from_secs(120))
            .backoff(BackoffConfig::new(
                Duration::from_millis(200),
                Duration::from_secs(60),
            ))
            .build();

        assert!(result.is_ok());
        if let Ok(config) = result {
            assert!(matches!(
                config.strategy(),
                WasmSupervisionStrategy::OneForAll
            ));
            assert!(matches!(config.restart_policy(), RestartPolicy::Permanent));
            assert_eq!(
                config.shutdown_policy(),
                ShutdownPolicy::Graceful(Duration::from_secs(10))
            );
            assert_eq!(config.max_restarts(), 5);
            assert_eq!(config.restart_window(), Duration::from_secs(120));
            assert_eq!(config.backoff().base_delay(), Duration::from_millis(200));
            assert_eq!(config.backoff().max_delay(), Duration::from_secs(60));
        }
    }

    #[test]
    fn test_builder_strategy() {
        let result = SupervisorConfig::builder()
            .strategy(WasmSupervisionStrategy::RestForOne)
            .build();

        assert!(result.is_ok());
        if let Ok(config) = result {
            assert!(matches!(
                config.strategy(),
                WasmSupervisionStrategy::RestForOne
            ));
        }
    }

    #[test]
    fn test_builder_restart_policy() {
        let result = SupervisorConfig::builder()
            .restart_policy(RestartPolicy::Permanent)
            .build();

        assert!(result.is_ok());
        if let Ok(config) = result {
            assert!(matches!(config.restart_policy(), RestartPolicy::Permanent));
        }
    }

    #[test]
    fn test_builder_chaining() {
        let result = SupervisorConfig::builder()
            .strategy(WasmSupervisionStrategy::OneForAll)
            .max_restarts(10)
            .restart_window(Duration::from_secs(300))
            .build();

        assert!(result.is_ok());
        if let Ok(config) = result {
            assert!(matches!(
                config.strategy(),
                WasmSupervisionStrategy::OneForAll
            ));
            assert_eq!(config.max_restarts(), 10);
            assert_eq!(config.restart_window(), Duration::from_secs(300));
        }
    }

    // =========================================================================
    // Validation Tests (5)
    // =========================================================================

    #[test]
    fn test_validate_valid_config() {
        let config = SupervisorConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_zero_max_restarts() {
        let result = SupervisorConfig::builder().max_restarts(0).build();

        assert!(result.is_err());
        if let Err(err) = result {
            assert!(matches!(err, SupervisorConfigError::InvalidMaxRestarts(0)));
        }
    }

    #[test]
    fn test_validate_zero_restart_window() {
        let result = SupervisorConfig::builder()
            .restart_window(Duration::ZERO)
            .build();

        assert!(result.is_err());
        if let Err(err) = result {
            assert!(matches!(
                err,
                SupervisorConfigError::InvalidRestartWindow(_)
            ));
        }
    }

    #[test]
    fn test_validate_invalid_backoff_range() {
        let result = SupervisorConfig::builder()
            .backoff(BackoffConfig::new(
                Duration::from_secs(60),
                Duration::from_millis(100),
            ))
            .build();

        assert!(result.is_err());
        if let Err(err) = result {
            assert!(matches!(
                err,
                SupervisorConfigError::InvalidBackoffRange(_, _)
            ));
        }
    }

    #[test]
    fn test_validate_zero_graceful_timeout() {
        let result = SupervisorConfig::builder()
            .shutdown_policy(ShutdownPolicy::Graceful(Duration::ZERO))
            .build();

        assert!(result.is_err());
        if let Err(err) = result {
            assert!(matches!(
                err,
                SupervisorConfigError::InvalidShutdownTimeout(_)
            ));
        }
    }

    // =========================================================================
    // Builder Validation Tests (2)
    // =========================================================================

    #[test]
    fn test_builder_rejects_invalid_config() {
        let result = SupervisorConfig::builder().max_restarts(0).build();
        assert!(result.is_err());
    }

    #[test]
    fn test_builder_accepts_valid_config() {
        let result = SupervisorConfig::builder()
            .max_restarts(5)
            .restart_window(Duration::from_secs(120))
            .build();
        assert!(result.is_ok());
    }

    // =========================================================================
    // Conversion Tests (2)
    // =========================================================================

    #[test]
    fn test_to_restart_backoff() {
        let config = SupervisorConfig::default();
        let mut backoff = config.to_restart_backoff();

        // Verify the backoff is functional
        assert!(!backoff.is_limit_exceeded());
        assert_eq!(backoff.restart_count(), 0);

        // Record restarts up to the limit
        backoff.record_restart();
        backoff.record_restart();
        backoff.record_restart();
        assert!(backoff.is_limit_exceeded());
    }

    #[test]
    fn test_to_restart_backoff_custom() {
        let result = SupervisorConfig::builder()
            .max_restarts(5)
            .restart_window(Duration::from_secs(120))
            .backoff(BackoffConfig::new(
                Duration::from_millis(200),
                Duration::from_secs(60),
            ))
            .build();

        assert!(result.is_ok());
        if let Ok(config) = result {
            let mut backoff = config.to_restart_backoff();

            // With custom base_delay of 200ms, first delay should be 200ms * 2^1 = 400ms
            backoff.record_restart();
            let delay = backoff.calculate_delay();
            assert_eq!(delay, Duration::from_millis(400));
        }
    }

    // =========================================================================
    // Accessor Tests (2)
    // =========================================================================

    #[test]
    fn test_accessors() {
        let result = SupervisorConfig::builder()
            .strategy(WasmSupervisionStrategy::RestForOne)
            .restart_policy(RestartPolicy::Temporary)
            .shutdown_policy(ShutdownPolicy::Immediate)
            .max_restarts(7)
            .restart_window(Duration::from_secs(180))
            .backoff(BackoffConfig::new(
                Duration::from_millis(50),
                Duration::from_secs(10),
            ))
            .build();

        assert!(result.is_ok());
        if let Ok(config) = result {
            assert!(matches!(
                config.strategy(),
                WasmSupervisionStrategy::RestForOne
            ));
            assert!(matches!(config.restart_policy(), RestartPolicy::Temporary));
            assert_eq!(config.shutdown_policy(), ShutdownPolicy::Immediate);
            assert_eq!(config.max_restarts(), 7);
            assert_eq!(config.restart_window(), Duration::from_secs(180));
            assert_eq!(config.backoff().base_delay(), Duration::from_millis(50));
            assert_eq!(config.backoff().max_delay(), Duration::from_secs(10));
        }
    }

    #[test]
    fn test_backoff_config_accessors() {
        let backoff = BackoffConfig::new(Duration::from_millis(250), Duration::from_secs(45));
        assert_eq!(backoff.base_delay(), Duration::from_millis(250));
        assert_eq!(backoff.max_delay(), Duration::from_secs(45));
    }

    // =========================================================================
    // Trait and Equality Tests (3)
    // =========================================================================

    #[test]
    fn test_config_clone_and_eq() {
        let config = SupervisorConfig::default();
        let cloned = config.clone();
        assert_eq!(config, cloned);
    }

    #[test]
    fn test_config_debug() {
        let config = SupervisorConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("SupervisorConfig"));
        assert!(debug_str.contains("strategy"));
        assert!(debug_str.contains("restart_policy"));
        assert!(debug_str.contains("shutdown_policy"));
        assert!(debug_str.contains("max_restarts"));
        assert!(debug_str.contains("restart_window"));
        assert!(debug_str.contains("backoff"));
    }

    #[test]
    fn test_config_partial_eq() {
        let config1 = SupervisorConfig::default();
        let config2 = SupervisorConfig::default();
        assert_eq!(config1, config2);

        let result = SupervisorConfig::new(5, Duration::from_secs(120));
        assert!(result.is_ok());
        if let Ok(config3) = result {
            assert_ne!(config1, config3);
        }
    }

    // =========================================================================
    // Bounds Tests (2)
    // =========================================================================

    #[test]
    fn test_send_sync_bounds() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<SupervisorConfig>();
        assert_send_sync::<WasmSupervisionStrategy>();
        assert_send_sync::<BackoffConfig>();
        assert_send_sync::<SupervisorConfigError>();
    }

    #[test]
    fn test_error_clone() {
        let err = SupervisorConfigError::InvalidMaxRestarts(0);
        let cloned = err.clone();
        assert!(matches!(
            cloned,
            SupervisorConfigError::InvalidMaxRestarts(0)
        ));
    }

    // =========================================================================
    // Error Tests (2)
    // =========================================================================

    #[test]
    fn test_error_display_messages() {
        let err1 = SupervisorConfigError::InvalidMaxRestarts(0);
        let msg1 = format!("{}", err1);
        assert!(msg1.contains("max_restarts"));
        assert!(msg1.contains("0"));

        let err2 = SupervisorConfigError::InvalidRestartWindow(Duration::ZERO);
        let msg2 = format!("{}", err2);
        assert!(msg2.contains("restart_window"));

        let err3 = SupervisorConfigError::InvalidBackoffRange(
            Duration::from_secs(60),
            Duration::from_millis(100),
        );
        let msg3 = format!("{}", err3);
        assert!(msg3.contains("base_delay"));
        assert!(msg3.contains("max_delay"));

        let err4 = SupervisorConfigError::InvalidShutdownTimeout(Duration::ZERO);
        let msg4 = format!("{}", err4);
        assert!(msg4.contains("shutdown timeout"));
    }

    #[test]
    fn test_error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<SupervisorConfigError>();
    }
}
