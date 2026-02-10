//! Messaging trait abstractions.
//!
//! This module contains trait definitions for message routing and correlation
//! tracking. These traits are implemented by the `messaging/` module (Layer 3)
//! and used by other modules through dependency injection.

// Layer 1: Standard library imports (per PROJECTS_STANDARD.md §2.1)
// (none)

// Layer 2: Third-party crate imports (per PROJECTS_STANDARD.md §2.1)
// (none)

// Layer 3: Internal module imports (per PROJECTS_STANDARD.md §2.1)
use super::correlation::CorrelationId;
use super::errors::MessagingError;
use crate::core::component::id::ComponentId;
use crate::core::component::message::MessagePayload;

/// Trait for message routing between components.
///
/// `MessageRouter` defines the interface for sending messages between WASM
/// components. It supports both fire-and-forget and request-response patterns.
///
/// # Architecture Note
///
/// This trait is defined in `core/messaging/` (Layer 1) as an abstraction.
/// The concrete implementation (e.g., `ResponseRouter`) lives in `messaging/`
/// module (Layer 3). This follows the Dependency Inversion Principle.
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync` to allow use from multiple threads
/// and async contexts.
///
/// # Examples
///
/// ```rust
/// use airssys_wasm::core::messaging::traits::MessageRouter;
/// use airssys_wasm::core::messaging::errors::MessagingError;
/// use airssys_wasm::core::messaging::correlation::CorrelationId;
/// use airssys_wasm::core::component::id::ComponentId;
/// use airssys_wasm::core::component::message::MessagePayload;
///
/// struct MockRouter;
///
/// impl MessageRouter for MockRouter {
///     fn send(
///         &self,
///         _target: &ComponentId,
///         _payload: MessagePayload,
///     ) -> Result<(), MessagingError> {
///         Ok(())
///     }
///
///     fn request(
///         &self,
///         _target: &ComponentId,
///         _payload: MessagePayload,
///         _timeout_ms: u64,
///     ) -> Result<CorrelationId, MessagingError> {
///         Ok(CorrelationId::generate())
///     }
///
///     fn cancel_request(
///         &self,
///         _correlation_id: &CorrelationId,
///     ) -> Result<(), MessagingError> {
///         Ok(())
///     }
/// }
/// ```
pub trait MessageRouter: Send + Sync {
    /// Sends a fire-and-forget message to a target component.
    ///
    /// The sender does not wait for a response. Any return value from the
    /// target's `handle_message` export is ignored by the runtime.
    ///
    /// # Arguments
    ///
    /// * `target` - ComponentId of the message recipient
    /// * `payload` - Message payload to send
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the message was queued for delivery, or an error
    /// if delivery failed immediately.
    ///
    /// # Errors
    ///
    /// - `MessagingError::TargetNotFound` - Target component does not exist
    /// - `MessagingError::QueueFull` - Message queue is at capacity
    /// - `MessagingError::DeliveryFailed` - Delivery failed for other reasons
    fn send(&self, target: &ComponentId, payload: MessagePayload) -> Result<(), MessagingError>;

    /// Sends a request expecting a response from the target.
    ///
    /// The runtime tracks the request and correlates the response. When the
    /// target component returns from `handle_message`, the runtime captures
    /// the return value and delivers it to the requester via `handle_callback`.
    ///
    /// # Arguments
    ///
    /// * `target` - ComponentId of the message recipient
    /// * `payload` - Request payload to send
    /// * `timeout_ms` - Maximum time to wait for response in milliseconds
    ///
    /// # Returns
    ///
    /// Returns a `CorrelationId` that uniquely identifies this request.
    /// The correlation ID is used to match the response when it arrives.
    ///
    /// # Errors
    ///
    /// - `MessagingError::TargetNotFound` - Target component does not exist
    /// - `MessagingError::QueueFull` - Message queue is at capacity
    /// - `MessagingError::DeliveryFailed` - Request failed to send
    fn request(
        &self,
        target: &ComponentId,
        payload: MessagePayload,
        timeout_ms: u64,
    ) -> Result<CorrelationId, MessagingError>;

