//! Atomic-based metrics implementation.
//!
//! Provides a lock-free, high-performance metrics recorder using atomic operations.
//! This is the default implementation for mailbox metrics.
//!
//! # Performance
//!
//! - Counter operations: ~10-30ns (lock-free atomic fetch_add)
//! - Timestamp updates: ~50-100ns (RwLock write)
//! - No allocations, no blocking (except timestamp write lock)
//!
//! # Example
//!
//! ```rust
//! use airssys_rt::mailbox::metrics::{MetricsRecorder, AtomicMetrics};
//! use chrono::Utc;
//!
//! let metrics = AtomicMetrics::default();
//! 
//! // Fast, lock-free operations
//! metrics.record_sent();
//! metrics.record_received();
//! 
//! assert_eq!(metrics.sent_count(), 1);
//! assert_eq!(metrics.received_count(), 1);
//! ```

// Layer 1: Standard library imports
use std::sync::atomic::{AtomicU64, Ordering};

// Layer 2: Third-party crate imports
use chrono::{DateTime, Utc}; // §3.2 MANDATORY
use parking_lot::RwLock;

// Layer 3: Internal module imports
use super::MetricsRecorder;

/// Lock-free atomic metrics recorder.
///
/// Uses atomic operations for counters (10-30ns overhead) and RwLock for timestamp.
/// This is the default metrics implementation, optimized for low overhead.
///
/// # Performance Characteristics
///
/// - **Counter operations**: ~10-30ns per operation
///   - `record_sent()`, `record_received()`, `record_dropped()` are lock-free
///   - Uses `Ordering::Relaxed` for maximum performance
///   - No contention between threads
///
/// - **Timestamp updates**: ~50-100ns per operation
///   - `update_last_message()` uses RwLock write lock
///   - Minimal contention (infrequent writes)
///
/// - **Query operations**: ~5-10ns per operation
///   - `sent_count()`, `received_count()`, etc. are lock-free reads
///   - No allocations, pure atomic loads
///
/// # Thread Safety
///
/// All operations are thread-safe:
/// - Atomic counters use lock-free atomic operations
/// - Timestamp uses parking_lot RwLock (optimized, low overhead)
/// - Safe to share across multiple mailbox senders
///
/// # Example
///
/// ```rust
/// use airssys_rt::mailbox::metrics::{MetricsRecorder, AtomicMetrics};
/// use chrono::Utc;
/// use std::sync::Arc;
/// use std::thread;
///
/// let metrics = Arc::new(AtomicMetrics::default());
/// 
/// // Can be safely shared across threads
/// let m1 = Arc::clone(&metrics);
/// let h1 = thread::spawn(move || {
///     for _ in 0..1000 {
///         m1.record_sent();
///     }
/// });
///
/// let m2 = Arc::clone(&metrics);
/// let h2 = thread::spawn(move || {
///     for _ in 0..1000 {
///         m2.record_sent();
///     }
/// });
///
/// h1.join().unwrap();
/// h2.join().unwrap();
///
/// assert_eq!(metrics.sent_count(), 2000);
/// ```
#[derive(Debug, Default)]
pub struct AtomicMetrics {
    messages_sent: AtomicU64,
    messages_received: AtomicU64,
    messages_dropped: AtomicU64,
    last_message_at: RwLock<Option<DateTime<Utc>>>,
}

// Manual Clone implementation: creates a new instance with current values copied
impl Clone for AtomicMetrics {
    fn clone(&self) -> Self {
        Self {
            messages_sent: AtomicU64::new(self.messages_sent.load(Ordering::Relaxed)),
            messages_received: AtomicU64::new(self.messages_received.load(Ordering::Relaxed)),
            messages_dropped: AtomicU64::new(self.messages_dropped.load(Ordering::Relaxed)),
            last_message_at: RwLock::new(*self.last_message_at.read()),
        }
    }
}

impl AtomicMetrics {
    /// Create a new AtomicMetrics instance with zero counters.
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_rt::mailbox::metrics::{AtomicMetrics, MetricsRecorder};
    ///
    /// let metrics = AtomicMetrics::new();
    /// assert_eq!(metrics.sent_count(), 0);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }
}

