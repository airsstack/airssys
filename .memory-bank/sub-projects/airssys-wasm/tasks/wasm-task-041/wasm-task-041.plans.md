# WASM-TASK-041: Implementation Plans

## References
- ADR-WASM-031 (Component & Messaging Module Design)
- ADR-WASM-026 (Implementation Roadmap)
- ADR-WASM-025 (Clean Slate Architecture)
- ADR-WASM-023 (Module Boundary Enforcement)
- ADR-WASM-009 (Component Communication Model)
- PROJECTS_STANDARD.md (§2.1, §2.2, §4.3)

---

## Actions

### Action 1: Implement Fire-and-Forget Pattern and Traits

**Objective**: Create the fire-and-forget messaging pattern and define the MessageSender and CorrelationManager traits.

**Detailed Steps**:

#### Step 1.1: Create `src/messaging/patterns.rs`

```rust
//! Messaging patterns for component communication.
//!
//! Provides high-level messaging patterns:
//! - FireAndForget: Send message without waiting for response
//! - RequestResponse: Send request and correlate response (WASM-TASK-042)

// Layer 1: Standard library imports
// (none needed)

// Layer 2: Third-party crate imports
// (none needed)

// Layer 3: Internal module imports
use crate::core::component::id::ComponentId;
use crate::core::component::MessagePayload;
use crate::core::errors::messaging::MessagingError;

/// Fire-and-forget messaging pattern
///
/// Sends messages without expecting a response.
/// Best for notifications, events, and one-way commands.
pub struct FireAndForget;

impl FireAndForget {
    /// Send message without expecting response
    ///
    /// # Arguments
    /// * `target` - The component to send to
    /// * `payload` - The message payload
    /// * `router` - The message router implementation
    ///
    /// # Returns
    /// Ok(()) if message was sent successfully
    ///
    /// # Errors
    /// - If target component is not found
    /// - If message delivery fails
    pub async fn send(
        target: &ComponentId,
        payload: MessagePayload,
        router: &impl MessageSender,
    ) -> Result<(), MessagingError> {
        router.send(target, payload).await
    }
}

/// Trait for sending messages
///
/// Abstraction for message routing, enabling dependency injection
/// and testing with mock implementations.
pub trait MessageSender: Send + Sync {
    /// Send a message without correlation
    ///
    /// # Arguments
    /// * `target` - The component to send to
    /// * `payload` - The message payload
    async fn send(
        &self,
        target: &ComponentId,
        payload: MessagePayload,
    ) -> Result<(), MessagingError>;

    /// Send a message with correlation ID for request-response
    ///
    /// # Arguments
    /// * `target` - The component to send to
    /// * `payload` - The message payload
    /// * `correlation_id` - The correlation identifier for tracking response
    async fn send_with_correlation(
        &self,
        target: &ComponentId,
        payload: MessagePayload,
        correlation_id: &str,
    ) -> Result<(), MessagingError>;
}

/// Trait for managing correlations in request-response patterns
///
/// Tracks pending requests and matches responses.
pub trait CorrelationManager: Send + Sync {
    /// Register a new correlation for tracking
    ///
    /// # Arguments
    /// * `id` - The correlation identifier
    /// * `timeout_ms` - Timeout in milliseconds
    async fn register(&self, id: &str, timeout_ms: u64) -> Result<(), MessagingError>;

    /// Complete a correlation with a response
    ///
    /// # Arguments
    /// * `id` - The correlation identifier
    /// * `response` - The response payload
    async fn complete(&self, id: &str, response: MessagePayload) -> Result<(), MessagingError>;

    /// Check if a correlation is still pending
    ///
    /// # Arguments
    /// * `id` - The correlation identifier
    async fn is_pending(&self, id: &str) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock MessageSender for testing
    struct MockMessageSender {
        should_fail: bool,
    }

    impl MockMessageSender {
        fn new() -> Self {
            Self { should_fail: false }
        }

        fn with_failure() -> Self {
            Self { should_fail: true }
        }
    }

    #[async_trait::async_trait]
    impl MessageSender for MockMessageSender {
        async fn send(
            &self,
            _target: &ComponentId,
            _payload: MessagePayload,
        ) -> Result<(), MessagingError> {
            if self.should_fail {
                Err(MessagingError::TargetNotFound("mock error".to_string()))
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
                Err(MessagingError::TargetNotFound("mock error".to_string()))
            } else {
                Ok(())
            }
        }
    }

    #[tokio::test]
    async fn test_fire_and_forget_send_success() {
        let router = MockMessageSender::new();
        let target = ComponentId::new("target-component");
        let payload = MessagePayload::Text("test message".to_string());

        let result = FireAndForget::send(&target, payload, &router).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_fire_and_forget_send_failure() {
        let router = MockMessageSender::with_failure();
        let target = ComponentId::new("target-component");
        let payload = MessagePayload::Text("test message".to_string());

        let result = FireAndForget::send(&target, payload, &router).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_message_sender_trait_send() {
        let router = MockMessageSender::new();
        let target = ComponentId::new("target");
        let payload = MessagePayload::Binary(vec![1, 2, 3]);

        let result = router.send(&target, payload).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_message_sender_trait_send_with_correlation() {
        let router = MockMessageSender::new();
        let target = ComponentId::new("target");
        let payload = MessagePayload::Text("request".to_string());
        let correlation_id = "correlation-123";

        let result = router.send_with_correlation(&target, payload, correlation_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_fire_and_forget_with_text_payload() {
        let router = MockMessageSender::new();
        let target = ComponentId::new("component-a");
        let payload = MessagePayload::Text("Hello, Component A!".to_string());

        let result = FireAndForget::send(&target, payload, &router).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_fire_and_forget_with_binary_payload() {
        let router = MockMessageSender::new();
        let target = ComponentId::new("component-b");
        let payload = MessagePayload::Binary(vec![0xDE, 0xAD, 0xBE, 0xEF]);

        let result = FireAndForget::send(&target, payload, &router).await;
        assert!(result.is_ok());
    }
}
```

