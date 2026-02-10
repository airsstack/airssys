//! Correlation tracking implementation for request-response patterns.
//!
//! Manages pending requests with timeouts and oneshot channel delivery.
//! This module provides the concrete [`CorrelationTrackerImpl`] that
//! implements the [`CorrelationTracker`] trait from `core/messaging/traits`.
//!
//! # Architecture
//!
//! This module is part of `messaging/` (Layer 3B). It depends on:
//! - `core/component/` for [`MessagePayload`]
//! - `core/messaging/` for [`CorrelationTracker`], [`CorrelationId`], [`MessagingError`]
//!
//! # References
//!
//! - ADR-WASM-031: Component & Messaging Module Design
//! - ADR-WASM-009: Component Communication Model (Pattern 2: Request-Response)

// Layer 1: Standard library imports
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

// Layer 2: Third-party crate imports
use tokio::sync::oneshot;

// Layer 3: Internal module imports
use crate::core::component::message::MessagePayload;
use crate::core::messaging::correlation::CorrelationId;
use crate::core::messaging::errors::MessagingError;
use crate::core::messaging::traits::CorrelationTracker;

/// Concrete implementation of [`CorrelationTracker`] for request-response patterns.
///
/// Tracks pending requests using oneshot channels for response delivery.
/// Provides timeout-based expiration and cleanup of stale correlations.
///
/// # Thread Safety
///
/// Uses `std::sync::RwLock` for interior mutability. All public methods
/// are safe to call from multiple threads concurrently.
///
/// # Examples
///
/// ```rust
/// use airssys_wasm::messaging::correlation::CorrelationTrackerImpl;
/// use airssys_wasm::core::messaging::traits::CorrelationTracker;
/// use airssys_wasm::core::messaging::correlation::CorrelationId;
/// use airssys_wasm::core::component::message::MessagePayload;
///
/// let tracker = CorrelationTrackerImpl::new();
/// let id = CorrelationId::new("request-123");
///
/// // Register a pending request
/// tracker.register(&id, 5000).unwrap();
/// assert!(tracker.is_pending(&id));
///
/// // Complete with response
/// let response = MessagePayload::new(vec![1, 2, 3]);
/// tracker.complete(&id, response).unwrap();
/// assert!(!tracker.is_pending(&id));
/// ```
pub struct CorrelationTrackerImpl {
    /// Pending requests tracked by correlation ID string
    pending: RwLock<HashMap<String, PendingRequest>>,
}

/// Internal struct for tracking a pending request.
///
/// Each pending request holds a oneshot sender for delivering the response
/// and a deadline after which the request is considered timed out.
#[derive(Debug)]
struct PendingRequest {
    /// Oneshot sender for delivering the response payload
    sender: oneshot::Sender<MessagePayload>,
    /// Deadline after which this request is expired
    deadline: Instant,
}

impl CorrelationTrackerImpl {
    /// Creates a new empty correlation tracker.
    pub fn new() -> Self {
        Self {
            pending: RwLock::new(HashMap::new()),
        }
    }

    /// Creates a new correlation and returns a receiver for the response.
    ///
    /// This method is used when the caller needs direct access to the
    /// response via a oneshot channel (e.g., for await-based patterns).
    ///
    /// # Arguments
    ///
    /// * `correlation_id` - The correlation identifier
    /// * `timeout_ms` - Timeout in milliseconds
    ///
    /// # Returns
    ///
    /// A `oneshot::Receiver<MessagePayload>` that will receive the response
    /// when [`complete`](CorrelationTracker::complete) is called.
    ///
    /// # Errors
    ///
    /// Returns `MessagingError::DeliveryFailed` if the correlation ID already
    /// exists or if the internal lock is poisoned.
    pub fn create(
        &self,
        correlation_id: &CorrelationId,
        timeout_ms: u64,
    ) -> Result<oneshot::Receiver<MessagePayload>, MessagingError> {
        let (sender, receiver) = oneshot::channel();

        let request = PendingRequest {
            sender,
            deadline: Instant::now() + Duration::from_millis(timeout_ms),
        };

        let mut pending = self
            .pending
            .write()
            .map_err(|e| MessagingError::DeliveryFailed(format!("Lock poisoned: {}", e)))?;

        match pending.entry(correlation_id.as_str().to_owned()) {
            Entry::Occupied(_) => Err(MessagingError::DeliveryFailed(format!(
                "Correlation already exists: {}",
                correlation_id
            ))),
            Entry::Vacant(e) => {
                e.insert(request);
                Ok(receiver)
            }
        }
    }

    /// Cleans up all expired correlations.
    ///
    /// Removes all correlations whose deadline has passed. The oneshot senders
    /// are dropped, which causes any associated receivers to return an error.
    ///
    /// # Errors
    ///
    /// Returns `MessagingError::DeliveryFailed` if the internal lock is poisoned.
    pub fn cleanup_expired(&self) -> Result<(), MessagingError> {
        let now = Instant::now();
        let mut pending = self
            .pending
            .write()
            .map_err(|e| MessagingError::DeliveryFailed(format!("Lock poisoned: {}", e)))?;
        pending.retain(|_, req| req.deadline > now);
        Ok(())
    }
}

