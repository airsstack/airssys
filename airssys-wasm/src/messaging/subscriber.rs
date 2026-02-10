//! Component subscription and mailbox management.
//!
//! Provides [`ComponentSubscriber`] for managing message delivery channels
//! to component actors. Each component registers a delivery function that
//! enables the messaging layer to push messages to the component's mailbox.
//!
//! # Architecture
//!
//! This module is part of `messaging/` (Layer 3B). It depends on:
//! - `core/component/` for `ComponentId`, `ComponentMessage`
//! - `core/messaging/` for `MessagingError`
//!
//! This module does NOT import from `component/` (Layer 3A), `runtime/`,
//! `security/`, or `system/`.
//!
//! The `system/` module (Layer 4) is responsible for creating the concrete
//! delivery functions that bridge to `airssys-rt` mailbox senders.
//!
//! # References
//!
//! - ADR-WASM-031: Component & Messaging Module Design
//! - ADR-WASM-009: Component Communication Model (push-based delivery)

// Layer 1: Standard library imports
use std::collections::HashMap;
use std::fmt;
use std::sync::RwLock;

// Layer 2: Third-party crate imports
// (none)

// Layer 3: Internal module imports
use crate::core::component::id::ComponentId;
use crate::core::component::message::ComponentMessage;
use crate::core::messaging::errors::MessagingError;

/// Type alias for the delivery function used to send messages to a component.
///
/// The delivery function is injected by the `system/` module (Layer 4) and
/// bridges from the messaging layer to the airssys-rt actor mailbox.
type DeliveryFn = Box<dyn Fn(ComponentMessage) -> Result<(), MessagingError> + Send + Sync>;

/// Manages mailbox registrations for push-based message delivery to components.
///
/// Each component registers a delivery function that enables the messaging
/// layer to push messages directly to the component's actor mailbox.
///
/// # Thread Safety
///
/// Uses `RwLock<HashMap>` for interior mutability:
/// - Multiple concurrent reads (deliver lookups)
/// - Exclusive access for writes (register/unregister)
///
/// # Error Handling
///
/// All methods that access the RwLock return `Result` to handle potential
/// lock poisoning, following workspace policy of denying `unwrap_used`.
///
/// # Examples
///
/// ```rust,ignore
/// use airssys_wasm::messaging::subscriber::ComponentSubscriber;
/// use airssys_wasm::core::component::id::ComponentId;
///
/// let subscriber = ComponentSubscriber::new();
/// let id = ComponentId::new("app", "service", "v1");
///
/// // Register a mock delivery function
/// subscriber.register_mailbox(id.clone(), Box::new(|_msg| Ok(()))).unwrap();
///
/// // Check registration
/// assert!(subscriber.is_registered(&id).unwrap());
///
/// // Unregister
/// subscriber.unregister_mailbox(&id).unwrap();
/// assert!(!subscriber.is_registered(&id).unwrap());
/// ```
pub struct ComponentSubscriber {
    /// Maps component IDs to their delivery functions
    mailboxes: RwLock<HashMap<ComponentId, DeliveryFn>>,
}

impl ComponentSubscriber {
    /// Creates a new empty `ComponentSubscriber`.
    pub fn new() -> Self {
        Self {
            mailboxes: RwLock::new(HashMap::new()),
        }
    }

    /// Registers a delivery function for a component.
    ///
    /// If the component ID already has a registered delivery function,
    /// it is replaced with the new one.
    ///
    /// # Arguments
    ///
    /// * `id` - The component identifier
    /// * `delivery_fn` - Function to deliver messages to this component
    ///
    /// # Errors
    ///
    /// Returns `MessagingError::DeliveryFailed` if the lock is poisoned.
    pub fn register_mailbox(
        &self,
        id: ComponentId,
        delivery_fn: DeliveryFn,
    ) -> Result<(), MessagingError> {
        let mut mailboxes = self
            .mailboxes
            .write()
            .map_err(|e| MessagingError::DeliveryFailed(format!("Lock poisoned: {}", e)))?;
        mailboxes.insert(id, delivery_fn);
        Ok(())
    }

    /// Unregisters a component's delivery function.
    ///
    /// # Arguments
    ///
    /// * `id` - The component identifier to unregister
    ///
    /// # Returns
    ///
    /// `true` if the component was registered and removed, `false` if not found.
    ///
    /// # Errors
    ///
    /// Returns `MessagingError::DeliveryFailed` if the lock is poisoned.
    pub fn unregister_mailbox(&self, id: &ComponentId) -> Result<bool, MessagingError> {
        let mut mailboxes = self
            .mailboxes
            .write()
            .map_err(|e| MessagingError::DeliveryFailed(format!("Lock poisoned: {}", e)))?;
        Ok(mailboxes.remove(id).is_some())
    }

