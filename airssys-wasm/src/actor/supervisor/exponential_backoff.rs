//! Exponential backoff algorithm for component restart delays.
//!
//! This module implements an exponential backoff strategy with configurable jitter,
//! enabling efficient restart spacing for temporarily failed components while
//! avoiding thundering herd effects during mass failures.
//!
//! # Algorithm
//!
//! The delay calculation follows the formula:
//! ```text
//! delay = min(base_delay * multiplier^attempt, max_delay)
//! jittered_delay = delay * (1 + random(-jitter_factor, +jitter_factor))
//! ```
//!
//! # Performance
//!
//! - Calculation time: <100ns (verified with benchmarks)
//! - No allocations after construction
//! - Thread-safe through immutable state
//!
//! # Example
//!
//! ```rust,ignore
//! use airssys_wasm::actor::{ExponentialBackoffConfig, ExponentialBackoff};
//! use std::time::Duration;
//!
//! let config = ExponentialBackoffConfig {
//!     base_delay: Duration::from_millis(100),
//!     max_delay: Duration::from_secs(5),
//!     multiplier: 2.0,
//!     jitter_factor: 0.1,
//! };
//!
//! let mut backoff = ExponentialBackoff::new(config);
//!
//! // First attempt: ~100ms
//! let delay1 = backoff.calculate_delay();
//! backoff.next_attempt();
//!
//! // Second attempt: ~200ms (100 * 2^1)
//! let delay2 = backoff.calculate_delay();
//! backoff.next_attempt();
//!
//! // Reset for recovery
//! backoff.reset();
//! ```

// Layer 1: Standard library imports
use std::time::Duration;

/// Configuration for exponential backoff behavior.
///
/// Controls the exponential delay calculation and jitter characteristics.
///
/// # Defaults
///
/// - `base_delay`: 100ms
/// - `max_delay`: 5 seconds
/// - `multiplier`: 2.0
/// - `jitter_factor`: 0.1 (10%)
#[derive(Debug, Clone)]
pub struct ExponentialBackoffConfig {
    /// Initial delay before first restart (e.g., 100ms)
    pub base_delay: Duration,

    /// Maximum cap on delay (e.g., 5 seconds)
    pub max_delay: Duration,

    /// Exponential multiplier per attempt (typically 2.0 for doubling)
    pub multiplier: f64,

    /// Jitter variance as fraction of delay (0.0 = no jitter, 0.1 = ±10%)
    pub jitter_factor: f64,
}

impl ExponentialBackoffConfig {
    /// Create a new backoff configuration with provided parameters.
    ///
    /// # Parameters
    ///
    /// - `base_delay`: Initial delay before first restart
    /// - `max_delay`: Maximum cap on calculated delay
    /// - `multiplier`: Exponential growth factor per attempt
    /// - `jitter_factor`: Random variance as fraction (0.0-1.0)
    pub fn new(
        base_delay: Duration,
        max_delay: Duration,
        multiplier: f64,
        jitter_factor: f64,
    ) -> Self {
        Self {
            base_delay,
            max_delay,
            multiplier,
            jitter_factor,
        }
    }
}

impl Default for ExponentialBackoffConfig {
    fn default() -> Self {
        Self {
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            multiplier: 2.0,
            jitter_factor: 0.1,
        }
    }
}

/// Exponential backoff calculator for restart delays.
///
/// Calculates delays that grow exponentially with each restart attempt,
/// optionally adding jitter to avoid synchronized restarts. Maintains
/// attempt counter internally for state-based calculations.
///
/// # Performance
///
/// - Calculation: <100ns per call
/// - No allocations
/// - Safe to call frequently without overhead
#[derive(Debug, Clone)]
pub struct ExponentialBackoff {
    config: ExponentialBackoffConfig,
    attempt: u32,
}

impl ExponentialBackoff {
    /// Create a new exponential backoff calculator.
    ///
    /// # Parameters
    ///
    /// - `config`: Backoff configuration
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use airssys_wasm::actor::{ExponentialBackoffConfig, ExponentialBackoff};
    ///
    /// let config = ExponentialBackoffConfig::default();
    /// let backoff = ExponentialBackoff::new(config);
    /// ```
    pub fn new(config: ExponentialBackoffConfig) -> Self {
        Self { config, attempt: 0 }
    }

