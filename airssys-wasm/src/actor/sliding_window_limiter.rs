//! Sliding window restart rate limiter.
//!
//! Enforces maximum restart limits within specified time windows to prevent
//! restart storms and detect permanently failed components.
//!
//! # Algorithm
//!
//! Uses a sliding window approach:
//! 1. Maintain queue of recent restart timestamps
//! 2. On check, remove restarts outside the time window
//! 3. If count < max, allow restart; otherwise deny
//! 4. Performance: O(n) where n = max_restarts (typically 5-10)
//!
//! # Example
//!
//! ```rust,ignore
//! use airssys_wasm::actor::{SlidingWindowConfig, SlidingWindowLimiter, WindowLimitResult};
//! use std::time::Duration;
//!
//! let config = SlidingWindowConfig {
//!     max_restarts: 5,
//!     window_duration: Duration::from_secs(60),
//! };
//!
//! let mut limiter = SlidingWindowLimiter::new(config);
//!
//! // Check if restart is allowed
//! match limiter.check_can_restart() {
//!     WindowLimitResult::AllowRestart => {
//!         println!("Restart allowed");
//!         limiter.record_restart();
//!     }
//!     WindowLimitResult::DenyRestart { reason, next_available } => {
//!         println!("Restart denied: {}", reason);
//!         if let Some(next) = next_available {
//!             println!("Next available restart at: {:?}", next);
//!         }
//!     }
//! }
//! ```

// Layer 1: Standard library imports
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Configuration for sliding window restart limiting.
///
/// Controls how many restarts are allowed within a given time window.
#[derive(Debug, Clone)]
pub struct SlidingWindowConfig {
    /// Maximum number of restarts allowed within the window
    pub max_restarts: u32,

    /// Time window during which restarts are counted (e.g., 60 seconds)
    pub window_duration: Duration,
}

impl SlidingWindowConfig {
    /// Create a new sliding window configuration.
    ///
    /// # Parameters
    ///
    /// - `max_restarts`: Maximum restarts allowed (e.g., 5)
    /// - `window_duration`: Time window (e.g., 60 seconds)
    pub fn new(max_restarts: u32, window_duration: Duration) -> Self {
        Self {
            max_restarts,
            window_duration,
        }
    }
}

impl Default for SlidingWindowConfig {
    fn default() -> Self {
        Self {
            max_restarts: 5,
            window_duration: Duration::from_secs(60),
        }
    }
}

/// Result of checking if a restart is allowed.
#[derive(Debug, Clone, Copy)]
pub enum WindowLimitResult {
    /// Restart is allowed within the window limit
    AllowRestart,

    /// Restart is denied; exceeded maximum in window
    DenyRestart {
        /// Explanation of why restart was denied
        reason: &'static str,

        /// When the next restart might be available (if calculable)
        next_available: Option<Instant>,
    },
}

/// Sliding window limiter for restart rate control.
///
/// Tracks recent restart timestamps and enforces a maximum number of restarts
/// within a rolling time window. Enables early detection of unrecoverable
/// component failures.
#[derive(Debug)]
pub struct SlidingWindowLimiter {
    config: SlidingWindowConfig,

    /// Queue of restart timestamps (oldest first)
    restart_times: VecDeque<Instant>,

    /// Whether component is permanently failed
    permanently_failed: bool,

    /// How many times we've hit the limit
    limit_hit_count: u32,
}

impl SlidingWindowLimiter {
    /// Create a new sliding window limiter.
    ///
    /// # Parameters
    ///
    /// - `config`: Configuration for window and limits
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use airssys_wasm::actor::{SlidingWindowConfig, SlidingWindowLimiter};
    /// use std::time::Duration;
    ///
    /// let config = SlidingWindowConfig::new(5, Duration::from_secs(60));
    /// let limiter = SlidingWindowLimiter::new(config);
    /// ```
    pub fn new(config: SlidingWindowConfig) -> Self {
        let capacity = config.max_restarts as usize + 1;
        Self {
            config,
            restart_times: VecDeque::with_capacity(capacity),
            permanently_failed: false,
            limit_hit_count: 0,
        }
    }

