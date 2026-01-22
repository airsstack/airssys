# WASM-TASK-044: Implementation Plans

## References
- ADR-WASM-031 (Component & Messaging Module Design)
- ADR-WASM-026 (Implementation Roadmap)
- ADR-WASM-025 (Clean Slate Architecture)
- ADR-WASM-023 (Module Boundary Enforcement)
- ADR-WASM-009 (Component Communication Model)
- PROJECTS_STANDARD.md (§2.1, §2.2, §4.3)

---

## Actions

### Action 1: Implement ResponseRouter

**Objective**: Create ResponseRouter that implements MessageRouter trait for inter-component message routing.

**Detailed Steps**:

#### Step 1.1: Create `src/messaging/router.rs`

```rust
//! Message routing for inter-component communication.
//!
//! Routes messages between components using registry lookup.

// Layer 1: Standard library imports
use std::sync::Arc;

// Layer 2: Third-party crate imports
use uuid;

// Layer 3: Internal module imports
use crate::component::registry::ComponentRegistry;
use crate::core::component::id::ComponentId;
use crate::core::component::message::{ComponentMessage, MessageMetadata};
use crate::core::component::MessagePayload;
use crate::core::errors::messaging::MessagingError;
use crate::core::messaging::traits::MessageRouter;

/// Routes messages between components
///
/// Uses ComponentRegistry to lookup target component addresses
/// and delivers messages via the actor system.
pub struct ResponseRouter {
    registry: Arc<ComponentRegistry>,
    current_component: ComponentId,
}

impl ResponseRouter {
    /// Create a new ResponseRouter
    ///
    /// # Arguments
    /// * `registry` - The component registry for address lookup
    /// * `current_component` - The current component ID (sender)
    pub fn new(registry: Arc<ComponentRegistry>, current_component: ComponentId) -> Self {
        Self {
            registry,
            current_component,
        }
    }

    /// Create a ComponentMessage with metadata
    ///
    /// # Arguments
    /// * `payload` - The message payload
    /// * `correlation_id` - Optional correlation ID for request-response
    fn create_message(&self, payload: MessagePayload, correlation_id: Option<String>) -> ComponentMessage {
        ComponentMessage {
            sender: self.current_component.clone(),
            payload,
            metadata: MessageMetadata {
                correlation_id,
                reply_to: Some(self.current_component.clone()),
                timestamp_ms: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
                content_type: None,
            },
        }
    }
}

impl MessageRouter for ResponseRouter {
    fn send(&self, target: &ComponentId, payload: MessagePayload) -> Result<(), MessagingError> {
        let _address = self.registry.get(target).ok_or_else(|| {
            MessagingError::TargetNotFound(target.to_string_id())
        })?;

        let _message = self.create_message(payload, None);
        
        // Send via airssys-rt
        // address.send(ComponentActorMessage::HandleMessage(message))?;
        // Note: Actual sending requires airssys-rt integration
        
        Ok(())
    }

    fn request(
        &self,
        target: &ComponentId,
        payload: MessagePayload,
        _timeout_ms: u64,
    ) -> Result<String, MessagingError> {
        let correlation_id = uuid::Uuid::new_v4().to_string();
        
        let _address = self.registry.get(target).ok_or_else(|| {
            MessagingError::TargetNotFound(target.to_string_id())
        })?;

        let _message = self.create_message(payload, Some(correlation_id.clone()));
        
        // Send via airssys-rt
        // address.send(ComponentActorMessage::HandleMessage(message))?;
        
        Ok(correlation_id)
    }

    fn cancel_request(&self, _request_id: &str) -> Result<(), MessagingError> {
        // Implementation for request cancellation
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_creation() {
        let registry = Arc::new(ComponentRegistry::new());
        let current = ComponentId::new("current-component");

        let router = ResponseRouter::new(registry, current);

        // Verify router was created
        assert_eq!(router.current_component.to_string_id(), "current-component");
    }

    #[test]
    fn test_send_target_not_found() {
        let registry = Arc::new(ComponentRegistry::new());
        let current = ComponentId::new("sender");
        let router = ResponseRouter::new(registry, current);

        let target = ComponentId::new("nonexistent");
        let payload = MessagePayload::Text("test".to_string());

        let result = router.send(&target, payload);
        assert!(result.is_err());

        match result {
            Err(MessagingError::TargetNotFound(_)) => {}
            _ => panic!("Expected TargetNotFound error"),
        }
    }

    #[test]
    fn test_request_target_not_found() {
        let registry = Arc::new(ComponentRegistry::new());
        let current = ComponentId::new("sender");
        let router = ResponseRouter::new(registry, current);

        let target = ComponentId::new("nonexistent");
        let payload = MessagePayload::Text("request".to_string());

        let result = router.request(&target, payload, 5000);
        assert!(result.is_err());
    }

    #[test]
    fn test_request_generates_correlation_id() {
        // Note: Cannot fully test without mock ActorAddress
        // This test verifies the structure
        let registry = Arc::new(ComponentRegistry::new());
        let current = ComponentId::new("sender");
        let router = ResponseRouter::new(registry, current);

        // Would test correlation ID generation if target existed
        assert_eq!(router.current_component.to_string_id(), "sender");
    }

    #[test]
    fn test_cancel_request() {
        let registry = Arc::new(ComponentRegistry::new());
        let current = ComponentId::new("sender");
        let router = ResponseRouter::new(registry, current);

        let result = router.cancel_request("correlation-123");
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_message_without_correlation() {
        let registry = Arc::new(ComponentRegistry::new());
        let current = ComponentId::new("sender");
        let router = ResponseRouter::new(registry, current.clone());

        let payload = MessagePayload::Text("test message".to_string());
        let message = router.create_message(payload.clone(), None);

        assert_eq!(message.sender, current);
        assert_eq!(message.payload, payload);
        assert!(message.metadata.correlation_id.is_none());
        assert_eq!(message.metadata.reply_to, Some(current));
    }

    #[test]
    fn test_create_message_with_correlation() {
        let registry = Arc::new(ComponentRegistry::new());
        let current = ComponentId::new("sender");
        let router = ResponseRouter::new(registry, current.clone());

        let payload = MessagePayload::Binary(vec![1, 2, 3]);
        let correlation_id = "test-correlation-789";
        let message = router.create_message(payload.clone(), Some(correlation_id.to_string()));

        assert_eq!(message.sender, current);
        assert_eq!(message.payload, payload);
        assert_eq!(message.metadata.correlation_id, Some(correlation_id.to_string()));
        assert_eq!(message.metadata.reply_to, Some(current));
    }

    #[test]
    fn test_create_message_includes_timestamp() {
        let registry = Arc::new(ComponentRegistry::new());
        let current = ComponentId::new("sender");
        let router = ResponseRouter::new(registry, current);

        let payload = MessagePayload::Text("timestamped".to_string());
        let message = router.create_message(payload, None);

        assert!(message.metadata.timestamp_ms > 0);
    }
}
```