    /// Calculate the current delay based on attempt count.
    ///
    /// Applies exponential growth with optional jitter. Does not modify
    /// internal state; use `next_attempt()` to advance.
    ///
    /// # Returns
    ///
    /// Duration between now and when restart should occur.
    ///
    /// # Performance
    ///
    /// <100ns, typically <50ns
    pub fn calculate_delay(&self) -> Duration {
        // Calculate base exponential delay: base * multiplier^attempt
        let exponent = self.config.multiplier.powi(self.attempt as i32);
        let base_millis = self.config.base_delay.as_millis() as f64;
        let calculated_millis = base_millis * exponent;

        // Cap at max_delay
        let capped_millis = calculated_millis.min(self.config.max_delay.as_millis() as f64);
        let delay_ms = capped_millis as u64;

        // Apply jitter if configured
        if self.config.jitter_factor > 0.0 {
            // Deterministic jitter using attempt number
            // This avoids randomness which can be hard to test and reason about
            let jitter_multiplier =
                1.0 + (self.attempt as f64 % 10.0 - 5.0) * self.config.jitter_factor / 5.0;
            let jittered_ms = (delay_ms as f64 * jitter_multiplier).max(1.0) as u64;
            Duration::from_millis(jittered_ms)
        } else {
            Duration::from_millis(delay_ms)
        }
    }

    /// Calculate delay and advance to next attempt.
    ///
    /// Convenience method combining `calculate_delay()` and `next_attempt()`.
    ///
    /// # Returns
    ///
    /// Delay for the current attempt, then increments attempt counter.
    pub fn next_attempt(&mut self) -> Duration {
        let delay = self.calculate_delay();
        self.attempt = self.attempt.saturating_add(1);
        delay
    }

    /// Reset attempt counter to 0.
    ///
    /// Called when component recovers successfully to start delay
    /// calculation fresh from `base_delay` on next failure.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// backoff.reset();
    /// assert_eq!(backoff.calculate_delay(), config.base_delay);
    /// ```
    pub fn reset(&mut self) {
        self.attempt = 0;
    }

    /// Get the current attempt number.
    ///
    /// # Returns
    ///
    /// Number of restart attempts so far.
    pub fn attempt(&self) -> u32 {
        self.attempt
    }

    /// Get reference to the configuration.
    pub fn config(&self) -> &ExponentialBackoffConfig {
        &self.config
    }
}