    /// Cancels a pending request before timeout.
    ///
    /// This removes the correlation tracking for the given request. If a
    /// response arrives after cancellation, it will be discarded.
    ///
    /// # Arguments
    ///
    /// * `correlation_id` - ID of the request to cancel
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the request was cancelled, or an error if the
    /// request was not found or already completed.
    ///
    /// # Errors
    ///
    /// - `MessagingError::DeliveryFailed` - Request not found or already completed
    fn cancel_request(&self, correlation_id: &CorrelationId) -> Result<(), MessagingError>;
}

/// Trait for tracking request-response correlations.
///
/// `CorrelationTracker` manages pending requests and their timeouts. It is
/// used internally by the messaging system to match responses with their
/// original requests.
///
/// # Architecture Note
///
/// This trait is defined in `core/messaging/` (Layer 1) as an abstraction.
/// The concrete implementation (e.g., `CorrelationTrackerImpl`) lives in
/// `messaging/` module (Layer 3).
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync` to allow concurrent access from
/// multiple component executors.
///
/// # Examples
///
/// ```rust
/// use airssys_wasm::core::messaging::traits::CorrelationTracker;
/// use airssys_wasm::core::messaging::errors::MessagingError;
/// use airssys_wasm::core::messaging::correlation::CorrelationId;
/// use airssys_wasm::core::component::message::MessagePayload;
///
/// struct MockTracker;
///
/// impl CorrelationTracker for MockTracker {
///     fn register(
///         &self,
///         _correlation_id: &CorrelationId,
///         _timeout_ms: u64,
///     ) -> Result<(), MessagingError> {
///         Ok(())
///     }
///
///     fn complete(
///         &self,
///         _correlation_id: &CorrelationId,
///         _response: MessagePayload,
///     ) -> Result<(), MessagingError> {
///         Ok(())
///     }
///
///     fn is_pending(&self, _correlation_id: &CorrelationId) -> bool {
///         false
///     }
///
///     fn remove(
///         &self,
///         _correlation_id: &CorrelationId,
///     ) -> Result<(), MessagingError> {
///         Ok(())
///     }
/// }
/// ```
pub trait CorrelationTracker: Send + Sync {
    /// Registers a pending request with timeout.
    ///
    /// This starts tracking a new request. If no response arrives within
    /// the timeout period, the request will be marked as timed out.
    ///
    /// # Arguments
    ///
    /// * `correlation_id` - Unique ID for this request
    /// * `timeout_ms` - Maximum time to wait for response in milliseconds
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the request was registered successfully.
    ///
    /// # Errors
    ///
    /// - `MessagingError::DeliveryFailed` - Request ID already exists
    fn register(
        &self,
        correlation_id: &CorrelationId,
        timeout_ms: u64,
    ) -> Result<(), MessagingError>;

    /// Completes a pending request with a response.
    ///
    /// This marks the request as completed and stores the response for
    /// delivery to the requester via `handle_callback`.
    ///
    /// # Arguments
    ///
    /// * `correlation_id` - ID of the request to complete
    /// * `response` - Response payload from the target component
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the request was completed successfully.
    ///
    /// # Errors
    ///
    /// - `MessagingError::DeliveryFailed` - Request not found or already completed
    fn complete(
        &self,
        correlation_id: &CorrelationId,
        response: MessagePayload,
    ) -> Result<(), MessagingError>;

    /// Checks if a correlation is still pending.
    ///
    /// # Arguments
    ///
    /// * `correlation_id` - ID of the request to check
    ///
    /// # Returns
    ///
    /// Returns `true` if the request is still waiting for a response,
    /// `false` if completed, timed out, or not found.
    fn is_pending(&self, correlation_id: &CorrelationId) -> bool;

