//! Message routing for inter-component communication.
//!
//! Provides [`ResponseRouter`], the concrete implementation of
//! [`MessageRouter`](crate::core::messaging::traits::MessageRouter)
//! for routing messages between WASM component actors.
//!
//! # Architecture
//!
//! This module is part of `messaging/` (Layer 3B). It depends on:
//! - `core/component/` for `ComponentId`, `ComponentMessage`, `MessageMetadata`, `MessagePayload`
//! - `core/component/traits` for `ComponentResolver` (abstraction for component lookup)
//! - `core/messaging/` for `MessagingError`, `CorrelationId`, `MessageRouter` trait
//!
//! **IMPORTANT:** This module does NOT import from `component/` (Layer 3A).
//! It uses the `ComponentResolver` trait from `core/` instead of the concrete
//! `ComponentRegistry`. The concrete registry is injected by `system/` (Layer 4).
//!
//! # References
//!
//! - ADR-WASM-031: Component & Messaging Module Design
//! - ADR-WASM-009: Component Communication Model
//! - KNOWLEDGE-WASM-037: Dependency Inversion Principle

// Layer 1: Standard library imports
use std::sync::Arc;

// Layer 2: Third-party crate imports
use chrono::Utc;

// Layer 3: Internal module imports
use crate::core::component::id::ComponentId;
use crate::core::component::message::ComponentMessage;
use crate::core::component::message::MessageMetadata;
use crate::core::component::message::MessagePayload;
use crate::core::component::traits::ComponentResolver;
use crate::core::messaging::correlation::CorrelationId;
use crate::core::messaging::errors::MessagingError;
use crate::core::messaging::traits::MessageRouter;

/// Routes messages between WASM components via component resolver lookup.
///
/// `ResponseRouter` uses a [`ComponentResolver`] trait to verify target
/// component existence and creates properly-formed [`ComponentMessage`]
/// envelopes with metadata (sender, correlation ID, timestamp, reply-to).
///
/// # Architecture
///
/// ResponseRouter implements the [`MessageRouter`] trait defined in
/// `core/messaging/traits`. It depends on the [`ComponentResolver`] trait
/// (from `core/component/traits`), NOT the concrete `ComponentRegistry`.
/// The concrete resolver is injected by `system/` (Layer 4).
///
/// This follows the Dependency Inversion Principle per KNOWLEDGE-WASM-037:
/// messaging/ depends on abstractions (core/ traits), not concrete types
/// (component/ structs).
///
/// Generic parameter `R` allows any type implementing `ComponentResolver`,
/// providing static dispatch and compile-time type safety (per S6.2).
/// This is consistent with `OslSecurityBridge<P: SecurityPolicy>` and
/// `ComponentSpawner<E: RuntimeEngine, L: ComponentLoader>`.
///
/// Actual message delivery to actor mailboxes is handled by the `system/`
/// module (Layer 4).
///
/// # Thread Safety
///
/// `ResponseRouter` is `Send + Sync` because it holds `Arc<R>` where
/// `R: ComponentResolver` (which requires `Send + Sync`) and a `ComponentId`
/// (immutable after creation).
///
/// # Examples
///
/// ```rust,ignore
/// use std::sync::Arc;
/// use airssys_wasm::messaging::router::ResponseRouter;
/// use airssys_wasm::core::component::traits::ComponentResolver;
/// use airssys_wasm::core::component::id::ComponentId;
/// use airssys_wasm::core::messaging::traits::MessageRouter;
///
/// // In tests, use a mock resolver:
/// let resolver = Arc::new(MockResolver::new());
/// let sender_id = ComponentId::new("app", "sender", "v1");
/// let router = ResponseRouter::new(resolver, sender_id);
///
/// // In production (system/ layer), inject the real ComponentRegistry:
/// // let registry = Arc::new(ComponentRegistry::new());
/// // let router = ResponseRouter::new(registry, sender_id);
/// ```
pub struct ResponseRouter<R: ComponentResolver> {
    /// Component resolver for target existence verification (static dispatch per S6.2)
    resolver: Arc<R>,
    /// The component ID of the current sender
    current_component: ComponentId,
}

impl<R: ComponentResolver> ResponseRouter<R> {
    /// Creates a new `ResponseRouter`.
    ///
    /// # Arguments
    ///
    /// * `resolver` - Shared component resolver for target existence checks.
    ///   In production, this is a `ComponentRegistry`. In tests, a mock.
    /// * `current_component` - The ComponentId of the sending component
    pub fn new(resolver: Arc<R>, current_component: ComponentId) -> Self {
        Self {
            resolver,
            current_component,
        }
    }

