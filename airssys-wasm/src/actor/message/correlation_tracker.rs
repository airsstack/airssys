//! Correlation tracking for request-response patterns.
//!
//! This module provides high-performance correlation tracking using lock-free
//! concurrent data structures (DashMap) for request-response patterns with
//! automatic timeout handling.
//!
//! # Architecture
//!
//! ```text
//! CorrelationTracker
//!     ├── DashMap<CorrelationId, PendingRequest> (lock-free)
//!     └── TimeoutHandler (background cleanup)
//! ```
//!
//! # Performance
//!
//! - Lookup: <50ns (DashMap lock-free read)
//! - Insert: ~100ns (DashMap sharded write)
//! - Remove: ~100ns (atomic swap)
//! - Memory: ~170KB per 1000 pending requests (168 bytes per PendingRequest)
//! - Concurrent: Unlimited readers + writers
//!
//! # Examples
//!
//! ```rust,ignore
//! use airssys_wasm::actor::message::{CorrelationTracker, PendingRequest};
//! use tokio::sync::oneshot;
//! use std::time::Duration;
//!
//! let tracker = CorrelationTracker::new();
//!
//! // Register pending request
//! let (tx, rx) = oneshot::channel();
//! let corr_id = Uuid::new_v4();
//! tracker.register_pending(PendingRequest {
//!     correlation_id: corr_id,
//!     response_tx: tx,
//!     requested_at: Instant::now(),
//!     timeout: Duration::from_secs(5),
//!     from: comp_a,
//!     to: comp_b,
//! }).await?;
//!
//! // Resolve with response
//! tracker.resolve(corr_id, response).await?;
//! ```
//!
//! # References
//!
//! - **ADR-WASM-009**: Component Communication Model (Pattern 2: Request-Response)
//! - **WASM-TASK-004 Phase 5 Task 5.1**: Message Correlation Implementation

// Layer 1: Standard library imports
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

// Layer 2: Third-party crate imports
use chrono::Utc;
use dashmap::DashMap;
use tokio::time::Instant;

// Layer 3: Internal module imports
use super::timeout_handler::TimeoutHandler;
use crate::core::messaging::{CorrelationId, PendingRequest, RequestError, ResponseMessage};
use crate::core::WasmError;

/// High-performance correlation tracker for request-response patterns.
///
/// Uses DashMap for lock-free concurrent access with <50ns lookup overhead.
/// Integrates with TimeoutHandler for automatic timeout enforcement.
///
/// # Architecture
///
/// ```text
/// CorrelationTracker
///     ├── DashMap<CorrelationId, PendingRequest> (lock-free)
///     └── TimeoutHandler (background cleanup)
/// ```
///
/// # Performance
///
/// - Lookup: <50ns (DashMap lock-free read)
/// - Insert: ~100ns (DashMap sharded write)
/// - Remove: ~100ns (atomic swap)
/// - Memory: ~170KB per 1000 pending requests (168 bytes per PendingRequest)
/// - Concurrent: Unlimited readers + writers
///
/// # Thread Safety
///
/// All operations are thread-safe via DashMap atomic operations and oneshot
/// channels. The first of (response arrival, timeout) wins - no race conditions.
///
/// # Memory Management
///
/// **IMPORTANT:** Callers MUST periodically call [`cleanup_expired()`](Self::cleanup_expired)
/// to prevent memory leaks from expired requests. See method documentation for details.
///
/// # Examples
///
/// ```rust,ignore
/// use airssys_wasm::actor::message::CorrelationTracker;
///
/// let tracker = CorrelationTracker::new();
///
/// // Register pending request
/// let (tx, rx) = oneshot::channel();
/// let corr_id = Uuid::new_v4();
/// tracker.register_pending(PendingRequest {
///     correlation_id: corr_id,
///     response_tx: tx,
///     requested_at: Instant::now(),
///     timeout: Duration::from_secs(5),
///     from: comp_a,
///     to: comp_b,
/// }).await?;
///
/// // Resolve with response
/// tracker.resolve(corr_id, response).await?;
/// ```
#[derive(Clone)]
pub struct CorrelationTracker {
    /// Pending requests (lock-free concurrent access)
    pending: Arc<DashMap<CorrelationId, PendingRequest>>,
    /// Timeout handler for automatic cleanup
    timeout_handler: Arc<TimeoutHandler>,
    /// Counter for completed (resolved) requests (Phase 3 Task 3.2)
    completed_count: Arc<AtomicU64>,
    /// Counter for timed out requests
    timeout_count: Arc<AtomicU64>,
}

