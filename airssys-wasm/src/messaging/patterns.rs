//! Messaging patterns for component communication.
//!
//! Provides high-level messaging patterns that abstract over
//! message routing and correlation:
//!
//! - [`FireAndForget`]: Send message without waiting for response
//! - `RequestResponse`: Send request and correlate response (WASM-TASK-042)
//!
//! # Architecture
//!
//! This module is part of `messaging/` (Layer 3B). It depends on:
//! - `core/component/` for `ComponentId` and `MessagePayload`
//! - `core/messaging/` for `MessagingError`
//!
//! The [`MessageSender`] and [`CorrelationManager`] traits enable
//! dependency injection: concrete implementations are provided by
//! higher layers (e.g., `system/`), and tests use mock implementations.
//!
//! # References
//!
//! - ADR-WASM-031: Component & Messaging Module Design
//! - ADR-WASM-009: Component Communication Model (Pattern 1: Fire-and-Forget)
//! - KNOWLEDGE-WASM-040: Messaging Module Comprehensive Reference

// Layer 1: Standard library imports
// (none needed)

// Layer 2: Third-party crate imports
// (none needed)

// Layer 3: Internal module imports
use crate::core::component::id::ComponentId;
use crate::core::component::message::MessagePayload;
use crate::core::messaging::errors::MessagingError;

/// Fire-and-forget messaging pattern.
///
/// Sends messages to a target component without expecting a response.
/// The sender returns immediately after the message is queued for delivery.
/// Any return value from the target's `handle_message` export is ignored
/// by the runtime.
///
/// Best for notifications, events, status updates, and one-way commands.
///
/// # Performance
///
/// Target latency: ~280ns per message (ADR-WASM-009).
///
/// # Examples
///
/// ```rust,no_run
/// use airssys_wasm::messaging::patterns::{FireAndForget, MessageSender};
/// use airssys_wasm::core::component::id::ComponentId;
/// use airssys_wasm::core::component::message::MessagePayload;
///
/// # async fn example(router: &impl MessageSender) {
/// let target = ComponentId::new("app", "analytics", "v1");
/// let payload = MessagePayload::new(vec![1, 2, 3]);
///
/// let result = FireAndForget::send(&target, payload, router).await;
/// # }
/// ```
pub struct FireAndForget;

impl FireAndForget {
    /// Send a message without expecting a response.
    ///
    /// Delegates to the provided [`MessageSender`] implementation to
    /// route the message to the target component. The sender does not
    /// track any correlation -- the message is fire-and-forget.
    ///
    /// # Arguments
    ///
    /// * `target` - The component to send to
    /// * `payload` - The message payload (raw bytes)
    /// * `router` - The message routing implementation
    ///
    /// # Returns
    ///
    /// `Ok(())` if the message was queued for delivery successfully.
    ///
    /// # Errors
    ///
    /// - [`MessagingError::TargetNotFound`] if the target component does not exist
    /// - [`MessagingError::DeliveryFailed`] if message delivery fails
    /// - [`MessagingError::QueueFull`] if the message queue is at capacity
    pub async fn send(
        target: &ComponentId,
        payload: MessagePayload,
        router: &impl MessageSender,
    ) -> Result<(), MessagingError> {
        router.send(target, payload).await
    }
}

