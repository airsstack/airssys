//! Message routing for inter-component communication.
//!
//! This module provides `ResponseRouter` which routes request-response messages
//! from components back to their requesters using correlation IDs.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │           ResponseRouter              │
//! │  • Routes responses via CorrelationID  │
//! │  • Tracks routing metrics             │
//! │  • Detects orphaned responses        │
//! └─────────────────────────────────────────┘
//!            ↓ uses
//! ┌─────────────────────────────────────────┐
//! │       CorrelationTracker             │
//! │  • Maps CorrelationID → channel    │
//! │  • Delivers responses to requesters │
//! └─────────────────────────────────────────┘
//! ```
//!
//! # Request-Response Pattern
//!
//! When a component sends a request to another component:
//! 1. Request message is sent with a unique correlation ID
//! 2. CorrelationTracker registers the pending request
//! 3. ResponseRouter routes the response back using the correlation ID
//!
//! # Thread Safety
//!
//! ResponseRouter is thread-safe via Arc-wrapped CorrelationTracker with DashMap.
//! All operations are lock-free with O(1) complexity.
//!
//! # References
//!
//! - **KNOWLEDGE-WASM-005**: Messaging Architecture
//! - **KNOWLEDGE-WASM-029**: Request-Response Messaging Pattern

// Layer 1: Standard library imports
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

// Layer 2: Third-party crate imports
use serde::{Deserialize, Serialize};

// Layer 3: Internal crate imports
use crate::actor::message::CorrelationTracker;
use crate::core::messaging::{CorrelationId, RequestError, ResponseMessage};
use crate::core::ComponentId;
use chrono::Utc;

#[allow(dead_code)]
/// Routes responses to pending requests using correlation IDs.
///
/// `ResponseRouter` is responsible for:
/// - Looking up pending requests by correlation ID
/// - Delivering responses to requesters via oneshot channels
/// - Tracking routing metrics (successes, orphans, errors)
///
/// # Usage
///
/// The ResponseRouter is created and owned by MessagingService. It is used
/// internally by the messaging infrastructure to route responses back to
/// components that made requests.
///
/// # Thread Safety
///
/// ResponseRouter is thread-safe via an Arc-wrapped CorrelationTracker. All clones
/// share the same CorrelationTracker instance.
///
/// # Performance
///
/// - Correlation ID lookup: ~150ns (DashMap lookup + oneshot send)
/// - Metric updates: ~10ns (atomic operations)
#[derive(Clone)]
pub struct ResponseRouter {
    /// Correlation tracker for pending request lookup
    correlation_tracker: Arc<CorrelationTracker>,

    /// Metrics for monitoring response routing
    metrics: Arc<ResponseRouterMetrics>,
}

/// Metrics for response routing.
#[derive(Debug, Default)]
struct ResponseRouterMetrics {
    /// Total responses routed successfully
    responses_routed: AtomicU64,

    /// Responses that failed to route (no pending request)
    responses_orphaned: AtomicU64,

    /// Responses that were error results
    error_responses: AtomicU64,
}

impl ResponseRouter {
    /// Create a new ResponseRouter with the given correlation tracker.
    ///
    /// # Arguments
    ///
    /// * `correlation_tracker` - Shared correlation tracker for request-response matching
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use airssys_wasm::messaging::ResponseRouter;
    /// use airssys_wasm::actor::message::CorrelationTracker;
    /// use std::sync::Arc;
    ///
    /// let tracker = Arc::new(CorrelationTracker::new());
    /// let router = ResponseRouter::new(tracker);
    /// ```
    pub fn new(correlation_tracker: Arc<CorrelationTracker>) -> Self {
        Self {
            correlation_tracker,
            metrics: Arc::new(ResponseRouterMetrics::default()),
        }
    }