#### Step 1.2: Create `src/messaging/mod.rs`

```rust
//! Messaging module - Inter-component communication.
//!
//! Provides messaging patterns, correlation tracking, and routing.

pub mod patterns;

pub use patterns::{CorrelationManager, FireAndForget, MessageSender};
```

**Deliverables**:
- `src/messaging/patterns.rs` with FireAndForget struct
- MessageSender trait (send, send_with_correlation)
- CorrelationManager trait (register, complete, is_pending)
- FireAndForget::send() implementation
- Unit tests with MockMessageSender (6+ tests)
- Module declaration in `src/messaging/mod.rs`

**Constraints**:
- Must not import from `runtime/`, `security/`, or `system/`
- Must follow §2.1 3-Layer imports
- Must follow §2.2 No FQN in type annotations
- Traits must be Send + Sync
- Use async_trait if needed

---

## Verification Section

### Automated Tests
```bash
# Unit tests for patterns module
cargo test -p airssys-wasm --lib -- messaging::patterns

# All messaging module tests
cargo test -p airssys-wasm --lib -- messaging

# Build verification
cargo build -p airssys-wasm

# Clippy
cargo clippy -p airssys-wasm --lib -- -D warnings
```

### Architecture Compliance
```bash
# Verify no forbidden imports in patterns.rs
grep -rn "use crate::runtime" src/messaging/patterns.rs  # Should be empty
grep -rn "use crate::security" src/messaging/patterns.rs  # Should be empty
grep -rn "use crate::system" src/messaging/patterns.rs    # Should be empty

# Verify no FQN in type annotations
grep -rn "::" src/messaging/patterns.rs | grep -v "^.*use " | grep -v "// " | grep "::"  # Should be empty
```

---

## Success Criteria
- [ ] `src/messaging/patterns.rs` exists and compiles
- [ ] FireAndForget struct with send() method
- [ ] MessageSender trait (Send + Sync)
- [ ] CorrelationManager trait (Send + Sync)
- [ ] Unit tests pass (6+ tests)
- [ ] MockMessageSender for testing
- [ ] `cargo clippy -p airssys-wasm --lib -- -D warnings` passes
- [ ] No forbidden imports (architecture compliance)
- [ ] §2.1 3-Layer imports verified
- [ ] §2.2 No FQN verified