    /// Removes a correlation without completing it.
    ///
    /// This is used for cancellation or cleanup. The request is removed
    /// from tracking without delivering a response.
    ///
    /// # Arguments
    ///
    /// * `correlation_id` - ID of the request to remove
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the request was removed successfully.
    ///
    /// # Errors
    ///
    /// - `MessagingError::DeliveryFailed` - Request not found
    fn remove(&self, correlation_id: &CorrelationId) -> Result<(), MessagingError>;
}

/// Trait for sending messages between components asynchronously.
///
/// Abstracts over message routing, enabling dependency injection
/// and testing with mock implementations. Concrete implementations
/// are provided by higher layers (e.g., `ResponseRouter` in `messaging/router.rs`).
///
/// # Relationship to `MessageRouter`
///
/// `MessageRouter` (above) provides synchronous message routing for contexts
/// where blocking is acceptable. `MessageSender` provides async equivalents
/// for use in the messaging layer patterns (fire-and-forget, request-response).
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync` for use across async tasks.
pub trait MessageSender: Send + Sync {
    /// Send a message without correlation tracking.
    ///
    /// Used by the fire-and-forget pattern. The message is routed to the
    /// target component without any response tracking.
    ///
    /// # Arguments
    ///
    /// * `target` - The component to send to
    /// * `payload` - The message payload
    ///
    /// # Errors
    ///
    /// - [`MessagingError::TargetNotFound`] if target does not exist
    /// - [`MessagingError::DeliveryFailed`] if routing fails
    /// - [`MessagingError::QueueFull`] if queue is at capacity
    fn send(
        &self,
        target: &ComponentId,
        payload: MessagePayload,
    ) -> impl std::future::Future<Output = Result<(), MessagingError>> + Send;

    /// Send a message with a correlation ID for request-response tracking.
    ///
    /// Used by the request-response pattern. The correlation ID is included
    /// in the message metadata so the runtime can match the response.
    ///
    /// # Arguments
    ///
    /// * `target` - The component to send to
    /// * `payload` - The message payload
    /// * `correlation_id` - The correlation identifier for tracking the response
    ///
    /// # Errors
    ///
    /// - [`MessagingError::TargetNotFound`] if target does not exist
    /// - [`MessagingError::DeliveryFailed`] if routing fails
    /// - [`MessagingError::QueueFull`] if queue is at capacity
    fn send_with_correlation(
        &self,
        target: &ComponentId,
        payload: MessagePayload,
        correlation_id: &str,
    ) -> impl std::future::Future<Output = Result<(), MessagingError>> + Send;
}

/// Trait for managing correlations in request-response patterns asynchronously.
///
/// Tracks pending requests and matches them with responses. Used
/// internally by the request-response messaging pattern.
///
/// # Relationship to `CorrelationTracker`
///
/// `CorrelationTracker` (above) provides synchronous correlation management
/// using `&CorrelationId`. `CorrelationManager` provides async equivalents
/// using `&str` for flexibility in the messaging layer.
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync` for concurrent access from
/// multiple component executors.
pub trait CorrelationManager: Send + Sync {
    /// Register a new correlation for tracking.
    ///
    /// Starts tracking a pending request. If no response arrives within
    /// the timeout period, the request should be considered timed out.
    ///
    /// # Arguments
    ///
    /// * `id` - The correlation identifier
    /// * `timeout_ms` - Timeout in milliseconds
    ///
    /// # Errors
    ///
    /// - [`MessagingError::DeliveryFailed`] if registration fails
    fn register(
        &self,
        id: &str,
        timeout_ms: u64,
    ) -> impl std::future::Future<Output = Result<(), MessagingError>> + Send;

    /// Complete a correlation with a response payload.
    ///
    /// Marks the request as completed and delivers the response.
    ///
    /// # Arguments
    ///
    /// * `id` - The correlation identifier
    /// * `response` - The response payload from the target component
    ///
    /// # Errors
    ///
    /// - [`MessagingError::InvalidMessage`] if correlation ID not found
    /// - [`MessagingError::CorrelationTimeout`] if the request has timed out
    fn complete(
        &self,
        id: &str,
        response: MessagePayload,
    ) -> impl std::future::Future<Output = Result<(), MessagingError>> + Send;