    /// Route a response to the requesting component.
    ///
    /// Looks up the pending request by correlation ID and delivers the response
    /// via the oneshot channel established during `send-request`. The
    /// CorrelationTracker handles the channel delivery and cleanup.
    ///
    /// # Arguments
    ///
    /// * `correlation_id` - Correlation ID from the original request
    /// * `result` - Response result (Ok for success payload, Err for error)
    /// * `from` - Component ID that produced the response
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Response routed successfully
    /// * `Err(WasmError)` - Routing failed (no pending request, already resolved)
    ///
    /// # Errors
    ///
    /// - `WasmError::Internal` - Correlation ID not found (already resolved or timeout)
    ///
    /// # Performance
    ///
    /// ~150ns (DashMap lookup + oneshot send)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let router = messaging_service.response_router();
    ///
    /// // After handle-message returns, route the response
    /// router.route_response(
    ///     correlation_id,
    ///     Ok(response_payload),
    ///     ComponentId::new("responder"),
    /// ).await?;
    /// ```
    pub async fn route_response(
        &self,
        correlation_id: CorrelationId,
        result: Result<Vec<u8>, RequestError>,
        from: ComponentId,
    ) -> Result<(), crate::core::WasmError> {
        // Track error responses
        if result.is_err() {
            self.metrics.error_responses.fetch_add(1, Ordering::Relaxed);
        }

        // Create ResponseMessage
        let response = ResponseMessage {
            correlation_id,
            from,
            to: ComponentId::new(""), // Will be filled by CorrelationTracker::resolve()
            result,
            timestamp: Utc::now(),
        };

        // Resolve via correlation tracker (delivers to oneshot channel)
        match self.correlation_tracker.resolve(correlation_id, response).await {
            Ok(()) => {
                self.metrics.responses_routed.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }
            Err(e) => {
                self.metrics.responses_orphaned.fetch_add(1, Ordering::Relaxed);
                Err(e)
            }
        }
    }

    /// Check if a correlation ID has a pending request.
    ///
    /// Useful for determining whether a response should be routed or ignored.
    /// Fire-and-forget messages won't have pending requests.
    ///
    /// # Arguments
    ///
    /// * `correlation_id` - Correlation ID to check
    ///
    /// # Returns
    ///
    /// `true` if there's a pending request for this correlation ID
    pub fn has_pending_request(&self, correlation_id: &CorrelationId) -> bool {
        self.correlation_tracker.contains(correlation_id)
    }

    /// Get the number of responses routed successfully.
    pub fn responses_routed_count(&self) -> u64 {
        self.metrics.responses_routed.load(Ordering::Relaxed)
    }

    /// Get the number of orphaned responses (no pending request).
    pub fn responses_orphaned_count(&self) -> u64 {
        self.metrics.responses_orphaned.load(Ordering::Relaxed)
    }

    /// Get the number of error responses.
    pub fn error_responses_count(&self) -> u64 {
        self.metrics.error_responses.load(Ordering::Relaxed)
    }

    /// Get a snapshot of response router metrics.
    pub fn get_stats(&self) -> ResponseRouterStats {
        ResponseRouterStats {
            responses_routed: self.metrics.responses_routed.load(Ordering::Relaxed),
            responses_orphaned: self.metrics.responses_orphaned.load(Ordering::Relaxed),
            error_responses: self.metrics.error_responses.load(Ordering::Relaxed),
        }
    }
}

