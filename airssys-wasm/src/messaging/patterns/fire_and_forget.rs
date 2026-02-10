//! Fire-and-forget messaging pattern.
//!
//! Provides the [`FireAndForget`] pattern for sending messages
//! without expecting a response.
//!
//! # Architecture
//!
//! This module is part of `messaging/patterns/` (Layer 3B). It depends on:
//! - `core/component/` for `ComponentId` and `MessagePayload`
//! - `core/messaging/` for `MessagingError` and `MessageSender` trait
//!
//! # References
//!
//! - ADR-WASM-031: Component & Messaging Module Design
//! - ADR-WASM-009: Component Communication Model (Pattern 1: Fire-and-Forget)

// Layer 1: Standard library imports
// (none needed)

// Layer 2: Third-party crate imports
// (none needed)

// Layer 3: Internal module imports
use crate::core::component::id::ComponentId;
use crate::core::component::message::MessagePayload;
use crate::core::messaging::errors::MessagingError;
use crate::core::messaging::traits::MessageSender;

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
/// use airssys_wasm::messaging::patterns::fire_and_forget::FireAndForget;
/// use airssys_wasm::core::messaging::traits::MessageSender;
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