/// Trait for sending messages between components.
///
/// Abstracts over message routing, enabling dependency injection
/// and testing with mock implementations. Concrete implementations
/// are provided by higher layers (e.g., `ResponseRouter` in `messaging/router.rs`).
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync` for use across async tasks.
///
/// # Design Note
///
/// This trait is defined at Layer 3 (`messaging/`) and is async, unlike
/// the synchronous `MessageRouter` trait in `core/messaging/traits.rs`
/// (Layer 0). The async nature reflects the runtime context where
/// message sending occurs.
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

/// Trait for managing correlations in request-response patterns.
///
/// Tracks pending requests and matches them with responses. Used
/// internally by the request-response messaging pattern (WASM-TASK-042).
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync` for concurrent access from
/// multiple component executors.
///
/// # Design Note
///
/// This trait complements `CorrelationTracker` in `core/messaging/traits.rs`
/// by providing an async interface suitable for use in the messaging layer.
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

    // ---------------------------------------------------------------
    // FireAndForget tests
    // ---------------------------------------------------------------

    #[tokio::test]
    async fn test_fire_and_forget_send_success() {
        let router = MockMessageSender::new();
        let target = ComponentId::new("app", "service", "001");
        let payload = MessagePayload::new(vec![1, 2, 3]);

        let result = FireAndForget::send(&target, payload, &router).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_fire_and_forget_send_target_not_found() {
        let router = MockMessageSender::failing();
        let target = ComponentId::new("app", "missing", "001");
        let payload = MessagePayload::new(vec![1, 2, 3]);

        let result = FireAndForget::send(&target, payload, &router).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(MessagingError::TargetNotFound(_))));
    }

    #[tokio::test]
    async fn test_fire_and_forget_send_queue_full() {
        let router = QueueFullSender;
        let target = ComponentId::new("app", "service", "001");
        let payload = MessagePayload::new(vec![1, 2, 3]);

        let result = FireAndForget::send(&target, payload, &router).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(MessagingError::QueueFull)));
    }

    #[tokio::test]
    async fn test_fire_and_forget_with_empty_payload() {
        let router = MockMessageSender::new();
        let target = ComponentId::new("app", "service", "001");
        let payload = MessagePayload::new(vec![]);

        let result = FireAndForget::send(&target, payload, &router).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_fire_and_forget_with_large_payload() {
        let router = MockMessageSender::new();
        let target = ComponentId::new("app", "analytics", "v1");
        let payload = MessagePayload::new(vec![0xDE, 0xAD, 0xBE, 0xEF]);

        let result = FireAndForget::send(&target, payload, &router).await;
        assert!(result.is_ok());
    }

    // ---------------------------------------------------------------
    // MessageSender trait tests
    // ---------------------------------------------------------------

    #[tokio::test]
    async fn test_message_sender_send_directly() {
        let sender = MockMessageSender::new();
        let target = ComponentId::new("ns", "comp", "inst");
        let payload = MessagePayload::new(vec![10, 20, 30]);

        let result = sender.send(&target, payload).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_message_sender_send_with_correlation() {
        let sender = MockMessageSender::new();
        let target = ComponentId::new("ns", "comp", "inst");
        let payload = MessagePayload::new(vec![1, 2, 3]);
        let correlation_id = "corr-abc-123";

        let result = sender
            .send_with_correlation(&target, payload, correlation_id)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_message_sender_send_with_correlation_failure() {
        let sender = MockMessageSender::failing();
        let target = ComponentId::new("ns", "comp", "inst");
        let payload = MessagePayload::new(vec![1, 2, 3]);
        let correlation_id = "corr-abc-123";

        let result = sender
            .send_with_correlation(&target, payload, correlation_id)
            .await;
        assert!(result.is_err());
        assert!(matches!(result, Err(MessagingError::DeliveryFailed(_))));
    }

    // ---------------------------------------------------------------
    // Trait bounds verification tests
    // ---------------------------------------------------------------

    #[test]
    fn test_fire_and_forget_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<FireAndForget>();
    }

    #[test]
    fn test_message_sender_requires_send_sync() {
        fn assert_send_sync<T: MessageSender + ?Sized>() {}
        assert_send_sync::<MockMessageSender>();
    }

    // ---------------------------------------------------------------
    // Error propagation tests
    // ---------------------------------------------------------------

    #[tokio::test]
    async fn test_fire_and_forget_propagates_exact_error() {
        let router = MockMessageSender::failing();
        let target = ComponentId::new("app", "service", "001");
        let payload = MessagePayload::new(vec![1]);

        let result = FireAndForget::send(&target, payload, &router).await;
        match result {
            Err(MessagingError::TargetNotFound(msg)) => {
                assert!(msg.contains("mock"));
            }
            other => panic!("Expected TargetNotFound, got {:?}", other),
        }
    }
}