    /// Check if a correlation is still pending.
    ///
    /// # Arguments
    ///
    /// * `id` - The correlation identifier
    ///
    /// # Returns
    ///
    /// `true` if the request is still waiting for a response,
    /// `false` if completed, timed out, or not found.
    fn is_pending(&self, id: &str) -> impl std::future::Future<Output = bool> + Send;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock implementations for testing
    struct MockMessageRouter;

    impl MessageRouter for MockMessageRouter {
        fn send(
            &self,
            _target: &ComponentId,
            _payload: MessagePayload,
        ) -> Result<(), MessagingError> {
            Ok(())
        }

        fn request(
            &self,
            _target: &ComponentId,
            _payload: MessagePayload,
            _timeout_ms: u64,
        ) -> Result<CorrelationId, MessagingError> {
            Ok(CorrelationId::new("mock-id"))
        }

        fn cancel_request(&self, _correlation_id: &CorrelationId) -> Result<(), MessagingError> {
            Ok(())
        }
    }

    struct MockCorrelationTracker {
        pending: std::sync::atomic::AtomicBool,
    }

    impl MockCorrelationTracker {
        fn new() -> Self {
            Self {
                pending: std::sync::atomic::AtomicBool::new(false),
            }
        }
    }

    impl CorrelationTracker for MockCorrelationTracker {
        fn register(
            &self,
            _correlation_id: &CorrelationId,
            _timeout_ms: u64,
        ) -> Result<(), MessagingError> {
            self.pending
                .store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }

        fn complete(
            &self,
            _correlation_id: &CorrelationId,
            _response: MessagePayload,
        ) -> Result<(), MessagingError> {
            self.pending
                .store(false, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }

        fn is_pending(&self, _correlation_id: &CorrelationId) -> bool {
            self.pending.load(std::sync::atomic::Ordering::SeqCst)
        }

        fn remove(&self, _correlation_id: &CorrelationId) -> Result<(), MessagingError> {
            self.pending
                .store(false, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }
    }

    #[test]
    fn test_message_router_send() {
        let router = MockMessageRouter;
        let target = ComponentId::new("app", "service", "001");
        let payload = MessagePayload::new(vec![1, 2, 3]);

        let result = router.send(&target, payload);
        assert!(result.is_ok());
    }

    #[test]
    fn test_message_router_request() {
        let router = MockMessageRouter;
        let target = ComponentId::new("app", "service", "001");
        let payload = MessagePayload::new(vec![1, 2, 3]);

        let result = router.request(&target, payload, 5000);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "mock-id");
    }