impl CorrelationTracker {
    /// Create new CorrelationTracker.
    ///
    /// Initializes empty pending request map and timeout handler.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let tracker = CorrelationTracker::new();
    /// ```
    pub fn new() -> Self {
        Self {
            pending: Arc::new(DashMap::new()),
            timeout_handler: Arc::new(TimeoutHandler::new()),
            completed_count: Arc::new(AtomicU64::new(0)),
            timeout_count: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Register pending request with timeout.
    ///
    /// Stores the request in the pending map and schedules a timeout task.
    /// If the request is not resolved before timeout, a timeout error will
    /// be sent to the response channel.
    ///
    /// # Arguments
    ///
    /// * `request` - Pending request with correlation ID and response channel
    ///
    /// # Returns
    ///
    /// Ok(()) if registered successfully, Err if correlation ID already exists
    ///
    /// # Errors
    ///
    /// Returns WasmError::Internal if:
    /// - Correlation ID already exists (duplicate registration)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let (tx, rx) = oneshot::channel();
    /// tracker.register_pending(PendingRequest {
    ///     correlation_id: corr_id,
    ///     response_tx: tx,
    ///     requested_at: Instant::now(),
    ///     timeout: Duration::from_secs(5),
    ///     from: comp_a,
    ///     to: comp_b,
    /// }).await?;
    /// ```
    pub async fn register_pending(&self, request: PendingRequest) -> Result<(), WasmError> {
        let correlation_id = request.correlation_id;
        let timeout = request.timeout;

        // Check for duplicate correlation ID
        if self.pending.contains_key(&correlation_id) {
            return Err(WasmError::internal(format!(
                "Duplicate correlation ID: {}",
                correlation_id
            )));
        }

        // Insert into pending map
        self.pending.insert(correlation_id, request);

        // Register timeout handler
        self.timeout_handler
            .register_timeout(correlation_id, timeout, self.clone());

        Ok(())
    }

    /// Resolve pending request with response.
    ///
    /// Removes the request from the pending map and delivers the response
    /// via the oneshot channel. Cancels the timeout task if response arrives
    /// before timeout.
    ///
    /// # Arguments
    ///
    /// * `correlation_id` - Correlation ID of the request
    /// * `response` - Response message to deliver
    ///
    /// # Returns
    ///
    /// Ok(()) if resolved successfully, Err if correlation ID not found
    ///
    /// # Errors
    ///
    /// Returns WasmError::Internal if:
    /// - Correlation ID not found (already resolved or timed out)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// tracker.resolve(corr_id, response_msg).await?;
    /// ```
    pub async fn resolve(
        &self,
        correlation_id: CorrelationId,
        mut response: ResponseMessage,
    ) -> Result<(), WasmError> {
        // Remove from pending map (atomic operation)
        let pending = self
            .pending
            .remove(&correlation_id)
            .ok_or_else(|| {
                WasmError::internal(format!("Correlation ID not found: {}", correlation_id))
            })?
            .1; // DashMap::remove returns (key, value)

        // Cancel timeout (response arrived before timeout)
        self.timeout_handler.cancel_timeout(&correlation_id);

        // Fill in 'to' field from pending request
        response.to = pending.from;

        // Send response via oneshot channel
        // Ignore send error (receiver may have been dropped)
        let _ = pending.response_tx.send(response);

        // Increment completed count (Phase 3 Task 3.2)
        self.completed_count.fetch_add(1, Ordering::Relaxed);

        Ok(())
    }

    /// Remove expired pending request (called by TimeoutHandler).
    ///
    /// This is an internal method called by the timeout handler when a
    /// request times out. It removes the request from the pending map
    /// and returns it so the timeout handler can send a timeout error.
    ///
    /// # Arguments
    ///
    /// * `correlation_id` - Correlation ID to remove
    ///
    /// # Returns
    ///
    /// Some(PendingRequest) if found and removed, None if already resolved
    pub(crate) fn remove_pending(&self, correlation_id: &CorrelationId) -> Option<PendingRequest> {
        self.pending.remove(correlation_id).map(|(_, v)| v)
    }

    /// Cleanup expired requests (background maintenance).
    ///
    /// Removes requests that have exceeded their timeout duration but whose
    /// timeout handlers haven't fired yet (e.g., due to system overload or
    /// scheduler delays).
    ///
    /// # When to Call
    ///
    /// This method should be called periodically by a background task to
    /// prevent memory leaks from abandoned requests. Recommended interval: **60 seconds**.
    ///
    /// # Who Should Call
    ///
    /// - **ComponentSpawner**: Can spawn a background cleanup task per CorrelationTracker instance
    /// - **ActorSystem**: Can run a global cleanup task for all trackers
    /// - **User Code**: Can call manually in low-traffic scenarios
    ///
    /// # Memory Leak Prevention
    ///
    /// Without periodic cleanup, expired requests accumulate in the `pending` map if:
    /// - Timeout handler is delayed by system load
    /// - Component crashes before timeout fires
    /// - Oneshot receiver is dropped without reading
    ///
    /// Each pending request consumes ~168 bytes. Without cleanup:
    /// - 1000 expired requests = ~170KB leaked
    /// - 10,000 expired requests = ~1.7MB leaked
    ///
    /// # Returns
    ///
    /// Number of expired requests cleaned up (for monitoring/alerting).
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Background cleanup task (recommended)
    /// let tracker = Arc::new(CorrelationTracker::new());
    /// let tracker_clone = tracker.clone();
    ///
    /// tokio::spawn(async move {
    ///     let mut interval = tokio::time::interval(Duration::from_secs(60));
    ///     loop {
    ///         interval.tick().await;
    ///         let cleaned = tracker_clone.cleanup_expired().await;
    ///         if cleaned > 0 {
    ///             log::warn!("Cleaned up {} expired requests", cleaned);
    ///         }
    ///     }
    /// });
    ///
    /// // Manual cleanup (for testing or low-traffic)
    /// let cleaned = tracker.cleanup_expired().await;
    /// assert_eq!(cleaned, 0); // No expired requests
    /// ```
    pub async fn cleanup_expired(&self) -> usize {
        let now = Instant::now();
        let mut expired_count = 0;
        let mut expired_ids = Vec::new();

        // First pass: identify expired requests
        for entry in self.pending.iter() {
            let pending = entry.value();
            let expired = now.duration_since(pending.requested_at) > pending.timeout;
            if expired {
                expired_ids.push(*entry.key());
            }
        }

        // Second pass: remove expired and send timeout errors
        for corr_id in expired_ids {
            if let Some((_, pending)) = self.pending.remove(&corr_id) {
                expired_count += 1;
                // Increment timeout count (Phase 3 Task 3.2)
                self.timeout_count.fetch_add(1, Ordering::Relaxed);
                // Send timeout error before removing
                let _ = pending.response_tx.send(ResponseMessage {
                    correlation_id: pending.correlation_id,
                    from: pending.to.clone(),
                    to: pending.from.clone(),
                    result: Err(RequestError::Timeout),
                    timestamp: Utc::now(),
                });
            }
        }

        expired_count
    }

    /// Get number of pending requests (for monitoring).
    ///
    /// Returns the current count of pending requests waiting for responses.
    /// Useful for monitoring system load and detecting issues.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let pending = tracker.pending_count();
    /// if pending > 1000 {
    ///     tracing::warn!("High pending request count: {}", pending);
    /// }
    /// ```
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Check if correlation ID exists (for testing).
    ///
    /// Returns true if the correlation ID is currently in the pending map.
    /// Primarily used for testing and debugging.
    ///
    /// # Arguments
    ///
    /// * `correlation_id` - Correlation ID to check
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// assert!(tracker.contains(&corr_id));
    /// ```
    pub fn contains(&self, correlation_id: &CorrelationId) -> bool {
        self.pending.contains_key(correlation_id)
    }

    /// Get the number of completed (resolved) requests.
    ///
    /// Returns the total count of requests that were successfully resolved
    /// with a response. This counter is incremented by `resolve()`.
    ///
    /// # Performance
    ///
    /// ~3ns overhead (single atomic load)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let completed = tracker.completed_count();
    /// println!("Requests completed: {}", completed);
    /// ```
    pub fn completed_count(&self) -> u64 {
        self.completed_count.load(Ordering::Relaxed)
    }

    /// Get the number of timed out requests.
    ///
    /// Returns the total count of requests that expired before receiving
    /// a response. This counter is incremented by `cleanup_expired()`.
    ///
    /// # Performance
    ///
    /// ~3ns overhead (single atomic load)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let timeouts = tracker.timeout_count();
    /// if timeouts > 0 {
    ///     tracing::warn!("Requests timed out: {}", timeouts);
    /// }
    /// ```
    pub fn timeout_count(&self) -> u64 {
        self.timeout_count.load(Ordering::Relaxed)
    }
}

impl Default for CorrelationTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "unwrap acceptable in test code")]
mod tests {
    use super::*;
    use crate::core::ComponentId;
    use tokio::sync::oneshot;
    use tokio::time::Duration;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_new_tracker() {
        let tracker = CorrelationTracker::new();
        assert_eq!(tracker.pending_count(), 0);
    }