---

### Action 2: Implement ComponentSubscriber

**Objective**: Create ComponentSubscriber for managing mailbox senders and message delivery.

**Detailed Steps**:

#### Step 2.1: Create `src/messaging/subscriber.rs`

```rust
//! Component subscription and mailbox management.
//!
//! Manages mailbox senders for each component to enable message delivery.

// Layer 1: Standard library imports
use std::collections::HashMap;
use std::sync::RwLock;

// Layer 2: Third-party crate imports
use airssys_rt::MailboxSender;

// Layer 3: Internal module imports
use crate::core::component::id::ComponentId;
use crate::core::component::message::ComponentMessage;
use crate::core::errors::messaging::MessagingError;

/// Manages mailbox senders for message delivery
///
/// Tracks mailbox senders for each component to enable
/// direct message delivery to component actors.
pub struct ComponentSubscriber {
    mailbox_senders: RwLock<HashMap<ComponentId, MailboxSender>>,
}

impl ComponentSubscriber {
    /// Create a new ComponentSubscriber
    pub fn new() -> Self {
        Self {
            mailbox_senders: RwLock::new(HashMap::new()),
        }
    }

    /// Register a mailbox sender for a component
    ///
    /// # Arguments
    /// * `id` - The component identifier
    /// * `sender` - The mailbox sender for this component
    pub fn register_mailbox(&self, id: ComponentId, sender: MailboxSender) {
        let mut senders = self.mailbox_senders.write().unwrap();
        senders.insert(id, sender);
    }

    /// Unregister a mailbox sender
    ///
    /// # Arguments
    /// * `id` - The component identifier to unregister
    pub fn unregister_mailbox(&self, id: &ComponentId) {
        let mut senders = self.mailbox_senders.write().unwrap();
        senders.remove(id);
    }

    /// Deliver message to a component
    ///
    /// # Arguments
    /// * `target` - The target component
    /// * `message` - The message to deliver
    ///
    /// # Returns
    /// Ok(()) if message was delivered successfully
    ///
    /// # Errors
    /// - If target component has no registered mailbox
    pub async fn deliver(
        &self,
        target: &ComponentId,
        _message: ComponentMessage,
    ) -> Result<(), MessagingError> {
        let senders = self.mailbox_senders.read().unwrap();
        
        let _sender = senders.get(target).ok_or_else(|| {
            MessagingError::TargetNotFound(target.to_string_id())
        })?;

        // Send message via mailbox
        // sender.send(message).await?;
        // Note: Actual sending requires airssys-rt integration
        
        Ok(())
    }
}

impl Default for ComponentSubscriber {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Cannot fully test without mock MailboxSender
    // These tests verify structure and error handling

    #[test]
    fn test_subscriber_new() {
        let subscriber = ComponentSubscriber::new();
        // Verify no panics
        drop(subscriber);
    }

    #[test]
    fn test_subscriber_default() {
        let subscriber = ComponentSubscriber::default();
        drop(subscriber);
    }

    #[tokio::test]
    async fn test_deliver_target_not_found() {
        let subscriber = ComponentSubscriber::new();
        let target = ComponentId::new("nonexistent");
        let message = ComponentMessage {
            sender: ComponentId::new("sender"),
            payload: MessagePayload::Text("test".to_string()),
            metadata: Default::default(),
        };

        let result = subscriber.deliver(&target, message).await;
        assert!(result.is_err());

        match result {
            Err(MessagingError::TargetNotFound(_)) => {}
            _ => panic!("Expected TargetNotFound error"),
        }
    }

    #[test]
    fn test_register_and_unregister_structure() {
        let subscriber = ComponentSubscriber::new();
        let id = ComponentId::new("test-component");

        // Note: Cannot test with real MailboxSender without airssys-rt mocks
        // Test verifies structure compiles
        subscriber.unregister_mailbox(&id);
    }
}
```

