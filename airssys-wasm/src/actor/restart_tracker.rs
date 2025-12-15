//! Restart history tracking and statistics for supervised components.
//!
//! This module maintains a circular buffer of restart attempts, enabling
//! diagnostics, rate limiting, and recovery tracking across component lifecycles.
//!
//! # Design
//!
//! - **Circular buffer**: Stores up to 100 recent restart records
//! - **Total counter**: Tracks lifetime restart count across all records
//! - **Recovery tracking**: Records successful recovery events
//! - **Rate calculation**: Computes restart frequency within recent time windows
//!
//! # Example
//!
//! ```rust,ignore
//! use airssys_wasm::actor::{RestartTracker, RestartReason};
//! use std::time::Duration;
//!
//! let mut tracker = RestartTracker::new();
//! tracker.record_restart(RestartReason::ComponentFailure, Duration::from_millis(100));
//!
//! // Query history
//! let history = tracker.get_history(5);
//! println!("Recent restarts: {:?}", history);
//!
//! // Get statistics
//! println!("Total restarts: {}", tracker.total_restarts());
//!
//! // Reset on recovery
//! tracker.reset_on_recovery();
//! ```

// Layer 1: Standard library imports
use std::time::{Duration, Instant};

/// Reason for component restart.
///
/// Categorizes why a restart was triggered for diagnostics and analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestartReason {
    /// Component crashed or exited unexpectedly
    ComponentFailure,

    /// Health check reported degraded/unhealthy status
    HealthCheckFailed,

    /// User-initiated restart
    ManualRestart,

    /// Restart triggered by timeout
    Timeout,
}

impl std::fmt::Display for RestartReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RestartReason::ComponentFailure => write!(f, "component_failure"),
            RestartReason::HealthCheckFailed => write!(f, "health_check_failed"),
            RestartReason::ManualRestart => write!(f, "manual_restart"),
            RestartReason::Timeout => write!(f, "timeout"),
        }
    }
}

/// Record of a single restart attempt.
///
/// Captures all relevant information about when and why a restart occurred.
#[derive(Debug, Clone)]
pub struct RestartRecord {
    /// Restart attempt number within the current cycle
    pub attempt: u32,

    /// When this restart occurred
    pub timestamp: Instant,

    /// Why the restart was triggered
    pub reason: RestartReason,

    /// How long we waited before restarting
    pub delay_applied: Duration,
}

impl RestartRecord {
    /// Create a new restart record.
    pub fn new(attempt: u32, reason: RestartReason, delay_applied: Duration) -> Self {
        Self {
            attempt,
            timestamp: Instant::now(),
            reason,
            delay_applied,
        }
    }
}

/// Tracks restart history and statistics for a component.
///
/// Maintains a circular buffer of the last 100 restart records,
/// enabling diagnostics, rate calculation, and recovery tracking.
#[derive(Debug)]
pub struct RestartTracker {
    /// Circular buffer of restart records (max 100)
    records: Vec<RestartRecord>,

    /// Current position in circular buffer (0-99)
    buffer_pos: usize,

    /// Total number of restarts across all records
    total_restarts: u32,

    /// Number of successful recoveries
    successful_recovery_count: u32,

    /// Maximum records to keep in buffer
    max_records: usize,
}

impl RestartTracker {
    /// Create a new empty restart tracker.
    ///
    /// # Returns
    ///
    /// Tracker ready to record restart history.
    pub fn new() -> Self {
        Self {
            records: Vec::with_capacity(100),
            buffer_pos: 0,
            total_restarts: 0,
            successful_recovery_count: 0,
            max_records: 100,
        }
    }

    /// Record a restart attempt.
    ///
    /// Adds to circular buffer and updates total count.
    ///
    /// # Parameters
    ///
    /// - `reason`: Why the restart was triggered
    /// - `delay`: How long we waited before restarting
    pub fn record_restart(&mut self, reason: RestartReason, delay: Duration) {
        let attempt = (self.total_restarts % 1000) + 1; // Keep attempt in reasonable range

        let record = RestartRecord::new(attempt, reason, delay);

        if self.records.len() < self.max_records {
            // Buffer not full yet, append
            self.records.push(record);
        } else {
            // Buffer full, overwrite at circular position
            self.records[self.buffer_pos] = record;
        }

        self.buffer_pos = (self.buffer_pos + 1) % self.max_records;
        self.total_restarts = self.total_restarts.saturating_add(1);
    }