impl CorrelationTracker for CorrelationTrackerImpl {
    fn register(
        &self,
        correlation_id: &CorrelationId,
        timeout_ms: u64,
    ) -> Result<(), MessagingError> {
        let (sender, _receiver) = oneshot::channel();

        let request = PendingRequest {
            sender,
            deadline: Instant::now() + Duration::from_millis(timeout_ms),
        };

        let mut pending = self
            .pending
            .write()
            .map_err(|e| MessagingError::DeliveryFailed(format!("Lock poisoned: {}", e)))?;

        match pending.entry(correlation_id.as_str().to_owned()) {
            Entry::Occupied(_) => Err(MessagingError::DeliveryFailed(format!(
                "Correlation already exists: {}",
                correlation_id
            ))),
            Entry::Vacant(e) => {
                e.insert(request);
                Ok(())
            }
        }
    }

    fn complete(
        &self,
        correlation_id: &CorrelationId,
        response: MessagePayload,
    ) -> Result<(), MessagingError> {
        let mut pending = self
            .pending
            .write()
            .map_err(|e| MessagingError::DeliveryFailed(format!("Lock poisoned: {}", e)))?;

        if let Some(request) = pending.remove(correlation_id.as_str()) {
            if request.deadline > Instant::now() {
                let _ = request.sender.send(response);
                Ok(())
            } else {
                Err(MessagingError::CorrelationTimeout(
                    correlation_id.as_str().to_owned(),
                ))
            }
        } else {
            Err(MessagingError::InvalidMessage(format!(
                "No pending request for correlation {}",
                correlation_id
            )))
        }
    }

    fn is_pending(&self, correlation_id: &CorrelationId) -> bool {
        let pending = match self.pending.read() {
            Ok(guard) => guard,
            Err(_) => return false,
        };
        pending.contains_key(correlation_id.as_str())
    }

    fn remove(&self, correlation_id: &CorrelationId) -> Result<(), MessagingError> {
        let mut pending = self
            .pending
            .write()
            .map_err(|e| MessagingError::DeliveryFailed(format!("Lock poisoned: {}", e)))?;

        if pending.remove(correlation_id.as_str()).is_some() {
            Ok(())
        } else {
            Err(MessagingError::DeliveryFailed(format!(
                "No pending request for correlation {}",
                correlation_id
            )))
        }
    }
}

