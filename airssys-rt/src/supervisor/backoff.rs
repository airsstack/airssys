//! Restart rate limiting and exponential backoff.
//!
//! This module implements restart policies to prevent restart storms and cascading
//! failures. It provides sliding window rate limiting and exponential backoff delays
//! to ensure supervisors don't restart children too aggressively.
//!
//! # Key Features
//!
//! - **Sliding Window**: Track restarts in a time window, expire old entries
//! - **Rate Limiting**: Prevent exceeding max restarts in window
//! - **Exponential Backoff**: Increase delay between restart attempts
//! - **Configurable**: Adjust base delay, max delay, and restart limits
//!
//! # Examples
//!
//! ```rust
//! use airssys_rt::supervisor::RestartBackoff;
//! use std::time::Duration;
//!
//! let mut backoff = RestartBackoff::new(5, Duration::from_secs(60));
//!
//! // Record a restart
//! backoff.record_restart();
//!
//! // Check if limit exceeded
//! if backoff.is_limit_exceeded() {
//!     println!("Too many restarts, escalating to parent");
//! }
//!
//! // Get backoff delay (exponential)
//! let delay = backoff.calculate_delay();
//! tokio::time::sleep(delay).await;
//! ```

// Layer 1: Standard library imports
use std::collections::VecDeque;
use std::time::Duration;

// Layer 2: Third-party crate imports
use chrono::{DateTime, Utc};

// Layer 3: Internal module imports
// (none needed for backoff)

/// Restart backoff and rate limiting.
///
/// Tracks restart history in a sliding time window and enforces rate limits.
/// Provides exponential backoff delays to prevent restart storms.
///
/// # Configuration
///
/// - `max_restarts`: Maximum restarts allowed in the window
/// - `restart_window`: Time window for counting restarts (sliding)
///
/// # Sliding Window Behavior
///
/// The restart window slides forward as time passes. Old restarts automatically
/// expire and no longer count toward the limit. This prevents permanent lockout
/// after transient issues.
///
/// # Exponential Backoff Formula
///
/// ```text
/// delay = base_delay * 2^(min(restart_count, 10))
/// delay = min(delay, max_delay)
/// ```
///
/// Default configuration:
/// - Base delay: 100ms
/// - Max delay: 60 seconds
/// - Exponential factor: 2x per restart
/// - Cap: 10 restarts (prevents overflow)
///
/// # Examples
///
/// ```rust
/// use airssys_rt::supervisor::RestartBackoff;
/// use std::time::Duration;
///
/// // Allow 5 restarts per minute
/// let mut backoff = RestartBackoff::new(5, Duration::from_secs(60));
///
/// // Simulate rapid restarts
/// for _ in 0..3 {
///     backoff.record_restart();
///     if backoff.is_limit_exceeded() {
///         break;
///     }
/// }
///
/// // Calculate exponential delay
/// let delay = backoff.calculate_delay();
/// assert!(delay >= Duration::from_millis(100)); // Base delay
/// ```
#[derive(Debug, Clone)]
pub struct RestartBackoff {
    /// Maximum restarts allowed in the window
    max_restarts: u32,

    /// Time window for counting restarts (sliding)
    restart_window: Duration,

    /// History of restart timestamps (newest first)
    restart_history: VecDeque<DateTime<Utc>>,

    /// Base delay for exponential backoff (default: 100ms)
    base_delay: Duration,

    /// Maximum delay for exponential backoff (default: 60s)
    max_delay: Duration,
}

impl RestartBackoff {
    /// Create a new restart backoff tracker.
    ///
    /// # Parameters
    ///
    /// - `max_restarts`: Maximum restarts allowed in the window
    /// - `restart_window`: Time window for counting restarts
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::supervisor::RestartBackoff;
    /// use std::time::Duration;
    ///
    /// // Allow 5 restarts per minute
    /// let backoff = RestartBackoff::new(5, Duration::from_secs(60));
    /// ```
    pub fn new(max_restarts: u32, restart_window: Duration) -> Self {
        Self {
            max_restarts,
            restart_window,
            restart_history: VecDeque::new(),
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(60),
        }
    }