    /// Returns a reference to the current component ID.
    pub fn current_component(&self) -> &ComponentId {
        &self.current_component
    }

    /// Creates a [`ComponentMessage`] with full metadata.
    ///
    /// Populates the message envelope with sender, payload, and metadata
    /// including optional correlation ID, reply-to address, and timestamp.
    ///
    /// # Arguments
    ///
    /// * `payload` - The message payload bytes
    /// * `correlation_id` - Optional correlation ID for request-response patterns
    ///
    /// # Returns
    ///
    /// A fully-formed `ComponentMessage` ready for delivery.
    fn create_message(
        &self,
        payload: MessagePayload,
        correlation_id: Option<String>,
    ) -> ComponentMessage {
        let timestamp_ms = Utc::now().timestamp_millis() as u64;

        ComponentMessage::new(
            self.current_component.clone(),
            payload,
            MessageMetadata {
                correlation_id,
                reply_to: Some(self.current_component.clone()),
                timestamp_ms,
                content_type: None,
            },
        )
    }
}

impl<R: ComponentResolver> MessageRouter for ResponseRouter<R> {
    fn send(&self, target: &ComponentId, payload: MessagePayload) -> Result<(), MessagingError> {
        // Verify target exists via the resolver trait
        let exists = self
            .resolver
            .contains(target)
            .map_err(|e| MessagingError::DeliveryFailed(e.to_string()))?;

        if !exists {
            return Err(MessagingError::TargetNotFound(target.to_string_id()));
        }

        // Create the message envelope
        let _message = self.create_message(payload, None);

        // NOTE: Actual delivery to actor mailbox will be wired up by system/ (Layer 4).
        // The resolver lookup validates the target exists. The message is created and
        // ready for delivery. The system/ module will connect this to airssys-rt.

        Ok(())
    }

    fn request(
        &self,
        target: &ComponentId,
        payload: MessagePayload,
        _timeout_ms: u64,
    ) -> Result<CorrelationId, MessagingError> {
        // Generate unique correlation ID
        let correlation_id = CorrelationId::generate();

        // Verify target exists via the resolver trait
        let exists = self
            .resolver
            .contains(target)
            .map_err(|e| MessagingError::DeliveryFailed(e.to_string()))?;

        if !exists {
            return Err(MessagingError::TargetNotFound(target.to_string_id()));
        }

        // Create the message envelope with correlation ID
        let _message = self.create_message(payload, Some(correlation_id.as_str().to_owned()));

        // NOTE: Actual delivery and timeout tracking wired up by system/ (Layer 4).

        Ok(correlation_id)
    }

