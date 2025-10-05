//! Metrics recorder trait for mailbox operations.
//!
//! Defines the interface for recording and querying mailbox metrics.
//! Implementations can use different backends (atomics, channels, remote exporters).
//!
//! # Design Principles
//!
//! - **Generic constraints**: Used as `R: MetricsRecorder`, not `dyn` (§6.2)
//! - **Send + Sync**: Can be shared across threads safely
//! - **Method-based**: All operations through methods, no public fields
//! - **Future-proof**: Easy to add new implementations
//!
//! # Example
//!
//! ```rust
//! use airssys_rt::mailbox::metrics::{MetricsRecorder, AtomicMetrics};
//! use chrono::Utc;
//!
//! let metrics = AtomicMetrics::default();
//!
//! // Record operations
//! metrics.record_sent();
//! metrics.record_received();
//! metrics.update_last_message(Utc::now());
//!
//! // Query metrics
//! assert_eq!(metrics.sent_count(), 1);
//! assert_eq!(metrics.received_count(), 1);
//! assert_eq!(metrics.in_flight(), 0);
//! ```

// Layer 2: Third-party crate imports
use chrono::{DateTime, Utc}; // §3.2 MANDATORY

/// Trait for recording mailbox metrics.
///
/// This trait abstracts the metrics recording mechanism, allowing different
/// implementations (atomic counters, async channels, remote exporters, etc.)
/// without changing mailbox code.
///
/// # Design Principles
///
/// - **Generic constraints**: Used as `R: MetricsRecorder`, not `dyn` (§6.2)
/// - **Send + Sync**: Can be shared across threads safely
/// - **Method-based**: All operations through methods, no public fields
/// - **Future-proof**: Easy to add new implementations
///
/// # Implementations
///
/// - `AtomicMetrics`: Lock-free atomic counters (default, 10-30ns overhead)
/// - Future: `AsyncMetrics`, `NoOpMetrics`, `PrometheusMetrics`
///
/// # Example
///
/// ```rust
/// use airssys_rt::mailbox::metrics::{MetricsRecorder, AtomicMetrics};
/// use chrono::Utc;
///
/// let metrics = AtomicMetrics::default();
///
/// // Record operations
/// metrics.record_sent();
/// metrics.record_received();
/// metrics.update_last_message(Utc::now());
///
/// // Query metrics
/// assert_eq!(metrics.sent_count(), 1);
/// assert_eq!(metrics.received_count(), 1);
/// assert_eq!(metrics.in_flight(), 0);
/// ```
pub trait MetricsRecorder: Send + Sync {
    /// Record a message send operation.
    ///
    /// Called when a message is successfully sent to the mailbox.
    fn record_sent(&self);

    /// Record a message receive operation.
    ///
    /// Called when a message is successfully received from the mailbox.
    fn record_received(&self);

    /// Record a dropped message (backpressure or TTL expiration).
    ///
    /// Called when a message is dropped due to full mailbox with Drop strategy
    /// or when a message TTL has expired.
    fn record_dropped(&self);

    /// Update the timestamp of the last processed message.
    ///
    /// Called after successfully receiving a message to track activity.
    ///
    /// # Arguments
    ///
    /// * `timestamp` - The timestamp of the last processed message (§3.2)
    fn update_last_message(&self, timestamp: DateTime<Utc>);

    /// Get total number of messages sent.
    ///
    /// Returns the cumulative count of all messages sent to this mailbox.
    fn sent_count(&self) -> u64;

    /// Get total number of messages received.
    ///
    /// Returns the cumulative count of all messages received from this mailbox.
    fn received_count(&self) -> u64;

    /// Get total number of messages dropped.
    ///
    /// Returns the cumulative count of all messages dropped (backpressure or TTL).
    fn dropped_count(&self) -> u64;

    /// Get timestamp of last processed message.
    ///
    /// Returns `None` if no messages have been processed yet.
    fn last_message_at(&self) -> Option<DateTime<Utc>>;

    /// Get number of messages currently in-flight (sent but not received).
    ///
    /// Calculated as: `sent_count - received_count`
    /// Uses saturating subtraction to handle edge cases gracefully.
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_rt::mailbox::metrics::{MetricsRecorder, AtomicMetrics};
    ///
    /// let metrics = AtomicMetrics::default();
    ///
    /// // Send 5, receive 2
    /// for _ in 0..5 {
    ///     metrics.record_sent();
    /// }
    /// for _ in 0..2 {
    ///     metrics.record_received();
    /// }
    ///
    /// assert_eq!(metrics.in_flight(), 3);
    /// ```
    fn in_flight(&self) -> u64 {
        self.sent_count().saturating_sub(self.received_count())
    }
}