    /// Create a new restart backoff tracker with custom delays.
    ///
    /// # Parameters
    ///
    /// - `max_restarts`: Maximum restarts allowed in the window
    /// - `restart_window`: Time window for counting restarts
    /// - `base_delay`: Initial backoff delay (doubles each restart)
    /// - `max_delay`: Maximum backoff delay (cap)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::supervisor::RestartBackoff;
    /// use std::time::Duration;
    ///
    /// // Custom backoff: start at 1s, max 5 minutes
    /// let backoff = RestartBackoff::with_delays(
    ///     5,
    ///     Duration::from_secs(60),
    ///     Duration::from_secs(1),
    ///     Duration::from_secs(300),
    /// );
    /// ```
    pub fn with_delays(
        max_restarts: u32,
        restart_window: Duration,
        base_delay: Duration,
        max_delay: Duration,
    ) -> Self {
        Self {
            max_restarts,
            restart_window,
            restart_history: VecDeque::new(),
            base_delay,
            max_delay,
        }
    }

    /// Check if restart limit has been exceeded.
    ///
    /// Automatically expires old restarts outside the sliding window.
    /// Returns `true` if the number of recent restarts exceeds `max_restarts`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::supervisor::RestartBackoff;
    /// use std::time::Duration;
    ///
    /// let mut backoff = RestartBackoff::new(3, Duration::from_secs(60));
    ///
    /// // Not exceeded yet
    /// assert!(!backoff.is_limit_exceeded());
    ///
    /// // Add restarts
    /// for _ in 0..3 {
    ///     backoff.record_restart();
    /// }
    ///
    /// // Now at limit
    /// assert!(backoff.is_limit_exceeded());
    /// ```
    pub fn is_limit_exceeded(&mut self) -> bool {
        self.cleanup_expired_restarts();
        self.restart_history.len() >= self.max_restarts as usize
    }

    /// Record a restart at the current time.
    ///
    /// Adds a restart to the history. Old restarts are automatically expired
    /// by `is_limit_exceeded()` and `calculate_delay()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::supervisor::RestartBackoff;
    /// use std::time::Duration;
    ///
    /// let mut backoff = RestartBackoff::new(5, Duration::from_secs(60));
    ///
    /// backoff.record_restart();
    /// assert_eq!(backoff.restart_count(), 1);
    /// ```
    pub fn record_restart(&mut self) {
        self.restart_history.push_front(Utc::now());
    }

    /// Calculate exponential backoff delay based on recent restarts.
    ///
    /// Formula: `delay = base_delay * 2^(min(restart_count, 10))`
    ///
    /// Automatically expires old restarts outside the window before calculating.
    ///
    /// # Returns
    ///
    /// Duration to wait before next restart attempt.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::supervisor::RestartBackoff;
    /// use std::time::Duration;
    ///
    /// let mut backoff = RestartBackoff::new(10, Duration::from_secs(60));
    ///
    /// // First restart: 100ms
    /// backoff.record_restart();
    /// assert_eq!(backoff.calculate_delay(), Duration::from_millis(100));
    ///
    /// // Second restart: 200ms
    /// backoff.record_restart();
    /// assert_eq!(backoff.calculate_delay(), Duration::from_millis(200));
    ///
    /// // Third restart: 400ms
    /// backoff.record_restart();
    /// assert_eq!(backoff.calculate_delay(), Duration::from_millis(400));
    /// ```
    pub fn calculate_delay(&mut self) -> Duration {
        self.cleanup_expired_restarts();

        let restart_count = self.restart_history.len() as u32;

        // Cap at 10 restarts to prevent exponential overflow
        let capped_count = restart_count.min(10);

        // Calculate: base * 2^count
        let multiplier = 2u64.pow(capped_count);
        let delay_ms = self.base_delay.as_millis() as u64 * multiplier;
        let delay = Duration::from_millis(delay_ms);

        // Cap at max_delay
        delay.min(self.max_delay)
    }