    fn cancel_request(&self, _correlation_id: &CorrelationId) -> Result<(), MessagingError> {
        // NOTE: Cancel logic will be wired to CorrelationTrackerImpl by system/ (Layer 4).
        // For now, this is a no-op that succeeds.
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::component::errors::ComponentError;

    // ---------------------------------------------------------------
    // Mock ComponentResolver for testing (no dependency on component/)
    // ---------------------------------------------------------------

    struct MockResolver {
        registered: Vec<ComponentId>,
    }

    impl MockResolver {
        fn empty() -> Self {
            Self {
                registered: Vec::new(),
            }
        }

        fn with_component(id: ComponentId) -> Self {
            Self {
                registered: vec![id],
            }
        }
    }

    impl ComponentResolver for MockResolver {
        fn contains(&self, id: &ComponentId) -> Result<bool, ComponentError> {
            Ok(self.registered.iter().any(|r| r == id))
        }
    }

    // ---------------------------------------------------------------
    // Helper functions
    // ---------------------------------------------------------------

    fn create_router() -> ResponseRouter<MockResolver> {
        let resolver = Arc::new(MockResolver::empty());
        let current = ComponentId::new("app", "sender", "v1");
        ResponseRouter::new(resolver, current)
    }

    fn create_router_with_target() -> (ResponseRouter<MockResolver>, ComponentId) {
        let target = ComponentId::new("app", "receiver", "v1");
        let resolver = Arc::new(MockResolver::with_component(target.clone()));

        let current = ComponentId::new("app", "sender", "v1");
        let router = ResponseRouter::new(resolver, current);
        (router, target)
    }

    // ---------------------------------------------------------------
    // Constructor tests
    // ---------------------------------------------------------------

    #[test]
    fn test_router_creation() {
        let resolver = Arc::new(MockResolver::empty());
        let current = ComponentId::new("app", "sender", "v1");

        let router = ResponseRouter::new(resolver, current.clone());

        assert_eq!(router.current_component(), &current);
    }

    #[test]
    fn test_router_current_component_accessor() {
        let router = create_router();
        assert_eq!(router.current_component().to_string_id(), "app/sender/v1");
    }

    // ---------------------------------------------------------------
    // MessageRouter::send tests
    // ---------------------------------------------------------------

    #[test]
    fn test_send_target_not_found() {
        let router = create_router();
        let target = ComponentId::new("app", "nonexistent", "v1");
        let payload = MessagePayload::new(vec![1, 2, 3]);

        let result = router.send(&target, payload);
        assert!(result.is_err());
        assert!(matches!(result, Err(MessagingError::TargetNotFound(_))));
    }

    #[test]
    fn test_send_success_with_registered_target() {
        let (router, target) = create_router_with_target();
        let payload = MessagePayload::new(vec![1, 2, 3]);

        let result = router.send(&target, payload);
        assert!(result.is_ok());
    }

    #[test]
    fn test_send_with_empty_payload() {
        let (router, target) = create_router_with_target();
        let payload = MessagePayload::new(vec![]);

        let result = router.send(&target, payload);
        assert!(result.is_ok());
    }

    // ---------------------------------------------------------------
    // MessageRouter::request tests
    // ---------------------------------------------------------------

    #[test]
    fn test_request_target_not_found() {
        let router = create_router();
        let target = ComponentId::new("app", "nonexistent", "v1");
        let payload = MessagePayload::new(vec![1, 2, 3]);

        let result = router.request(&target, payload, 5000);
        assert!(result.is_err());
        assert!(matches!(result, Err(MessagingError::TargetNotFound(_))));
    }

    #[test]
    fn test_request_success_returns_correlation_id() {
        let (router, target) = create_router_with_target();
        let payload = MessagePayload::new(vec![1, 2, 3]);

        let result = router.request(&target, payload, 5000);
        assert!(result.is_ok());

        let correlation_id = result.unwrap();
        // CorrelationId::generate() produces UUID v4: 36 chars
        assert_eq!(correlation_id.as_str().len(), 36);
        assert!(correlation_id.as_str().contains('-'));
    }

    #[test]
    fn test_request_generates_unique_ids() {
        let (router, target) = create_router_with_target();

        let payload1 = MessagePayload::new(vec![1]);
        let id1 = router.request(&target, payload1, 5000).unwrap();

        let payload2 = MessagePayload::new(vec![2]);
        let id2 = router.request(&target, payload2, 5000).unwrap();

        assert_ne!(id1, id2);
    }

    // ---------------------------------------------------------------
    // MessageRouter::cancel_request tests
    // ---------------------------------------------------------------

    #[test]
    fn test_cancel_request_succeeds() {
        let router = create_router();
        let correlation_id = CorrelationId::new("test-correlation-123");

        let result = router.cancel_request(&correlation_id);
        assert!(result.is_ok());
    }

    // ---------------------------------------------------------------
    // create_message tests
    // ---------------------------------------------------------------

    #[test]
    fn test_create_message_without_correlation() {
        let router = create_router();
        let payload = MessagePayload::new(vec![10, 20, 30]);

        let message = router.create_message(payload.clone(), None);

        assert_eq!(message.sender.to_string_id(), "app/sender/v1");
        assert_eq!(message.payload, payload);
        assert!(message.metadata.correlation_id.is_none());
        assert_eq!(
            message
                .metadata
                .reply_to
                .as_ref()
                .map(|id| id.to_string_id()),
            Some("app/sender/v1".to_string())
        );
    }

    #[test]
    fn test_create_message_with_correlation() {
        let router = create_router();
        let payload = MessagePayload::new(vec![1, 2, 3]);
        let correlation = "test-correlation-789".to_string();

        let message = router.create_message(payload.clone(), Some(correlation.clone()));

        assert_eq!(message.sender.to_string_id(), "app/sender/v1");
        assert_eq!(message.payload, payload);
        assert_eq!(message.metadata.correlation_id, Some(correlation));
        assert!(message.metadata.reply_to.is_some());
    }

    #[test]
    fn test_create_message_includes_timestamp() {
        let router = create_router();
        let payload = MessagePayload::new(vec![42]);

        let message = router.create_message(payload, None);

        // Timestamp should be a recent value (not zero, and reasonable epoch millis)
        assert!(message.metadata.timestamp_ms > 0);
        // Should be a plausible timestamp (after year 2020 in millis)
        assert!(message.metadata.timestamp_ms > 1_577_836_800_000);
    }

    // ---------------------------------------------------------------
    // Thread safety tests
    // ---------------------------------------------------------------

    #[test]
    fn test_response_router_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ResponseRouter<MockResolver>>();
    }
}