    /// Get the last N restart records in reverse chronological order.
    ///
    /// # Parameters
    ///
    /// - `limit`: Maximum number of records to return
    ///
    /// # Returns
    ///
    /// Recent restart records, newest first.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let history = tracker.get_history(10);
    /// for record in history {
    ///     println!("Restart {} at {:?}: {}", record.attempt, record.timestamp, record.reason);
    /// }
    /// ```
    pub fn get_history(&self, limit: usize) -> Vec<RestartRecord> {
        if self.records.is_empty() {
            return vec![];
        }

        let mut result = vec![];
        let count = std::cmp::min(limit, self.records.len());

        // Start from the position before current (most recent record)
        let mut pos = if self.buffer_pos == 0 {
            self.records.len() - 1
        } else {
            self.buffer_pos - 1
        };

        for _ in 0..count {
            result.push(self.records[pos].clone());
            if pos == 0 {
                pos = self.records.len() - 1;
            } else {
                pos -= 1;
            }
        }

        result
    }

    /// Clear history and reset counters on successful recovery.
    ///
    /// Called when component operates successfully for a sustained period,
    /// indicating recovery from failure cycle.
    pub fn reset_on_recovery(&mut self) {
        self.records.clear();
        self.buffer_pos = 0;
        self.successful_recovery_count = self.successful_recovery_count.saturating_add(1);
    }

    /// Get total number of restarts recorded.
    ///
    /// # Returns
    ///
    /// Lifetime restart count.
    pub fn total_restarts(&self) -> u32 {
        self.total_restarts
    }

    /// Get number of successful recoveries.
    ///
    /// # Returns
    ///
    /// Count of times component recovered successfully.
    pub fn successful_recovery_count(&self) -> u32 {
        self.successful_recovery_count
    }

    /// Calculate restart frequency in recent time window.
    ///
    /// # Parameters
    ///
    /// - `window`: Duration to examine (e.g., last 60 seconds)
    ///
    /// # Returns
    ///
    /// Restarts per second during the window.
    pub fn recent_restart_rate(&self, window: Duration) -> f64 {
        let now = Instant::now();
        let window_start = now - window;

        let count = self
            .records
            .iter()
            .filter(|r| r.timestamp >= window_start && r.timestamp <= now)
            .count();

        count as f64 / window.as_secs_f64().max(0.001) // Avoid division by zero
    }

    /// Get number of records currently in history.
    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    /// Clear all history without incrementing recovery count.
    ///
    /// Used for testing or administrative resets.
    pub fn clear(&mut self) {
        self.records.clear();
        self.buffer_pos = 0;
    }

    /// Get restart reason statistics.
    ///
    /// # Returns
    ///
    /// Tuple of (failure_count, health_check_count, manual_count, timeout_count)
    pub fn reason_statistics(&self) -> (u32, u32, u32, u32) {
        let mut failure_count = 0;
        let mut health_check_count = 0;
        let mut manual_count = 0;
        let mut timeout_count = 0;

        for record in &self.records {
            match record.reason {
                RestartReason::ComponentFailure => failure_count += 1,
                RestartReason::HealthCheckFailed => health_check_count += 1,
                RestartReason::ManualRestart => manual_count += 1,
                RestartReason::Timeout => timeout_count += 1,
            }
        }

        (failure_count, health_check_count, manual_count, timeout_count)
    }
}

impl Default for RestartTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_single_restart() {
        let mut tracker = RestartTracker::new();

        tracker.record_restart(RestartReason::ComponentFailure, Duration::from_millis(100));

        assert_eq!(tracker.total_restarts(), 1);
        assert_eq!(tracker.record_count(), 1);