impl Default for CorrelationTrackerImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracker_new() {
        let tracker = CorrelationTrackerImpl::new();
        let id = CorrelationId::new("nonexistent");
        assert!(!tracker.is_pending(&id));
    }

    #[test]
    fn test_tracker_default() {
        let tracker = CorrelationTrackerImpl::default();
        let id = CorrelationId::new("nonexistent");
        assert!(!tracker.is_pending(&id));
    }

    #[test]
    fn test_register_correlation() {
        let tracker = CorrelationTrackerImpl::new();
        let correlation_id = CorrelationId::new("test-correlation-123");

        let result = tracker.register(&correlation_id, 5000);
        assert!(result.is_ok());
        assert!(tracker.is_pending(&correlation_id));
    }

    #[test]
    fn test_complete_correlation_success() {
        let tracker = CorrelationTrackerImpl::new();
        let correlation_id = CorrelationId::new("test-correlation-456");

        tracker.register(&correlation_id, 5000).unwrap();
        assert!(tracker.is_pending(&correlation_id));

        let response = MessagePayload::new(vec![1, 2, 3]);
        let result = tracker.complete(&correlation_id, response);
        assert!(result.is_ok());
        assert!(!tracker.is_pending(&correlation_id));
    }

    #[test]
    fn test_complete_nonexistent_correlation() {
        let tracker = CorrelationTrackerImpl::new();
        let correlation_id = CorrelationId::new("nonexistent-correlation");

        let response = MessagePayload::new(vec![10, 20, 30]);
        let result = tracker.complete(&correlation_id, response);
        assert!(result.is_err());
        assert!(matches!(result, Err(MessagingError::InvalidMessage(_))));
    }

    #[test]
    fn test_create_and_receive() {
        let tracker = CorrelationTrackerImpl::new();
        let correlation_id = CorrelationId::new("test-correlation-789");

        let mut receiver = tracker.create(&correlation_id, 5000).unwrap();
        assert!(tracker.is_pending(&correlation_id));

        // Complete the correlation
        let response = MessagePayload::new(vec![1, 2, 3]);
        tracker.complete(&correlation_id, response).unwrap();

        // Verify receiver got the response
        let received = receiver.try_recv();
        assert!(received.is_ok());
    }

    #[test]
    fn test_is_pending_after_complete() {
        let tracker = CorrelationTrackerImpl::new();
        let correlation_id = CorrelationId::new("test-correlation-complete");

        tracker.register(&correlation_id, 5000).unwrap();
        assert!(tracker.is_pending(&correlation_id));

        let response = MessagePayload::new(vec![100]);
        tracker.complete(&correlation_id, response).unwrap();

        assert!(!tracker.is_pending(&correlation_id));
    }

    #[test]
    fn test_multiple_correlations() {
        let tracker = CorrelationTrackerImpl::new();
        let id1 = CorrelationId::new("correlation-1");
        let id2 = CorrelationId::new("correlation-2");
        let id3 = CorrelationId::new("correlation-3");

        tracker.register(&id1, 5000).unwrap();
        tracker.register(&id2, 5000).unwrap();
        tracker.register(&id3, 5000).unwrap();

        assert!(tracker.is_pending(&id1));
        assert!(tracker.is_pending(&id2));
        assert!(tracker.is_pending(&id3));

        // Complete one
        tracker
            .complete(&id2, MessagePayload::new(vec![42]))
            .unwrap();

        assert!(tracker.is_pending(&id1));
        assert!(!tracker.is_pending(&id2));
        assert!(tracker.is_pending(&id3));
    }

    #[test]
    fn test_remove_correlation() {
        let tracker = CorrelationTrackerImpl::new();
        let correlation_id = CorrelationId::new("remove-me");

        tracker.register(&correlation_id, 5000).unwrap();
        assert!(tracker.is_pending(&correlation_id));

        let result = tracker.remove(&correlation_id);
        assert!(result.is_ok());
        assert!(!tracker.is_pending(&correlation_id));
    }

    #[test]
    fn test_remove_nonexistent_correlation() {
        let tracker = CorrelationTrackerImpl::new();
        let correlation_id = CorrelationId::new("nonexistent");

        let result = tracker.remove(&correlation_id);
        assert!(result.is_err());
        assert!(matches!(result, Err(MessagingError::DeliveryFailed(_))));
    }

    #[test]
    fn test_cleanup_expired_with_valid_correlations() {
        let tracker = CorrelationTrackerImpl::new();
        let correlation_id = CorrelationId::new("test-correlation-valid");

        // Register with long timeout (won't expire)
        tracker.register(&correlation_id, 60_000).unwrap();
        assert!(tracker.is_pending(&correlation_id));

        // Cleanup should not remove valid correlation
        tracker.cleanup_expired().unwrap();
        assert!(tracker.is_pending(&correlation_id));
    }

    #[tokio::test]
    async fn test_cleanup_expired_with_expired_correlations() {
        let tracker = CorrelationTrackerImpl::new();
        let correlation_id = CorrelationId::new("test-correlation-expired");

        // Register with very short timeout
        tracker.register(&correlation_id, 1).unwrap();
        assert!(tracker.is_pending(&correlation_id));

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Cleanup should remove expired correlation
        tracker.cleanup_expired().unwrap();
        assert!(!tracker.is_pending(&correlation_id));
    }

    #[tokio::test]
    async fn test_complete_expired_correlation_fails() {
        let tracker = CorrelationTrackerImpl::new();
        let correlation_id = CorrelationId::new("test-correlation-timeout");

        // Register with very short timeout
        tracker.register(&correlation_id, 1).unwrap();

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Try to complete expired correlation
        let response = MessagePayload::new(vec![99]);
        let result = tracker.complete(&correlation_id, response);
        assert!(result.is_err());
        assert!(matches!(result, Err(MessagingError::CorrelationTimeout(_))));
    }

    #[test]
    fn test_concurrent_access() {
        use std::sync::Arc;
        use std::thread;

        let tracker = Arc::new(CorrelationTrackerImpl::new());
        let mut handles = vec![];

        // Spawn multiple threads registering correlations
        for i in 0..10 {
            let tracker_clone = Arc::clone(&tracker);
            let handle = thread::spawn(move || {
                let correlation_id = CorrelationId::new(format!("correlation-{}", i));
                tracker_clone.register(&correlation_id, 5000).unwrap();
            });
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify all registered
        for i in 0..10 {
            let correlation_id = CorrelationId::new(format!("correlation-{}", i));
            assert!(tracker.is_pending(&correlation_id));
        }
    }

    #[test]
    fn test_register_duplicate_correlation_fails() {
        let tracker = CorrelationTrackerImpl::new();
        let correlation_id = CorrelationId::new("duplicate-id");

        // First registration succeeds
        let result = tracker.register(&correlation_id, 5000);
        assert!(result.is_ok());

        // Second registration with same ID fails
        let result = tracker.register(&correlation_id, 5000);
        assert!(result.is_err());
        assert!(matches!(result, Err(MessagingError::DeliveryFailed(_))));
    }

    #[test]
    fn test_create_duplicate_correlation_fails() {
        let tracker = CorrelationTrackerImpl::new();
        let correlation_id = CorrelationId::new("duplicate-create-id");

        // First create succeeds
        let result = tracker.create(&correlation_id, 5000);
        assert!(result.is_ok());

        // Second create with same ID fails
        let result = tracker.create(&correlation_id, 5000);
        assert!(result.is_err());
        assert!(matches!(result, Err(MessagingError::DeliveryFailed(_))));
    }

    #[test]
    fn test_tracker_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<CorrelationTrackerImpl>();
    }
}
