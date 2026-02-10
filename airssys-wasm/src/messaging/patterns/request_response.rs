//! Request-response messaging pattern.
//!
//! Provides the [`RequestResponse`] pattern for sending requests
//! and correlating responses via [`CorrelationId`].
//!
//! # Architecture
//!
//! This module is part of `messaging/patterns/` (Layer 3B). It depends on:
//! - `core/component/` for `ComponentId` and `MessagePayload`
//! - `core/messaging/` for `MessagingError`, `CorrelationId`, `MessageSender`, and `CorrelationManager`
//!
//! # References
//!
//! - ADR-WASM-031: Component & Messaging Module Design
//! - ADR-WASM-009: Component Communication Model (Pattern 2: Request-Response)
//! - KNOWLEDGE-WASM-029: Fire-and-Forget vs Request-Response

// Layer 1: Standard library imports
// (none needed)

// Layer 2: Third-party crate imports
// (none needed)

// Layer 3: Internal module imports
use crate::core::component::id::ComponentId;
use crate::core::component::message::MessagePayload;
use crate::core::messaging::correlation::CorrelationId;
use crate::core::messaging::errors::MessagingError;
use crate::core::messaging::traits::CorrelationManager;
use crate::core::messaging::traits::MessageSender;

/// Request-response messaging pattern.
///
/// Sends a request to a target component and tracks the correlation for
/// response matching. The caller receives a [`CorrelationId`] that can be
/// used to match the response when it arrives via `handle_callback`.
///
/// Best for RPC-style interactions, database queries, validation requests,
/// and any operation where a response is expected.
///
/// # Architecture
///
/// This pattern uses two trait abstractions from `core/messaging/traits`:
/// - [`MessageSender`]: Routes the request message to the target component
/// - [`CorrelationManager`]: Registers and tracks the pending request
///
/// Concrete implementations are injected by higher layers (e.g., `system/`).
///
/// # Performance
///
/// Target round-trip latency: ~560ns (ADR-WASM-009).
///
/// # Examples
///
/// ```rust,no_run
/// use airssys_wasm::messaging::patterns::request_response::RequestResponse;
/// use airssys_wasm::core::messaging::traits::{MessageSender, CorrelationManager};
/// use airssys_wasm::core::component::id::ComponentId;
/// use airssys_wasm::core::component::message::MessagePayload;
///
/// # async fn example(
/// #     router: &impl MessageSender,
/// #     tracker: &impl CorrelationManager,
/// # ) {
/// let target = ComponentId::new("app", "user-service", "v1");
/// let payload = MessagePayload::new(b"query-user-123".to_vec());
///
/// let correlation_id = RequestResponse::request(
///     &target,
///     payload,
///     5000, // 5 second timeout
///     router,
///     tracker,
/// ).await;
/// # }
/// ```
pub struct RequestResponse;