impl MetricsRecorder for AtomicMetrics {
    fn record_sent(&self) {
        self.messages_sent.fetch_add(1, Ordering::Relaxed);
    }

    fn record_received(&self) {
        self.messages_received.fetch_add(1, Ordering::Relaxed);
    }

    fn record_dropped(&self) {
        self.messages_dropped.fetch_add(1, Ordering::Relaxed);
    }

    fn update_last_message(&self, timestamp: DateTime<Utc>) {
        *self.last_message_at.write() = Some(timestamp);
    }

    fn sent_count(&self) -> u64 {
        self.messages_sent.load(Ordering::Relaxed)
    }

    fn received_count(&self) -> u64 {
        self.messages_received.load(Ordering::Relaxed)
    }

    fn dropped_count(&self) -> u64 {
        self.messages_dropped.load(Ordering::Relaxed)
    }

    fn last_message_at(&self) -> Option<DateTime<Utc>> {
        *self.last_message_at.read()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_atomic_metrics_default() {
        let metrics = AtomicMetrics::default();
        assert_eq!(metrics.sent_count(), 0);
        assert_eq!(metrics.received_count(), 0);
        assert_eq!(metrics.dropped_count(), 0);
        assert!(metrics.last_message_at().is_none());
    }

    #[test]
    fn test_atomic_metrics_new() {
        let metrics = AtomicMetrics::new();
        assert_eq!(metrics.sent_count(), 0);
        assert_eq!(metrics.received_count(), 0);
        assert_eq!(metrics.dropped_count(), 0);
        assert!(metrics.last_message_at().is_none());
    }

    #[test]
    fn test_record_sent() {
        let metrics = AtomicMetrics::new();

        metrics.record_sent();
        metrics.record_sent();

        assert_eq!(metrics.sent_count(), 2);
        assert_eq!(metrics.received_count(), 0);
    }

    #[test]
    fn test_record_received() {
        let metrics = AtomicMetrics::new();

        metrics.record_received();
        metrics.record_received();
        metrics.record_received();

        assert_eq!(metrics.received_count(), 3);
        assert_eq!(metrics.sent_count(), 0);
    }

    #[test]
    fn test_record_dropped() {
        let metrics = AtomicMetrics::new();

        metrics.record_dropped();

        assert_eq!(metrics.dropped_count(), 1);
    }

    #[test]
    fn test_update_last_message() {
        let metrics = AtomicMetrics::new();
        let now = Utc::now();

        metrics.update_last_message(now);

        let last = metrics.last_message_at();
        assert!(last.is_some());
        assert_eq!(last.unwrap(), now);
    }

    #[test]
    fn test_in_flight() {
        let metrics = AtomicMetrics::new();

        // Send 5, receive 2
        for _ in 0..5 {
            metrics.record_sent();
        }
        for _ in 0..2 {
            metrics.record_received();
        }

        assert_eq!(metrics.in_flight(), 3);
    }

    #[test]
    fn test_in_flight_saturating() {
        let metrics = AtomicMetrics::new();

        // Edge case: received > sent (shouldn't happen, but handle gracefully)
        metrics.record_received();

        assert_eq!(metrics.in_flight(), 0); // saturating_sub prevents underflow
    }

    #[test]
    fn test_concurrent_operations() {
        use std::sync::Arc;
        use std::thread;

        let metrics = Arc::new(AtomicMetrics::new());
        let mut handles = vec![];

        // Spawn 10 threads, each recording 100 sent
        for _ in 0..10 {
            let m = Arc::clone(&metrics);
            handles.push(thread::spawn(move || {
                for _ in 0..100 {
                    m.record_sent();
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(metrics.sent_count(), 1000);
    }

    #[test]
    fn test_mixed_operations() {
        let metrics = AtomicMetrics::new();
        let now = Utc::now();

        // Mixed operations
        metrics.record_sent();
        metrics.record_sent();
        metrics.record_sent();
        metrics.record_received();
        metrics.record_dropped();
        metrics.update_last_message(now);

        assert_eq!(metrics.sent_count(), 3);
        assert_eq!(metrics.received_count(), 1);
        assert_eq!(metrics.dropped_count(), 1);
        assert_eq!(metrics.in_flight(), 2);
        assert_eq!(metrics.last_message_at().unwrap(), now);
    }
}