        let history = tracker.get_history(1);
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].reason, RestartReason::ComponentFailure);
        assert_eq!(history[0].delay_applied, Duration::from_millis(100));
    }

    #[test]
    fn test_circular_buffer_overflow() {
        let mut tracker = RestartTracker::new();

        // Record 150 restarts (exceeds max of 100)
        for i in 0..150 {
            let reason = match i % 4 {
                0 => RestartReason::ComponentFailure,
                1 => RestartReason::HealthCheckFailed,
                2 => RestartReason::ManualRestart,
                _ => RestartReason::Timeout,
            };
            tracker.record_restart(reason, Duration::from_millis(100 + i as u64));
        }

        assert_eq!(tracker.total_restarts(), 150);
        assert_eq!(tracker.record_count(), 100); // Max capacity

        // The first 50 records should have been overwritten
        let history = tracker.get_history(100);
        assert_eq!(history.len(), 100);

        // Most recent record should be attempt 150
        assert_eq!(history[0].attempt, 150);
    }

    #[test]
    fn test_history_retrieval_fifo_order() {
        let mut tracker = RestartTracker::new();

        // Record 5 restarts with distinct delays
        for i in 1..=5 {
            tracker.record_restart(
                RestartReason::ComponentFailure,
                Duration::from_millis(i * 100),
            );
        }

        let history = tracker.get_history(5);

        // Should be newest first (LIFO retrieval)
        assert_eq!(history.len(), 5);
        assert_eq!(history[0].delay_applied, Duration::from_millis(500)); // Most recent
        assert_eq!(history[1].delay_applied, Duration::from_millis(400));
        assert_eq!(history[2].delay_applied, Duration::from_millis(300));
        assert_eq!(history[3].delay_applied, Duration::from_millis(200));
        assert_eq!(history[4].delay_applied, Duration::from_millis(100)); // Oldest
    }

    #[test]
    fn test_reset_on_recovery() {
        let mut tracker = RestartTracker::new();

        // Record some restarts
        for _ in 0..5 {
            tracker.record_restart(RestartReason::ComponentFailure, Duration::from_millis(100));
        }

        assert_eq!(tracker.total_restarts(), 5);
        assert_eq!(tracker.record_count(), 5);

        // Reset on recovery
        tracker.reset_on_recovery();

        assert_eq!(tracker.record_count(), 0); // History cleared
        assert_eq!(tracker.total_restarts(), 5); // Total not reset
        assert_eq!(tracker.successful_recovery_count(), 1);
    }

    #[test]
    fn test_total_restarts_counter() {
        let mut tracker = RestartTracker::new();

        assert_eq!(tracker.total_restarts(), 0);

        for i in 1..=20 {
            tracker.record_restart(RestartReason::ComponentFailure, Duration::from_millis(100));
            assert_eq!(tracker.total_restarts(), i as u32);
        }
    }

    #[test]
    fn test_recent_restart_rate() {
        let mut tracker = RestartTracker::new();

        // Record some restarts
        for _ in 0..3 {
            tracker.record_restart(RestartReason::ComponentFailure, Duration::from_millis(100));
        }

        // Immediate rate should be ~3 restarts per second
        let rate = tracker.recent_restart_rate(Duration::from_secs(1));
        assert!(rate >= 2.0, "Expected rate ~3, got {}", rate); // Allow some variance

        // Wait a bit and check rate in longer window
        let rate_10s = tracker.recent_restart_rate(Duration::from_secs(10));
        assert!(rate_10s > 0.0 && rate_10s <= 1.0); // Should be lower with longer window
    }

    #[test]
    fn test_restart_reason_tracking() {
        let mut tracker = RestartTracker::new();

        tracker.record_restart(RestartReason::ComponentFailure, Duration::from_millis(100));
        tracker.record_restart(RestartReason::ComponentFailure, Duration::from_millis(100));
        tracker.record_restart(RestartReason::HealthCheckFailed, Duration::from_millis(100));
        tracker.record_restart(RestartReason::ManualRestart, Duration::from_millis(100));
        tracker.record_restart(RestartReason::Timeout, Duration::from_millis(100));

        let (failure, health, manual, timeout) = tracker.reason_statistics();
        assert_eq!(failure, 2);
        assert_eq!(health, 1);
        assert_eq!(manual, 1);
        assert_eq!(timeout, 1);
    }

    #[test]
    fn test_default_creation() {
        let tracker = RestartTracker::default();
        assert_eq!(tracker.total_restarts(), 0);
        assert_eq!(tracker.record_count(), 0);
        assert_eq!(tracker.successful_recovery_count(), 0);
    }

    #[test]
    fn test_get_history_partial() {
        let mut tracker = RestartTracker::new();

        // Record 10 restarts
        for i in 1..=10 {
            tracker.record_restart(
                RestartReason::ComponentFailure,
                Duration::from_millis(i * 100),
            );
        }

        // Request only 3 most recent
        let history = tracker.get_history(3);
        assert_eq!(history.len(), 3);
        assert_eq!(history[0].delay_applied, Duration::from_millis(1000)); // Most recent
        assert_eq!(history[1].delay_applied, Duration::from_millis(900));
        assert_eq!(history[2].delay_applied, Duration::from_millis(800));
    }
}