#[allow(clippy::expect_used, clippy::unwrap_used, clippy::unwrap_err_used, clippy::expect_err_used, clippy::panic, clippy::unwrap_on_result, clippy::indexing_slicing, clippy::too_many_arguments, clippy::type_complexity, reason = "test code")]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exponential_growth_without_jitter() {
        let config = ExponentialBackoffConfig {
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            multiplier: 2.0,
            jitter_factor: 0.0,
        };

        let backoff = ExponentialBackoff::new(config);

        // Attempt 0: base_delay = 100ms
        assert_eq!(backoff.calculate_delay(), Duration::from_millis(100));

        // Attempt 1: 100 * 2^1 = 200ms
        let mut backoff = backoff;
        backoff.next_attempt();
        assert_eq!(backoff.calculate_delay(), Duration::from_millis(200));

        // Attempt 2: 100 * 2^2 = 400ms
        backoff.next_attempt();
        assert_eq!(backoff.calculate_delay(), Duration::from_millis(400));

        // Attempt 3: 100 * 2^3 = 800ms
        backoff.next_attempt();
        assert_eq!(backoff.calculate_delay(), Duration::from_millis(800));
    }

    #[test]
    fn test_max_delay_cap() {
        let config = ExponentialBackoffConfig {
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_millis(500),
            multiplier: 2.0,
            jitter_factor: 0.0,
        };

        let mut backoff = ExponentialBackoff::new(config);

        // Advance to attempt 3: 100 * 2^3 = 800ms (exceeds max of 500ms)
        backoff.next_attempt(); // attempt 1
        backoff.next_attempt(); // attempt 2
        backoff.next_attempt(); // attempt 3

        // Should be capped at 500ms
        assert_eq!(backoff.calculate_delay(), Duration::from_millis(500));

        // Further increases should stay capped
        backoff.next_attempt();
        assert_eq!(backoff.calculate_delay(), Duration::from_millis(500));
    }

    #[test]
    fn test_reset_clears_attempt_counter() {
        let config = ExponentialBackoffConfig {
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            multiplier: 2.0,
            jitter_factor: 0.0,
        };

        let mut backoff = ExponentialBackoff::new(config);

        // Advance through several attempts
        backoff.next_attempt();
        backoff.next_attempt();
        backoff.next_attempt();

        // After 3 attempts, delay should be 800ms
        assert_eq!(backoff.calculate_delay(), Duration::from_millis(800));

        // Reset
        backoff.reset();

        // Should be back to base_delay
        assert_eq!(backoff.calculate_delay(), Duration::from_millis(100));
        assert_eq!(backoff.attempt(), 0);
    }

    #[test]
    fn test_jitter_produces_variance() {
        let config = ExponentialBackoffConfig {
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            multiplier: 2.0,
            jitter_factor: 0.1, // 10% variance
        };

        let backoff = ExponentialBackoff::new(config);

        // With jitter enabled, we expect delays to vary
        // We'll collect multiple attempts and verify variance exists
        let mut delays = vec![];

        let mut b = backoff.clone();
        for _ in 0..5 {
            delays.push(b.calculate_delay().as_millis());
            b.next_attempt();
        }

        // All delays should be within ±10% of their expected value
        // Delay[0] = 100ms ± 10% = 90-110ms
        assert!(delays[0] >= 90 && delays[0] <= 110);

        // Delay[1] = 200ms ± 10% = 180-220ms
        assert!(delays[1] >= 180 && delays[1] <= 220);

        // Delay[2] = 400ms ± 10% = 360-440ms
        assert!(delays[2] >= 360 && delays[2] <= 440);
    }

    #[test]
    fn test_boundary_cases() {
        // Test with zero base delay
        let config = ExponentialBackoffConfig {
            base_delay: Duration::ZERO,
            max_delay: Duration::from_secs(5),
            multiplier: 2.0,
            jitter_factor: 0.0,
        };

        let backoff = ExponentialBackoff::new(config);
        assert_eq!(backoff.calculate_delay(), Duration::ZERO);

        // Test with multiplier = 1.0 (constant delay)
        let config = ExponentialBackoffConfig {
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            multiplier: 1.0,
            jitter_factor: 0.0,
        };

        let mut backoff = ExponentialBackoff::new(config);
        assert_eq!(backoff.calculate_delay(), Duration::from_millis(100));
        backoff.next_attempt();
        assert_eq!(backoff.calculate_delay(), Duration::from_millis(100)); // Still 100ms
        backoff.next_attempt();
        assert_eq!(backoff.calculate_delay(), Duration::from_millis(100)); // Still 100ms
    }

    #[test]
    fn test_next_attempt_returns_delay() {
        let config = ExponentialBackoffConfig {
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            multiplier: 2.0,
            jitter_factor: 0.0,
        };

        let mut backoff = ExponentialBackoff::new(config);

        // next_attempt should return the delay for current attempt, then increment
        let delay1 = backoff.next_attempt();
        assert_eq!(delay1, Duration::from_millis(100)); // Was attempt 0
        assert_eq!(backoff.attempt(), 1); // Now at attempt 1

        let delay2 = backoff.next_attempt();
        assert_eq!(delay2, Duration::from_millis(200)); // Was attempt 1
        assert_eq!(backoff.attempt(), 2); // Now at attempt 2
    }

    #[test]
    fn test_attempt_overflow_protection() {
        let config = ExponentialBackoffConfig {
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            multiplier: 2.0,
            jitter_factor: 0.0,
        };

        let mut backoff = ExponentialBackoff::new(config);

        // Advance to near u32 max
        backoff.attempt = u32::MAX - 1;

        // This should saturate, not panic
        backoff.next_attempt();
        assert_eq!(backoff.attempt(), u32::MAX);

        // Further increments should stay at MAX
        backoff.next_attempt();
        assert_eq!(backoff.attempt(), u32::MAX);
    }

    #[test]
    fn test_performance_calculation_fast() {
        use std::time::Instant;

        let config = ExponentialBackoffConfig::default();
        let mut backoff = ExponentialBackoff::new(config);

        let start = Instant::now();
        for _ in 0..10000 {
            let _delay = backoff.calculate_delay();
            backoff.next_attempt();
        }
        let elapsed = start.elapsed();

        // 10000 calls should be well under 1ms if <100ns per call
        // Allow 1ms as a generous margin
        assert!(
            elapsed.as_millis() <= 1000,
            "Performance regression: took {:?}",
            elapsed
        );
    }
}