/// Snapshot of response router statistics.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ResponseRouterStats {
    /// Total responses routed successfully
    pub responses_routed: u64,

    /// Responses that failed to route (no pending request)
    pub responses_orphaned: u64,

    /// Responses that were error results
    pub error_responses: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actor::message::PendingRequest;
    use tokio::sync::oneshot;
    use tokio::time::{Duration, Instant};

    #[test]
    fn test_response_router_new() {
        let tracker = Arc::new(CorrelationTracker::new());
        let router = ResponseRouter::new(tracker);

        // Router should be initialized
        assert_eq!(router.responses_routed_count(), 0);
        assert_eq!(router.responses_orphaned_count(), 0);
        assert_eq!(router.error_responses_count(), 0);
    }

    #[test]
    fn test_response_router_clone() {
        let router = ResponseRouter::new(Arc::new(CorrelationTracker::new()));
        let router_clone = router.clone();

        // Both routers should share the same correlation tracker
        assert_eq!(Arc::strong_count(&router.correlation_tracker), 2); // router and one internal Arc

        // Explicit drop ensures the router's internal Arc is the only remaining
        drop(router_clone);
        assert_eq!(Arc::strong_count(&router.correlation_tracker), 1); // only router's Arc remains
    }

    #[test]
    fn test_response_router_has_pending_request_false() {
        let tracker = Arc::new(CorrelationTracker::new());
        let router = ResponseRouter::new(tracker);

        // No requests registered
        assert!(!router.has_pending_request(&uuid::Uuid::new_v4()));
    }

    #[tokio::test]
    async fn test_response_router_has_pending_request_true() {
        let tracker = Arc::new(CorrelationTracker::new());
        let router = ResponseRouter::new(tracker);
        let (tx, _rx) = oneshot::channel();

        let correlation_id = uuid::Uuid::new_v4();
        let pending = PendingRequest {
            correlation_id,
            response_tx: tx,
            requested_at: Instant::now(),
            timeout: Duration::from_secs(30),
            from: ComponentId::new("requester"),
            to: ComponentId::new("responder"),
        };

        router.correlation_tracker.register_pending(pending).await.unwrap();

        // Should have pending request
        assert!(router.has_pending_request(&correlation_id));
    }

    #[tokio::test]
    async fn test_response_router_route_response_success() {
        let tracker = Arc::new(CorrelationTracker::new());
        let router = ResponseRouter::new(tracker.clone());
        let (tx, rx) = oneshot::channel();

        let correlation_id = uuid::Uuid::new_v4();
        let pending = PendingRequest {
            correlation_id,
            response_tx: tx,
            requested_at: Instant::now(),
            timeout: Duration::from_secs(30),
            from: ComponentId::new("requester"),
            to: ComponentId::new("responder"),
        };

        tracker.register_pending(pending).await.unwrap();

        // Route response
        router.route_response(
            correlation_id,
            Ok(vec![1, 2, 3]),
            ComponentId::new("responder"),
        ).await.unwrap();

        // Verify metrics
        assert_eq!(router.responses_routed_count(), 1);
        assert_eq!(router.responses_orphaned_count(), 0);

        // Verify response was delivered
        let response = rx.await.unwrap();
        assert_eq!(response.correlation_id, correlation_id);
        assert_eq!(response.result, Ok(vec![1, 2, 3]));
    }

    #[tokio::test]
    async fn test_response_router_route_response_error() {
        let tracker = Arc::new(CorrelationTracker::new());
        let router = ResponseRouter::new(tracker.clone());
        let (tx, rx) = oneshot::channel();

        let correlation_id = uuid::Uuid::new_v4();
        let pending = PendingRequest {
            correlation_id,
            response_tx: tx,
            requested_at: Instant::now(),
            timeout: Duration::from_secs(30),
            from: ComponentId::new("requester"),
            to: ComponentId::new("responder"),
        };

        tracker.register_pending(pending).await.unwrap();

        // Route error response
        router.route_response(
            correlation_id,
            Err(RequestError::Timeout),
            ComponentId::new("responder"),
        ).await.unwrap();

        // Verify metrics
        assert_eq!(router.responses_routed_count(), 1);
        assert_eq!(router.error_responses_count(), 1);

        // Verify error response was delivered
        let response = rx.await.unwrap();
        assert_eq!(response.correlation_id, correlation_id);
        assert!(response.result.is_err());
    }

    #[tokio::test]
    async fn test_response_router_orphaned_response() {
        let tracker = Arc::new(CorrelationTracker::new());
        let router = ResponseRouter::new(tracker.clone());

        let correlation_id = uuid::Uuid::new_v4();

        // Try to route response without pending request
        let result = router.route_response(
            correlation_id,
            Ok(vec![1, 2, 3]),
            ComponentId::new("responder"),
        ).await;

        // Should fail
        assert!(result.is_err());

        // Verify orphaned metric
        assert_eq!(router.responses_orphaned_count(), 1);
        assert_eq!(router.responses_routed_count(), 0);
    }

    #[test]
    fn test_response_router_get_stats() {
        let tracker = Arc::new(CorrelationTracker::new());
        let router = ResponseRouter::new(tracker);

        let stats = router.get_stats();
        assert_eq!(stats.responses_routed, 0);
        assert_eq!(stats.responses_orphaned, 0);
        assert_eq!(stats.error_responses, 0);
    }
}