    /// Check if a restart is currently allowed.
    ///
    /// Removes timestamps outside the window, then checks if count is below limit.
    ///
    /// # Returns
    ///
    /// `AllowRestart` if within limits, `DenyRestart` with reason if not.
    ///
    /// # Performance
    ///
    /// O(n) where n = max_restarts, typically very fast (5-10 items)
    pub fn check_can_restart(&mut self) -> WindowLimitResult {
        // Remove restarts outside the window
        let now = Instant::now();
        let window_start = now - self.config.window_duration;

        while let Some(&oldest) = self.restart_times.front() {
            if oldest < window_start {
                self.restart_times.pop_front();
            } else {
                break;
            }
        }

        let count = self.restart_times.len() as u32;

        if count < self.config.max_restarts {
            WindowLimitResult::AllowRestart
        } else {
            self.limit_hit_count = self.limit_hit_count.saturating_add(1);

            // Calculate when the oldest restart exits the window
            let next_available = self.restart_times.front().map(|oldest| {
                *oldest + self.config.window_duration
            });

            WindowLimitResult::DenyRestart {
                reason: "Maximum restart rate exceeded in time window",
                next_available,
            }
        }
    }

    /// Record that a restart occurred.
    ///
    /// Should be called after `check_can_restart()` returns `AllowRestart`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// match limiter.check_can_restart() {
    ///     WindowLimitResult::AllowRestart => {
    ///         // Perform restart...
    ///         limiter.record_restart();
    ///     }
    ///     _ => {}
    /// }
    /// ```
    pub fn record_restart(&mut self) {
        self.restart_times.push_back(Instant::now());
    }

    /// Determine if component should be considered permanently failed.
    ///
    /// Heuristic: More than 5 failed restart attempts with no recovery suggests
    /// the component cannot recover and restart attempts should cease.
    ///
    /// # Returns
    ///
    /// `true` if component appears permanently failed, `false` otherwise.
    pub fn is_permanently_failed(&self) -> bool {
        self.permanently_failed || self.limit_hit_count >= 5
    }

    /// Mark component as permanently failed (or recovery).
    pub fn set_permanently_failed(&mut self, failed: bool) {
        self.permanently_failed = failed;
    }

    /// Clear restart history and reset state.
    ///
    /// Called on component recovery or administrative reset.
    pub fn reset(&mut self) {
        self.restart_times.clear();
        self.limit_hit_count = 0;
        self.permanently_failed = false;
    }

    /// Get current number of restarts in the window.
    pub fn restart_count_in_window(&self) -> u32 {
        self.restart_times.len() as u32
    }

    /// Get how many times the restart limit has been hit.
    pub fn limit_hit_count(&self) -> u32 {
        self.limit_hit_count
    }
}

impl Default for SlidingWindowLimiter {
    fn default() -> Self {
        Self::new(SlidingWindowConfig::default())
    }
}