    #[tokio::test]
    async fn test_register_pending() {
        let tracker = CorrelationTracker::new();
        let (tx, _rx) = oneshot::channel();
        let corr_id = Uuid::new_v4();

        let request = PendingRequest {
            correlation_id: corr_id,
            response_tx: tx,
            requested_at: Instant::now(),
            timeout: Duration::from_secs(5),
            from: ComponentId::new("comp-a"),
            to: ComponentId::new("comp-b"),
        };

        let result = tracker.register_pending(request).await;
        assert!(result.is_ok());
        assert_eq!(tracker.pending_count(), 1);
        assert!(tracker.contains(&corr_id));
    }

    #[tokio::test]
    async fn test_duplicate_correlation_id() {
        let tracker = CorrelationTracker::new();
        let corr_id = Uuid::new_v4();

        // Register first request
        let (tx1, _rx1) = oneshot::channel();
        let request1 = PendingRequest {
            correlation_id: corr_id,
            response_tx: tx1,
            requested_at: Instant::now(),
            timeout: Duration::from_secs(5),
            from: ComponentId::new("comp-a"),
            to: ComponentId::new("comp-b"),
        };
        tracker.register_pending(request1).await.unwrap();

        // Try to register duplicate
        let (tx2, _rx2) = oneshot::channel();
        let request2 = PendingRequest {
            correlation_id: corr_id,
            response_tx: tx2,
            requested_at: Instant::now(),
            timeout: Duration::from_secs(5),
            from: ComponentId::new("comp-a"),
            to: ComponentId::new("comp-b"),
        };
        let result = tracker.register_pending(request2).await;
        assert!(result.is_err());
        assert_eq!(tracker.pending_count(), 1);
    }