    /// Get the current restart count (within window).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::supervisor::RestartBackoff;
    /// use std::time::Duration;
    ///
    /// let mut backoff = RestartBackoff::new(5, Duration::from_secs(60));
    ///
    /// assert_eq!(backoff.restart_count(), 0);
    ///
    /// backoff.record_restart();
    /// assert_eq!(backoff.restart_count(), 1);
    /// ```
    pub fn restart_count(&mut self) -> u32 {
        self.cleanup_expired_restarts();
        self.restart_history.len() as u32
    }

    /// Reset the restart history.
    ///
    /// Clears all tracked restarts. Useful for testing or manual intervention.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::supervisor::RestartBackoff;
    /// use std::time::Duration;
    ///
    /// let mut backoff = RestartBackoff::new(5, Duration::from_secs(60));
    ///
    /// backoff.record_restart();
    /// backoff.record_restart();
    /// assert_eq!(backoff.restart_count(), 2);
    ///
    /// backoff.reset();
    /// assert_eq!(backoff.restart_count(), 0);
    /// ```
    pub fn reset(&mut self) {
        self.restart_history.clear();
    }

    /// Remove restarts that are outside the sliding window.
    ///
    /// Called automatically by public methods. Keeps only restarts within
    /// `restart_window` from now.
    fn cleanup_expired_restarts(&mut self) {
        let now = Utc::now();

        // Convert std::time::Duration to chrono::Duration
        // This conversion can only fail if the duration is too large (> ~292 years)
        // If conversion fails (extremely unlikely), we keep all history for safety
        let Ok(chrono_window) = chrono::Duration::from_std(self.restart_window) else {
            return; // Keep all history if duration is invalid
        };
        let window_start = now - chrono_window;

        // Remove restarts older than window_start (history is newest-first)
        while let Some(&oldest) = self.restart_history.back() {
            if oldest < window_start {
                self.restart_history.pop_back();
            } else {
                break;
            }
        }
    }
}