impl RequestResponse {
    /// Send a request and register correlation for response tracking.
    ///
    /// Generates a unique [`CorrelationId`], registers it with the
    /// [`CorrelationManager`] for timeout tracking, then sends the request
    /// with the correlation ID attached via [`MessageSender::send_with_correlation`].
    ///
    /// The caller receives the correlation ID immediately. The actual response
    /// will arrive later via the component's `handle_callback` export.
    ///
    /// # Arguments
    ///
    /// * `target` - The component to send the request to
    /// * `payload` - The request payload (raw bytes)
    /// * `timeout_ms` - Maximum time to wait for response in milliseconds
    /// * `router` - The message routing implementation
    /// * `tracker` - The correlation tracking implementation
    ///
    /// # Returns
    ///
    /// `Ok(CorrelationId)` for tracking the response.
    ///
    /// # Errors
    ///
    /// - [`MessagingError::DeliveryFailed`] if correlation registration fails
    /// - [`MessagingError::TargetNotFound`] if the target component does not exist
    /// - [`MessagingError::DeliveryFailed`] if message delivery fails
    /// - [`MessagingError::QueueFull`] if the message queue is at capacity
    ///
    /// # Error Handling
    ///
    /// If correlation registration succeeds but message sending fails, the
    /// correlation remains registered. The `CorrelationManager` implementation
    /// is responsible for cleaning up timed-out correlations.
    pub async fn request(
        target: &ComponentId,
        payload: MessagePayload,
        timeout_ms: u64,
        router: &impl MessageSender,
        tracker: &impl CorrelationManager,
    ) -> Result<CorrelationId, MessagingError> {
        // Generate unique correlation ID
        let correlation_id = CorrelationId::generate();

        // Register correlation with timeout tracking
        tracker
            .register(correlation_id.as_str(), timeout_ms)
            .await?;

        // Send request with correlation ID attached
        router
            .send_with_correlation(target, payload, correlation_id.as_str())
            .await?;

        Ok(correlation_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // Mock implementations
    // ---------------------------------------------------------------

    /// Mock MessageSender that can be configured to succeed or fail.
    struct MockMessageSender {
        should_fail: bool,
    }

    impl MockMessageSender {
        fn new() -> Self {
            Self { should_fail: false }
        }

        fn failing() -> Self {
            Self { should_fail: true }
        }
    }

    impl MessageSender for MockMessageSender {
        async fn send(
            &self,
            _target: &ComponentId,
            _payload: MessagePayload,
        ) -> Result<(), MessagingError> {
            if self.should_fail {
                Err(MessagingError::TargetNotFound(
                    "mock: target not found".to_string(),
                ))
            } else {
                Ok(())
            }
        }

        async fn send_with_correlation(
            &self,
            _target: &ComponentId,
            _payload: MessagePayload,
            _correlation_id: &str,
        ) -> Result<(), MessagingError> {
            if self.should_fail {
                Err(MessagingError::DeliveryFailed(
                    "mock: delivery failed".to_string(),
                ))
            } else {
                Ok(())
            }
        }
    }

    /// Mock MessageSender that fails with QueueFull error.
    struct QueueFullSender;

    impl MessageSender for QueueFullSender {
        async fn send(
            &self,
            _target: &ComponentId,
            _payload: MessagePayload,
        ) -> Result<(), MessagingError> {
            Err(MessagingError::QueueFull)
        }

        async fn send_with_correlation(
            &self,
            _target: &ComponentId,
            _payload: MessagePayload,
            _correlation_id: &str,
        ) -> Result<(), MessagingError> {
            Err(MessagingError::QueueFull)
        }
    }

    /// Mock CorrelationManager that can be configured to succeed or fail.
    struct MockCorrelationManager {
        should_fail_register: bool,
    }

    impl MockCorrelationManager {
        fn new() -> Self {
            Self {
                should_fail_register: false,
            }
        }

        fn with_register_failure() -> Self {
            Self {
                should_fail_register: true,
            }
        }
    }

    impl CorrelationManager for MockCorrelationManager {
        async fn register(&self, _id: &str, _timeout_ms: u64) -> Result<(), MessagingError> {
            if self.should_fail_register {
                Err(MessagingError::DeliveryFailed(
                    "mock: registration failed".to_string(),
                ))
            } else {
                Ok(())
            }
        }

        async fn complete(
            &self,
            _id: &str,
            _response: MessagePayload,
        ) -> Result<(), MessagingError> {
            Ok(())
        }

        async fn is_pending(&self, _id: &str) -> bool {
            true
        }
    }

    // ---------------------------------------------------------------
    // RequestResponse tests
    // ---------------------------------------------------------------

    #[tokio::test]
    async fn test_request_response_success() {
        let router = MockMessageSender::new();
        let tracker = MockCorrelationManager::new();
        let target = ComponentId::new("app", "service", "001");
        let payload = MessagePayload::new(vec![1, 2, 3]);

        let result = RequestResponse::request(&target, payload, 5000, &router, &tracker).await;
        assert!(result.is_ok());

        let correlation_id = result.unwrap();
        assert!(!correlation_id.as_str().is_empty());
    }

    #[tokio::test]
    async fn test_request_response_returns_valid_uuid() {
        let router = MockMessageSender::new();
        let tracker = MockCorrelationManager::new();
        let target = ComponentId::new("app", "service", "001");
        let payload = MessagePayload::new(vec![1, 2, 3]);

        let correlation_id = RequestResponse::request(&target, payload, 5000, &router, &tracker)
            .await
            .unwrap();

        // CorrelationId::generate() produces UUID v4 format: 36 chars with dashes
        assert_eq!(correlation_id.as_str().len(), 36);
        assert!(correlation_id.as_str().contains('-'));
    }

    #[tokio::test]
    async fn test_request_response_generates_unique_ids() {
        let router = MockMessageSender::new();
        let tracker = MockCorrelationManager::new();
        let target = ComponentId::new("app", "service", "001");
        let payload1 = MessagePayload::new(vec![1, 2, 3]);
        let payload2 = MessagePayload::new(vec![4, 5, 6]);

        let id1 = RequestResponse::request(&target, payload1, 5000, &router, &tracker)
            .await
            .unwrap();
        let id2 = RequestResponse::request(&target, payload2, 5000, &router, &tracker)
            .await
            .unwrap();

        assert_ne!(id1, id2);
    }

    #[tokio::test]
    async fn test_request_response_correlation_registration_fails() {
        let router = MockMessageSender::new();
        let tracker = MockCorrelationManager::with_register_failure();
        let target = ComponentId::new("app", "service", "001");
        let payload = MessagePayload::new(vec![1, 2, 3]);

        let result = RequestResponse::request(&target, payload, 5000, &router, &tracker).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(MessagingError::DeliveryFailed(_))));
    }