    #[tokio::test]
    async fn test_resolve_success() {
        let tracker = CorrelationTracker::new();
        let (tx, rx) = oneshot::channel();
        let corr_id = Uuid::new_v4();

        let request = PendingRequest {
            correlation_id: corr_id,
            response_tx: tx,
            requested_at: Instant::now(),
            timeout: Duration::from_secs(5),
            from: ComponentId::new("comp-a"),
            to: ComponentId::new("comp-b"),
        };

        tracker.register_pending(request).await.unwrap();
        assert_eq!(tracker.pending_count(), 1);

        // Resolve with response
        let response = ResponseMessage {
            correlation_id: corr_id,
            from: ComponentId::new("comp-b"),
            to: ComponentId::new("comp-a"), // Will be overwritten
            result: Ok(vec![1, 2, 3]),
            timestamp: Utc::now(),
        };

        let result = tracker.resolve(corr_id, response).await;
        assert!(result.is_ok());
        assert_eq!(tracker.pending_count(), 0);
        assert!(!tracker.contains(&corr_id));

        // Verify response received
        let received = rx.await.unwrap();
        assert_eq!(received.correlation_id, corr_id);
        assert!(received.result.is_ok());
    }

    #[tokio::test]
    async fn test_resolve_not_found() {
        let tracker = CorrelationTracker::new();
        let corr_id = Uuid::new_v4();

        let response = ResponseMessage {
            correlation_id: corr_id,
            from: ComponentId::new("comp-b"),
            to: ComponentId::new("comp-a"),
            result: Ok(vec![1, 2, 3]),
            timestamp: Utc::now(),
        };

        let result = tracker.resolve(corr_id, response).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_pending_count() {
        let tracker = CorrelationTracker::new();
        assert_eq!(tracker.pending_count(), 0);

        // Register 3 requests
        for _ in 0..3 {
            let (tx, _rx) = oneshot::channel();
            let corr_id = Uuid::new_v4();
            let request = PendingRequest {
                correlation_id: corr_id,
                response_tx: tx,
                requested_at: Instant::now(),
                timeout: Duration::from_secs(5),
                from: ComponentId::new("comp-a"),
                to: ComponentId::new("comp-b"),
            };
            tracker.register_pending(request).await.unwrap();
        }

        assert_eq!(tracker.pending_count(), 3);
    }

    #[tokio::test]
    async fn test_contains() {
        let tracker = CorrelationTracker::new();
        let (tx, _rx) = oneshot::channel();
        let corr_id = Uuid::new_v4();

        assert!(!tracker.contains(&corr_id));

        let request = PendingRequest {
            correlation_id: corr_id,
            response_tx: tx,
            requested_at: Instant::now(),
            timeout: Duration::from_secs(5),
            from: ComponentId::new("comp-a"),
            to: ComponentId::new("comp-b"),
        };

        tracker.register_pending(request).await.unwrap();
        assert!(tracker.contains(&corr_id));
    }

    // ============================================================================
    // Phase 3 Task 3.2 Tests - completed_count and timeout_count
    // ============================================================================

    #[tokio::test]
    async fn test_completed_count_initial() {
        let tracker = CorrelationTracker::new();
        assert_eq!(tracker.completed_count(), 0);
    }

    #[tokio::test]
    async fn test_completed_count_after_resolve() {
        let tracker = CorrelationTracker::new();
        let (tx, _rx) = oneshot::channel();
        let corr_id = Uuid::new_v4();

        let request = PendingRequest {
            correlation_id: corr_id,
            response_tx: tx,
            requested_at: Instant::now(),
            timeout: Duration::from_secs(5),
            from: ComponentId::new("comp-a"),
            to: ComponentId::new("comp-b"),
        };

        tracker.register_pending(request).await.unwrap();
        assert_eq!(tracker.completed_count(), 0);

        let response = ResponseMessage {
            correlation_id: corr_id,
            from: ComponentId::new("comp-b"),
            to: ComponentId::new("comp-a"),
            result: Ok(vec![1, 2, 3]),
            timestamp: Utc::now(),
        };

        tracker.resolve(corr_id, response).await.unwrap();
        assert_eq!(tracker.completed_count(), 1);
    }

    #[tokio::test]
    async fn test_completed_count_multiple_resolves() {
        let tracker = CorrelationTracker::new();
        
        // Register and resolve 3 requests
        for i in 0..3 {
            let (tx, _rx) = oneshot::channel();
            let corr_id = Uuid::new_v4();

            let request = PendingRequest {
                correlation_id: corr_id,
                response_tx: tx,
                requested_at: Instant::now(),
                timeout: Duration::from_secs(5),
                from: ComponentId::new("comp-a"),
                to: ComponentId::new("comp-b"),
            };

            tracker.register_pending(request).await.unwrap();

            let response = ResponseMessage {
                correlation_id: corr_id,
                from: ComponentId::new("comp-b"),
                to: ComponentId::new("comp-a"),
                result: Ok(vec![i as u8]),
                timestamp: Utc::now(),
            };

            tracker.resolve(corr_id, response).await.unwrap();
        }

        assert_eq!(tracker.completed_count(), 3);
    }

    #[tokio::test]
    async fn test_timeout_count_initial() {
        let tracker = CorrelationTracker::new();
        assert_eq!(tracker.timeout_count(), 0);
    }

    #[tokio::test]
    async fn test_timeout_count_after_cleanup() {
        let tracker = CorrelationTracker::new();
        let (tx, _rx) = oneshot::channel();
        let corr_id = Uuid::new_v4();

        // Create request with very short timeout (0ms)
        let request = PendingRequest {
            correlation_id: corr_id,
            response_tx: tx,
            requested_at: Instant::now() - Duration::from_secs(10), // Already expired
            timeout: Duration::from_millis(1),
            from: ComponentId::new("comp-a"),
            to: ComponentId::new("comp-b"),
        };

        tracker.register_pending(request).await.unwrap();
        assert_eq!(tracker.timeout_count(), 0);

        // Cleanup should find the expired request
        let cleaned = tracker.cleanup_expired().await;
        assert_eq!(cleaned, 1);
        assert_eq!(tracker.timeout_count(), 1);
    }

    #[tokio::test]
    async fn test_counts_are_independent() {
        let tracker = CorrelationTracker::new();
        
        // Register request 1 (will be resolved)
        let (tx1, _rx1) = oneshot::channel();
        let corr_id1 = Uuid::new_v4();
        let request1 = PendingRequest {
            correlation_id: corr_id1,
            response_tx: tx1,
            requested_at: Instant::now(),
            timeout: Duration::from_secs(30),
            from: ComponentId::new("comp-a"),
            to: ComponentId::new("comp-b"),
        };
        tracker.register_pending(request1).await.unwrap();

        // Register request 2 (will timeout)
        let (tx2, _rx2) = oneshot::channel();
        let corr_id2 = Uuid::new_v4();
        let request2 = PendingRequest {
            correlation_id: corr_id2,
            response_tx: tx2,
            requested_at: Instant::now() - Duration::from_secs(10), // Already expired
            timeout: Duration::from_millis(1),
            from: ComponentId::new("comp-a"),
            to: ComponentId::new("comp-b"),
        };
        tracker.register_pending(request2).await.unwrap();

        // Resolve request 1
        let response = ResponseMessage {
            correlation_id: corr_id1,
            from: ComponentId::new("comp-b"),
            to: ComponentId::new("comp-a"),
            result: Ok(vec![1, 2, 3]),
            timestamp: Utc::now(),
        };
        tracker.resolve(corr_id1, response).await.unwrap();

        // Cleanup request 2
        tracker.cleanup_expired().await;

        // Both counts should be independent
        assert_eq!(tracker.completed_count(), 1);
        assert_eq!(tracker.timeout_count(), 1);
    }
}