impl Default for RestartBackoff {
    /// Create default restart backoff: 5 restarts per 60 seconds.
    fn default() -> Self {
        Self::new(5, Duration::from_secs(60))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_new_backoff() {
        let backoff = RestartBackoff::new(5, Duration::from_secs(60));

        assert_eq!(backoff.max_restarts, 5);
        assert_eq!(backoff.restart_window, Duration::from_secs(60));
        assert_eq!(backoff.base_delay, Duration::from_millis(100));
        assert_eq!(backoff.max_delay, Duration::from_secs(60));
    }

    #[test]
    fn test_with_delays() {
        let backoff = RestartBackoff::with_delays(
            3,
            Duration::from_secs(30),
            Duration::from_secs(1),
            Duration::from_secs(300),
        );

        assert_eq!(backoff.base_delay, Duration::from_secs(1));
        assert_eq!(backoff.max_delay, Duration::from_secs(300));
    }

    #[test]
    fn test_record_restart() {
        let mut backoff = RestartBackoff::new(5, Duration::from_secs(60));

        assert_eq!(backoff.restart_count(), 0);

        backoff.record_restart();
        assert_eq!(backoff.restart_count(), 1);

        backoff.record_restart();
        assert_eq!(backoff.restart_count(), 2);
    }

    #[test]
    fn test_is_limit_exceeded() {
        let mut backoff = RestartBackoff::new(3, Duration::from_secs(60));

        assert!(!backoff.is_limit_exceeded());

        backoff.record_restart();
        assert!(!backoff.is_limit_exceeded());

        backoff.record_restart();
        assert!(!backoff.is_limit_exceeded());

        backoff.record_restart();
        assert!(backoff.is_limit_exceeded());

        backoff.record_restart();
        assert!(backoff.is_limit_exceeded());
    }

    #[test]
    fn test_exponential_backoff_calculation() {
        let mut backoff = RestartBackoff::new(10, Duration::from_secs(60));

        // No restarts: base delay
        assert_eq!(backoff.calculate_delay(), Duration::from_millis(100));

        // 1 restart: 100ms * 2^1 = 200ms
        backoff.record_restart();
        assert_eq!(backoff.calculate_delay(), Duration::from_millis(200));

        // 2 restarts: 100ms * 2^2 = 400ms
        backoff.record_restart();
        assert_eq!(backoff.calculate_delay(), Duration::from_millis(400));

        // 3 restarts: 100ms * 2^3 = 800ms
        backoff.record_restart();
        assert_eq!(backoff.calculate_delay(), Duration::from_millis(800));

        // 4 restarts: 100ms * 2^4 = 1600ms
        backoff.record_restart();
        assert_eq!(backoff.calculate_delay(), Duration::from_millis(1600));
    }

    #[test]
    fn test_exponential_backoff_max_delay() {
        let mut backoff = RestartBackoff::with_delays(
            20,
            Duration::from_secs(300),
            Duration::from_millis(100),
            Duration::from_secs(5),
        );

        // Add many restarts to exceed max delay
        for _ in 0..15 {
            backoff.record_restart();
        }

        // Should be capped at max_delay
        let delay = backoff.calculate_delay();
        assert_eq!(delay, Duration::from_secs(5));
    }

    #[test]
    fn test_exponential_backoff_capped_at_10() {
        let mut backoff = RestartBackoff::new(20, Duration::from_secs(300));

        // Add 15 restarts
        for _ in 0..15 {
            backoff.record_restart();
        }

        // Should use 2^10 (capped), not 2^15
        // 100ms * 2^10 = 100ms * 1024 = 102,400ms
        let expected = Duration::from_millis(102_400);
        assert_eq!(backoff.calculate_delay(), expected.min(backoff.max_delay));
    }

    #[test]
    fn test_restart_window_expiration() {
        let mut backoff = RestartBackoff::new(3, Duration::from_millis(100));

        // Add 2 restarts
        backoff.record_restart();
        backoff.record_restart();
        assert_eq!(backoff.restart_count(), 2);

        // Wait for window to expire
        thread::sleep(Duration::from_millis(150));

        // Restarts should be expired
        assert_eq!(backoff.restart_count(), 0);
        assert!(!backoff.is_limit_exceeded());
    }

    #[test]
    fn test_reset() {
        let mut backoff = RestartBackoff::new(5, Duration::from_secs(60));

        backoff.record_restart();
        backoff.record_restart();
        backoff.record_restart();
        assert_eq!(backoff.restart_count(), 3);

        backoff.reset();
        assert_eq!(backoff.restart_count(), 0);
        assert!(!backoff.is_limit_exceeded());
    }

    #[test]
    fn test_sliding_window() {
        let mut backoff = RestartBackoff::new(3, Duration::from_millis(200));

        // Add 2 restarts immediately
        backoff.record_restart();
        backoff.record_restart();
        assert_eq!(backoff.restart_count(), 2);

        // Wait half window
        thread::sleep(Duration::from_millis(100));

        // Add another restart
        backoff.record_restart();
        assert_eq!(backoff.restart_count(), 3);

        // Wait for first 2 to expire
        thread::sleep(Duration::from_millis(150));

        // Only the last restart should remain
        assert_eq!(backoff.restart_count(), 1);
    }

    #[test]
    fn test_default() {
        let backoff = RestartBackoff::default();

        assert_eq!(backoff.max_restarts, 5);
        assert_eq!(backoff.restart_window, Duration::from_secs(60));
    }

    #[test]
    fn test_backoff_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<RestartBackoff>();
    }

    #[test]
    fn test_backoff_clone() {
        let mut backoff1 = RestartBackoff::new(5, Duration::from_secs(60));
        backoff1.record_restart();

        let mut backoff2 = backoff1.clone();
        assert_eq!(backoff1.restart_count(), backoff2.restart_count());
    }
}