    #[tokio::test]
    async fn test_request_response_send_fails() {
        let router = MockMessageSender::failing();
        let tracker = MockCorrelationManager::new();
        let target = ComponentId::new("app", "service", "001");
        let payload = MessagePayload::new(vec![1, 2, 3]);

        let result = RequestResponse::request(&target, payload, 5000, &router, &tracker).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(MessagingError::DeliveryFailed(_))));
    }

    #[tokio::test]
    async fn test_request_response_send_queue_full() {
        let router = QueueFullSender;
        let tracker = MockCorrelationManager::new();
        let target = ComponentId::new("app", "service", "001");
        let payload = MessagePayload::new(vec![1, 2, 3]);

        let result = RequestResponse::request(&target, payload, 5000, &router, &tracker).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(MessagingError::QueueFull)));
    }

    #[tokio::test]
    async fn test_request_response_with_various_timeouts() {
        let router = MockMessageSender::new();
        let tracker = MockCorrelationManager::new();
        let target = ComponentId::new("app", "service", "001");

        // Short timeout
        let payload = MessagePayload::new(vec![1]);
        let result = RequestResponse::request(&target, payload, 100, &router, &tracker).await;
        assert!(result.is_ok());

        // Long timeout
        let payload = MessagePayload::new(vec![2]);
        let result = RequestResponse::request(&target, payload, 60_000, &router, &tracker).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_request_response_with_empty_payload() {
        let router = MockMessageSender::new();
        let tracker = MockCorrelationManager::new();
        let target = ComponentId::new("app", "service", "001");
        let payload = MessagePayload::new(vec![]);

        let result = RequestResponse::request(&target, payload, 5000, &router, &tracker).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_request_response_with_binary_payload() {
        let router = MockMessageSender::new();
        let tracker = MockCorrelationManager::new();
        let target = ComponentId::new("app", "codec", "v1");
        let payload = MessagePayload::new(vec![0xDE, 0xAD, 0xBE, 0xEF]);

        let result = RequestResponse::request(&target, payload, 5000, &router, &tracker).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_request_response_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<RequestResponse>();
    }

    // ---------------------------------------------------------------
    // CorrelationManager trait tests
    // ---------------------------------------------------------------

    #[tokio::test]
    async fn test_correlation_manager_register() {
        let tracker = MockCorrelationManager::new();
        let result = tracker.register("test-correlation", 5000).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_correlation_manager_complete() {
        let tracker = MockCorrelationManager::new();
        let payload = MessagePayload::new(vec![10, 20, 30]);
        let result = tracker.complete("test-correlation", payload).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_correlation_manager_is_pending() {
        let tracker = MockCorrelationManager::new();
        let is_pending = tracker.is_pending("test-correlation").await;
        assert!(is_pending);
    }
}