    /// Checks if a component has a registered mailbox.
    ///
    /// # Arguments
    ///
    /// * `id` - The component identifier to check
    ///
    /// # Errors
    ///
    /// Returns `MessagingError::DeliveryFailed` if the lock is poisoned.
    pub fn is_registered(&self, id: &ComponentId) -> Result<bool, MessagingError> {
        let mailboxes = self
            .mailboxes
            .read()
            .map_err(|e| MessagingError::DeliveryFailed(format!("Lock poisoned: {}", e)))?;
        Ok(mailboxes.contains_key(id))
    }

    /// Returns the number of registered mailboxes.
    ///
    /// # Errors
    ///
    /// Returns `MessagingError::DeliveryFailed` if the lock is poisoned.
    pub fn count(&self) -> Result<usize, MessagingError> {
        let mailboxes = self
            .mailboxes
            .read()
            .map_err(|e| MessagingError::DeliveryFailed(format!("Lock poisoned: {}", e)))?;
        Ok(mailboxes.len())
    }

    /// Delivers a message to a target component.
    ///
    /// Looks up the target's delivery function and invokes it with the message.
    ///
    /// # Arguments
    ///
    /// * `target` - The target component identifier
    /// * `message` - The message to deliver
    ///
    /// # Errors
    ///
    /// - `MessagingError::TargetNotFound` if the target has no registered mailbox
    /// - `MessagingError::DeliveryFailed` if the delivery function returns an error
    /// - `MessagingError::DeliveryFailed` if the lock is poisoned
    pub fn deliver(
        &self,
        target: &ComponentId,
        message: ComponentMessage,
    ) -> Result<(), MessagingError> {
        let mailboxes = self
            .mailboxes
            .read()
            .map_err(|e| MessagingError::DeliveryFailed(format!("Lock poisoned: {}", e)))?;

        let delivery_fn = mailboxes
            .get(target)
            .ok_or_else(|| MessagingError::TargetNotFound(target.to_string_id()))?;

        delivery_fn(message)
    }
}