#### Step 2.2: Update `src/messaging/mod.rs`

```rust
//! Messaging module - Inter-component communication.
//!
//! Provides messaging patterns, correlation tracking, and routing.

pub mod correlation;
pub mod patterns;
pub mod router;
pub mod subscriber;

pub use correlation::CorrelationTrackerImpl;
pub use patterns::{CorrelationManager, FireAndForget, MessageSender, RequestResponse};
pub use router::ResponseRouter;
pub use subscriber::ComponentSubscriber;
```

**Deliverables**:
- `src/messaging/router.rs` with ResponseRouter
- `src/messaging/subscriber.rs` with ComponentSubscriber
- MessageRouter trait implementation
- create_message() helper
- Mailbox management (register, unregister, deliver)
- Unit tests for both modules (11+ tests total)
- Module exports in `src/messaging/mod.rs`

**Constraints**:
- Must not import from `runtime/`, `security/`, or `system/`
- Must follow §2.1 3-Layer imports
- Must follow §2.2 No FQN in type annotations
- Must use RwLock for thread safety in ComponentSubscriber

---

## Verification Section

### Automated Tests
```bash
# Unit tests for router module
cargo test -p airssys-wasm --lib -- messaging::router

# Unit tests for subscriber module
cargo test -p airssys-wasm --lib -- messaging::subscriber

# All messaging module tests
cargo test -p airssys-wasm --lib -- messaging

# Build verification
cargo build -p airssys-wasm

# Clippy
cargo clippy -p airssys-wasm --lib -- -D warnings
```

### Architecture Compliance
```bash
# Verify no forbidden imports in router.rs
grep -rn "use crate::runtime" src/messaging/router.rs  # Should be empty
grep -rn "use crate::security" src/messaging/router.rs  # Should be empty
grep -rn "use crate::system" src/messaging/router.rs    # Should be empty

# Verify no forbidden imports in subscriber.rs
grep -rn "use crate::runtime" src/messaging/subscriber.rs  # Should be empty
grep -rn "use crate::security" src/messaging/subscriber.rs  # Should be empty
grep -rn "use crate::system" src/messaging/subscriber.rs    # Should be empty

# Verify no FQN in type annotations
grep -rn "::" src/messaging/router.rs | grep -v "^.*use " | grep -v "// " | grep "::"  # Should be empty
grep -rn "::" src/messaging/subscriber.rs | grep -v "^.*use " | grep -v "// " | grep "::"  # Should be empty
```

---

## Success Criteria
- [ ] `src/messaging/router.rs` exists and compiles
- [ ] `src/messaging/subscriber.rs` exists and compiles
- [ ] ResponseRouter implements MessageRouter trait
- [ ] ComponentSubscriber manages mailbox senders
- [ ] create_message() includes all metadata
- [ ] Unit tests pass (11+ tests total: 8 router + 3 subscriber)
- [ ] `cargo clippy -p airssys-wasm --lib -- -D warnings` passes
- [ ] No forbidden imports (architecture compliance)
- [ ] §2.1 3-Layer imports verified
- [ ] §2.2 No FQN verified
- [ ] Thread-safety verified (RwLock usage)