#[cfg(test)]
#[allow(clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_allow_restart_within_limit() {
        let config = SlidingWindowConfig::new(5, Duration::from_secs(60));
        let mut limiter = SlidingWindowLimiter::new(config);

        // Should allow first 5 restarts
        for _ in 0..5 {
            match limiter.check_can_restart() {
                WindowLimitResult::AllowRestart => {
                    limiter.record_restart();
                }
                _ => panic!("Should allow restart within limit"),
            }
        }

        assert_eq!(limiter.restart_count_in_window(), 5);
    }

    #[test]
    fn test_deny_restart_at_limit() {
        let config = SlidingWindowConfig::new(3, Duration::from_secs(60));
        let mut limiter = SlidingWindowLimiter::new(config);

        // Fill up to limit
        for _ in 0..3 {
            limiter.record_restart();
        }

        // Next check should deny
        match limiter.check_can_restart() {
            WindowLimitResult::DenyRestart { reason, .. } => {
                assert!(reason.contains("Maximum"));
                assert_eq!(limiter.limit_hit_count(), 1);
            }
            _ => panic!("Should deny at limit"),
        }
    }

    #[test]
    fn test_sliding_window_cleanup() {
        let config = SlidingWindowConfig::new(2, Duration::from_millis(100));
        let mut limiter = SlidingWindowLimiter::new(config);

        // Record restarts at time 0
        limiter.record_restart();
        limiter.record_restart();

        assert_eq!(limiter.restart_count_in_window(), 2);

        // Wait for window to pass
        std::thread::sleep(Duration::from_millis(150));

        // Check removes old restarts
        match limiter.check_can_restart() {
            WindowLimitResult::AllowRestart => {
                // Old restarts should have been removed
                assert_eq!(limiter.restart_count_in_window(), 0);
            }
            _ => panic!("Should allow after window passes"),
        }
    }

    #[test]
    fn test_multiple_windows_in_sequence() {
        let config = SlidingWindowConfig::new(2, Duration::from_millis(50));
        let mut limiter = SlidingWindowLimiter::new(config);

        // First window: 2 restarts
        limiter.record_restart();
        limiter.record_restart();

        // Should be at limit
        assert!(matches!(
            limiter.check_can_restart(),
            WindowLimitResult::DenyRestart { .. }
        ));

        // Wait for window to clear
        std::thread::sleep(Duration::from_millis(60));

        // Should allow again (old restarts cleared)
        match limiter.check_can_restart() {
            WindowLimitResult::AllowRestart => {
                limiter.record_restart();
            }
            _ => panic!("Should allow after window clears"),
        }

        // Should be able to do one more
        match limiter.check_can_restart() {
            WindowLimitResult::AllowRestart => {
                limiter.record_restart();
            }
            _ => panic!("Should allow second restart in new window"),
        }
    }

    #[test]
    fn test_permanent_failure_detection_by_limit_hits() {
        let config = SlidingWindowConfig::new(1, Duration::from_millis(50));
        let mut limiter = SlidingWindowLimiter::new(config);

        // Hit the limit 5 times
        for _ in 0..5 {
            limiter.record_restart();
            match limiter.check_can_restart() {
                WindowLimitResult::DenyRestart { .. } => {}
                _ => panic!("Expected to hit limit"),
            }

            // Wait for window to clear to hit limit again
            std::thread::sleep(Duration::from_millis(60));
        }

        // Should now be marked as permanently failed
        assert!(limiter.is_permanently_failed());
    }

    #[test]
    fn test_permanent_failure_detection_manual() {
        let config = SlidingWindowConfig::new(5, Duration::from_secs(60));
        let mut limiter = SlidingWindowLimiter::new(config);

        assert!(!limiter.is_permanently_failed());

        limiter.set_permanently_failed(true);
        assert!(limiter.is_permanently_failed());

        limiter.set_permanently_failed(false);
        assert!(!limiter.is_permanently_failed());
    }

    #[test]
    fn test_reset_behavior() {
        let config = SlidingWindowConfig::new(3, Duration::from_secs(60));
        let mut limiter = SlidingWindowLimiter::new(config);

        // Build up state
        for _ in 0..3 {
            limiter.record_restart();
        }

        // Hit the limit a few times
        for _ in 0..3 {
            let _ = limiter.check_can_restart();
        }

        assert_eq!(limiter.limit_hit_count(), 3);
        assert_eq!(limiter.restart_count_in_window(), 3);

        // Reset
        limiter.reset();

        assert_eq!(limiter.limit_hit_count(), 0);
        assert_eq!(limiter.restart_count_in_window(), 0);
        assert!(!limiter.is_permanently_failed());
    }

    #[test]
    fn test_default_creation() {
        let limiter = SlidingWindowLimiter::default();
        assert_eq!(limiter.restart_count_in_window(), 0);
        assert!(!limiter.is_permanently_failed());
        assert_eq!(limiter.config.max_restarts, 5);
    }

    #[test]
    fn test_next_available_calculation() {
        let config = SlidingWindowConfig::new(1, Duration::from_secs(10));
        let mut limiter = SlidingWindowLimiter::new(config);

        let first_restart = Instant::now();
        limiter.record_restart();

        // Try to restart immediately (should fail)
        match limiter.check_can_restart() {
            WindowLimitResult::DenyRestart { next_available, .. } => {
                assert!(next_available.is_some());
                #[allow(clippy::expect_used)]
                let next = next_available.expect("next_available should be Some");
                // Next available should be roughly 10 seconds from first restart
                let elapsed = next.saturating_duration_since(first_restart);
                assert!(elapsed.as_secs() >= 9 && elapsed.as_secs() <= 11);
            }
            _ => panic!("Expected deny with next_available"),
        }
    }

    #[test]
    fn test_limit_hit_count_increments() {
        let config = SlidingWindowConfig::new(1, Duration::from_millis(50));
        let mut limiter = SlidingWindowLimiter::new(config);

        assert_eq!(limiter.limit_hit_count(), 0);

        limiter.record_restart();
        match limiter.check_can_restart() {
            WindowLimitResult::DenyRestart { .. } => assert_eq!(limiter.limit_hit_count(), 1),
            _ => panic!("Expected deny"),
        }

        std::thread::sleep(Duration::from_millis(60));

        limiter.record_restart();
        match limiter.check_can_restart() {
            WindowLimitResult::DenyRestart { .. } => assert_eq!(limiter.limit_hit_count(), 2),
            _ => panic!("Expected deny"),
        }
    }
}