impl Default for ComponentSubscriber {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for ComponentSubscriber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.mailboxes.read() {
            Ok(mailboxes) => f
                .debug_struct("ComponentSubscriber")
                .field("registered_count", &mailboxes.len())
                .finish(),
            Err(_) => f
                .debug_struct("ComponentSubscriber")
                .field("status", &"<lock poisoned>")
                .finish(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::component::message::MessageMetadata;
    use crate::core::component::message::MessagePayload;

    // ---------------------------------------------------------------
    // Helper functions
    // ---------------------------------------------------------------

    fn make_test_message(sender_name: &str) -> ComponentMessage {
        ComponentMessage::new(
            ComponentId::new("test", sender_name, "v1"),
            MessagePayload::new(vec![1, 2, 3]),
            MessageMetadata::default(),
        )
    }

    fn make_ok_delivery() -> DeliveryFn {
        Box::new(|_msg| Ok(()))
    }

    fn make_failing_delivery() -> DeliveryFn {
        Box::new(|_msg| {
            Err(MessagingError::DeliveryFailed(
                "mock delivery failed".to_string(),
            ))
        })
    }

    // ---------------------------------------------------------------
    // Constructor tests
    // ---------------------------------------------------------------

    #[test]
    fn test_subscriber_new() {
        let subscriber = ComponentSubscriber::new();
        assert_eq!(subscriber.count().unwrap(), 0);
    }

    #[test]
    fn test_subscriber_default() {
        let subscriber = ComponentSubscriber::default();
        assert_eq!(subscriber.count().unwrap(), 0);
    }

    // ---------------------------------------------------------------
    // Registration tests
    // ---------------------------------------------------------------

    #[test]
    fn test_register_mailbox() {
        let subscriber = ComponentSubscriber::new();
        let id = ComponentId::new("app", "service", "v1");

        let result = subscriber.register_mailbox(id.clone(), make_ok_delivery());
        assert!(result.is_ok());
        assert!(subscriber.is_registered(&id).unwrap());
        assert_eq!(subscriber.count().unwrap(), 1);
    }

    #[test]
    fn test_register_multiple_mailboxes() {
        let subscriber = ComponentSubscriber::new();
        let id1 = ComponentId::new("app", "service1", "v1");
        let id2 = ComponentId::new("app", "service2", "v1");

        subscriber
            .register_mailbox(id1.clone(), make_ok_delivery())
            .unwrap();
        subscriber
            .register_mailbox(id2.clone(), make_ok_delivery())
            .unwrap();

        assert!(subscriber.is_registered(&id1).unwrap());
        assert!(subscriber.is_registered(&id2).unwrap());
        assert_eq!(subscriber.count().unwrap(), 2);
    }

    #[test]
    fn test_register_overwrites_existing() {
        let subscriber = ComponentSubscriber::new();
        let id = ComponentId::new("app", "service", "v1");

        subscriber
            .register_mailbox(id.clone(), make_failing_delivery())
            .unwrap();
        subscriber
            .register_mailbox(id.clone(), make_ok_delivery())
            .unwrap();

        // Should have replaced the failing delivery with OK delivery
        assert_eq!(subscriber.count().unwrap(), 1);
        let msg = make_test_message("sender");
        assert!(subscriber.deliver(&id, msg).is_ok());
    }

    // ---------------------------------------------------------------
    // Unregistration tests
    // ---------------------------------------------------------------

    #[test]
    fn test_unregister_existing_mailbox() {
        let subscriber = ComponentSubscriber::new();
        let id = ComponentId::new("app", "service", "v1");

        subscriber
            .register_mailbox(id.clone(), make_ok_delivery())
            .unwrap();
        assert!(subscriber.is_registered(&id).unwrap());

        let removed = subscriber.unregister_mailbox(&id).unwrap();
        assert!(removed);
        assert!(!subscriber.is_registered(&id).unwrap());
        assert_eq!(subscriber.count().unwrap(), 0);
    }

    #[test]
    fn test_unregister_nonexistent_returns_false() {
        let subscriber = ComponentSubscriber::new();
        let id = ComponentId::new("app", "nonexistent", "v1");

        let removed = subscriber.unregister_mailbox(&id).unwrap();
        assert!(!removed);
    }

    // ---------------------------------------------------------------
    // Delivery tests
    // ---------------------------------------------------------------

    #[test]
    fn test_deliver_success() {
        let subscriber = ComponentSubscriber::new();
        let id = ComponentId::new("app", "target", "v1");

        subscriber
            .register_mailbox(id.clone(), make_ok_delivery())
            .unwrap();

        let msg = make_test_message("sender");
        let result = subscriber.deliver(&id, msg);
        assert!(result.is_ok());
    }

    #[test]
    fn test_deliver_target_not_found() {
        let subscriber = ComponentSubscriber::new();
        let target = ComponentId::new("app", "nonexistent", "v1");
        let msg = make_test_message("sender");

        let result = subscriber.deliver(&target, msg);
        assert!(result.is_err());
        assert!(matches!(result, Err(MessagingError::TargetNotFound(_))));
    }

    #[test]
    fn test_deliver_with_failing_delivery_fn() {
        let subscriber = ComponentSubscriber::new();
        let id = ComponentId::new("app", "failing", "v1");

        subscriber
            .register_mailbox(id.clone(), make_failing_delivery())
            .unwrap();

        let msg = make_test_message("sender");
        let result = subscriber.deliver(&id, msg);
        assert!(result.is_err());
        assert!(matches!(result, Err(MessagingError::DeliveryFailed(_))));
    }

    #[test]
    fn test_deliver_after_unregister_fails() {
        let subscriber = ComponentSubscriber::new();
        let id = ComponentId::new("app", "target", "v1");

        subscriber
            .register_mailbox(id.clone(), make_ok_delivery())
            .unwrap();
        subscriber.unregister_mailbox(&id).unwrap();

        let msg = make_test_message("sender");
        let result = subscriber.deliver(&id, msg);
        assert!(result.is_err());
        assert!(matches!(result, Err(MessagingError::TargetNotFound(_))));
    }

    // ---------------------------------------------------------------
    // Delivery function captures message tests
    // ---------------------------------------------------------------

    #[test]
    fn test_delivery_fn_receives_correct_message() {
        use std::sync::atomic::AtomicBool;
        use std::sync::atomic::Ordering;
        use std::sync::Arc;

        let was_called = Arc::new(AtomicBool::new(false));
        let was_called_clone = Arc::clone(&was_called);

        let delivery: DeliveryFn = Box::new(move |msg| {
            // Verify the message content
            assert_eq!(msg.sender.to_string_id(), "test/sender/v1");
            assert_eq!(msg.payload.as_bytes(), &[1, 2, 3]);
            was_called_clone.store(true, Ordering::SeqCst);
            Ok(())
        });

        let subscriber = ComponentSubscriber::new();
        let id = ComponentId::new("app", "target", "v1");
        subscriber.register_mailbox(id.clone(), delivery).unwrap();

        let msg = make_test_message("sender");
        subscriber.deliver(&id, msg).unwrap();

        assert!(was_called.load(Ordering::SeqCst));
    }

    // ---------------------------------------------------------------
    // Debug and trait tests
    // ---------------------------------------------------------------

    #[test]
    fn test_debug_format() {
        let subscriber = ComponentSubscriber::new();
        let debug_str = format!("{:?}", subscriber);
        assert!(debug_str.contains("ComponentSubscriber"));
        assert!(debug_str.contains("registered_count"));
        assert!(debug_str.contains("0"));
    }

    #[test]
    fn test_subscriber_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ComponentSubscriber>();
    }
}