    #[test]
    fn test_message_router_cancel_request() {
        let router = MockMessageRouter;
        let correlation_id = CorrelationId::new("cancel-me");

        let result = router.cancel_request(&correlation_id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_correlation_tracker_register() {
        let tracker = MockCorrelationTracker::new();
        let correlation_id = CorrelationId::new("test-123");

        let result = tracker.register(&correlation_id, 5000);
        assert!(result.is_ok());
        assert!(tracker.is_pending(&correlation_id));
    }

    #[test]
    fn test_correlation_tracker_complete() {
        let tracker = MockCorrelationTracker::new();
        let correlation_id = CorrelationId::new("test-123");
        let response = MessagePayload::new(vec![4, 5, 6]);

        tracker.register(&correlation_id, 5000).unwrap();
        assert!(tracker.is_pending(&correlation_id));

        let result = tracker.complete(&correlation_id, response);
        assert!(result.is_ok());
        assert!(!tracker.is_pending(&correlation_id));
    }

    #[test]
    fn test_correlation_tracker_remove() {
        let tracker = MockCorrelationTracker::new();
        let correlation_id = CorrelationId::new("test-123");

        tracker.register(&correlation_id, 5000).unwrap();
        assert!(tracker.is_pending(&correlation_id));

        let result = tracker.remove(&correlation_id);
        assert!(result.is_ok());
        assert!(!tracker.is_pending(&correlation_id));
    }

    #[test]
    fn test_message_router_is_send_sync() {
        fn assert_send_sync<T: Send + Sync + ?Sized>() {}
        assert_send_sync::<dyn MessageRouter>();
    }

    #[test]
    fn test_correlation_tracker_is_send_sync() {
        fn assert_send_sync<T: Send + Sync + ?Sized>() {}
        assert_send_sync::<dyn CorrelationTracker>();
    }

    // Gap analysis tests

    #[test]
    fn test_message_router_error_propagation() {
        struct FailingRouter;

        impl MessageRouter for FailingRouter {
            fn send(
                &self,
                _target: &ComponentId,
                _payload: MessagePayload,
            ) -> Result<(), MessagingError> {
                Err(MessagingError::TargetNotFound("not-found".to_string()))
            }

            fn request(
                &self,
                _target: &ComponentId,
                _payload: MessagePayload,
                _timeout_ms: u64,
            ) -> Result<CorrelationId, MessagingError> {
                Err(MessagingError::QueueFull)
            }

            fn cancel_request(
                &self,
                _correlation_id: &CorrelationId,
            ) -> Result<(), MessagingError> {
                Err(MessagingError::DeliveryFailed("cancelled".to_string()))
            }
        }

        let router = FailingRouter;
        let target = ComponentId::new("a", "b", "c");
        let payload = MessagePayload::new(vec![]);

        let result = router.send(&target, payload);
        assert!(matches!(result, Err(MessagingError::TargetNotFound(_))));
    }

    // ---------------------------------------------------------------
    // Lightweight mocks for async trait bound verification
    // ---------------------------------------------------------------

    /// Minimal mock for verifying MessageSender trait bounds.
    struct MockAsyncSender;

    impl MessageSender for MockAsyncSender {
        async fn send(
            &self,
            _target: &ComponentId,
            _payload: MessagePayload,
        ) -> Result<(), MessagingError> {
            Ok(())
        }

        async fn send_with_correlation(
            &self,
            _target: &ComponentId,
            _payload: MessagePayload,
            _correlation_id: &str,
        ) -> Result<(), MessagingError> {
            Ok(())
        }
    }

    /// Minimal mock for verifying CorrelationManager trait bounds.
    struct MockAsyncCorrelationMgr;

    impl CorrelationManager for MockAsyncCorrelationMgr {
        async fn register(&self, _id: &str, _timeout_ms: u64) -> Result<(), MessagingError> {
            Ok(())
        }

        async fn complete(
            &self,
            _id: &str,
            _response: MessagePayload,
        ) -> Result<(), MessagingError> {
            Ok(())
        }

        async fn is_pending(&self, _id: &str) -> bool {
            false
        }
    }

    // ---------------------------------------------------------------
    // Async trait tests (MessageSender, CorrelationManager)
    // ---------------------------------------------------------------

    #[test]
    fn test_message_sender_requires_send_sync() {
        // NOTE: Cannot use `dyn MessageSender` here because RPITIT traits
        // (using `-> impl Future + Send`) are NOT object-safe. Instead,
        // verify via a concrete mock type that the Send + Sync supertrait
        // bounds are satisfied.
        fn assert_send_sync<T: MessageSender + ?Sized>() {}
        assert_send_sync::<MockAsyncSender>();
    }

    #[test]
    fn test_correlation_manager_requires_send_sync() {
        // NOTE: Cannot use `dyn CorrelationManager` here because RPITIT
        // traits are NOT object-safe. See note above.
        fn assert_send_sync<T: CorrelationManager + ?Sized>() {}
        assert_send_sync::<MockAsyncCorrelationMgr>();
    }
}
