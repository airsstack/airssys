# WASM-TASK-042: Implementation Plans

## References
- ADR-WASM-031 (Component & Messaging Module Design)
- ADR-WASM-026 (Implementation Roadmap)
- ADR-WASM-025 (Clean Slate Architecture)
- ADR-WASM-023 (Module Boundary Enforcement)
- ADR-WASM-009 (Component Communication Model)
- PROJECTS_STANDARD.md (§2.1, §2.2, §4.3)

---

## Actions

### Action 1: Implement Request-Response Pattern

**Objective**: Extend messaging/patterns.rs with RequestResponse pattern for correlated messaging.

**Detailed Steps**:

#### Step 1.1: Extend `src/messaging/patterns.rs`

```rust
// Add to existing patterns.rs file:

/// Request-response messaging pattern
///
/// Sends a request and tracks the correlation for response matching.
/// Best for RPC-style interactions and query operations.
pub struct RequestResponse;

impl RequestResponse {
    /// Send request and await response via callback
    ///
    /// # Arguments
    /// * `target` - The component to send request to
    /// * `payload` - The request payload
    /// * `timeout_ms` - Timeout in milliseconds
    /// * `router` - The message router implementation
    /// * `tracker` - The correlation tracker implementation
    ///
    /// # Returns
    /// Ok(correlation_id) for tracking the response
    ///
    /// # Errors
    /// - If target component is not found
    /// - If correlation registration fails
    /// - If message delivery fails
    ///
    /// # Example
    /// ```ignore
    /// use airssys_wasm::messaging::{RequestResponse, MessageSender, CorrelationManager};
    /// use airssys_wasm::core::component::MessagePayload;
    ///
    /// async fn make_request(
    ///     target: &ComponentId,
    ///     router: &impl MessageSender,
    ///     tracker: &impl CorrelationManager,
    /// ) -> Result<String, MessagingError> {
    ///     let payload = MessagePayload::Text("query".to_string());
    ///     let correlation_id = RequestResponse::request(
    ///         target,
    ///         payload,
    ///         5000, // 5 second timeout
    ///         router,
    ///         tracker,
    ///     ).await?;
    ///     Ok(correlation_id)
    /// }
    /// ```
    pub async fn request(
        target: &ComponentId,
        payload: MessagePayload,
        timeout_ms: u64,
        router: &impl MessageSender,
        tracker: &impl CorrelationManager,
    ) -> Result<String, MessagingError> {
        // Generate correlation ID
        let correlation_id = uuid::Uuid::new_v4().to_string();
        
        // Register correlation
        tracker.register(&correlation_id, timeout_ms).await?;
        
        // Send request with correlation
        router.send_with_correlation(target, payload, &correlation_id).await?;
        
        Ok(correlation_id)
    }
}

// Add to imports at top of file:
// Layer 2: Third-party crate imports
use uuid;

#[cfg(test)]
mod tests {
    use super::*;

    // ... existing tests ...

    // Mock CorrelationManager for testing
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

    #[async_trait::async_trait]
    impl CorrelationManager for MockCorrelationManager {
        async fn register(&self, _id: &str, _timeout_ms: u64) -> Result<(), MessagingError> {
            if self.should_fail_register {
                Err(MessagingError::InvalidMessage("registration failed".to_string()))
            } else {
                Ok(())
            }
        }

        async fn complete(&self, _id: &str, _response: MessagePayload) -> Result<(), MessagingError> {
            Ok(())
        }

        async fn is_pending(&self, _id: &str) -> bool {
            true
        }
    }

    #[tokio::test]
    async fn test_request_response_success() {
        let router = MockMessageSender::new();
        let tracker = MockCorrelationManager::new();
        let target = ComponentId::new("target-component");
        let payload = MessagePayload::Text("request".to_string());

        let result = RequestResponse::request(&target, payload, 5000, &router, &tracker).await;
        assert!(result.is_ok());
        
        let correlation_id = result.unwrap();
        assert!(!correlation_id.is_empty());
    }

    #[tokio::test]
    async fn test_request_response_generates_unique_ids() {
        let router = MockMessageSender::new();
        let tracker = MockCorrelationManager::new();
        let target = ComponentId::new("target");
        let payload1 = MessagePayload::Text("request1".to_string());
        let payload2 = MessagePayload::Text("request2".to_string());

        let id1 = RequestResponse::request(&target, payload1, 5000, &router, &tracker).await.unwrap();
        let id2 = RequestResponse::request(&target, payload2, 5000, &router, &tracker).await.unwrap();

        assert_ne!(id1, id2);
    }

    #[tokio::test]
    async fn test_request_response_correlation_registration_fails() {
        let router = MockMessageSender::new();
        let tracker = MockCorrelationManager::with_register_failure();
        let target = ComponentId::new("target");
        let payload = MessagePayload::Text("request".to_string());

        let result = RequestResponse::request(&target, payload, 5000, &router, &tracker).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_request_response_send_fails() {
        let router = MockMessageSender::with_failure();
        let tracker = MockCorrelationManager::new();
        let target = ComponentId::new("target");
        let payload = MessagePayload::Text("request".to_string());

        let result = RequestResponse::request(&target, payload, 5000, &router, &tracker).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_request_response_with_timeout() {
        let router = MockMessageSender::new();
        let tracker = MockCorrelationManager::new();
        let target = ComponentId::new("target");
        let payload = MessagePayload::Text("request".to_string());
        let timeout_ms = 10_000; // 10 seconds

        let result = RequestResponse::request(&target, payload, timeout_ms, &router, &tracker).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_request_response_with_binary_payload() {
        let router = MockMessageSender::new();
        let tracker = MockCorrelationManager::new();
        let target = ComponentId::new("target");
        let payload = MessagePayload::Binary(vec![1, 2, 3, 4, 5]);

        let result = RequestResponse::request(&target, payload, 5000, &router, &tracker).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_correlation_manager_trait_register() {
        let tracker = MockCorrelationManager::new();
        let result = tracker.register("test-correlation", 5000).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_correlation_manager_trait_complete() {
        let tracker = MockCorrelationManager::new();
        let payload = MessagePayload::Text("response".to_string());
        let result = tracker.complete("test-correlation", payload).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_correlation_manager_trait_is_pending() {
        let tracker = MockCorrelationManager::new();
        let is_pending = tracker.is_pending("test-correlation").await;
        assert!(is_pending);
    }
}
```

#### Step 1.2: Update `src/messaging/mod.rs`

```rust
//! Messaging module - Inter-component communication.
//!
//! Provides messaging patterns, correlation tracking, and routing.

pub mod patterns;

pub use patterns::{CorrelationManager, FireAndForget, MessageSender, RequestResponse};
```

**Deliverables**:
- RequestResponse struct in `src/messaging/patterns.rs`
- RequestResponse::request() implementation
- UUID-based correlation ID generation
- Integration with MessageSender trait
- Integration with CorrelationManager trait
- Unit tests for request-response flow (9+ new tests)
- MockCorrelationManager for testing
- Updated module exports

**Constraints**:
- Must not import from `runtime/`, `security/`, or `system/`
- Must follow §2.1 3-Layer imports
- Must follow §2.2 No FQN in type annotations
- Must generate unique correlation IDs

---

## Verification Section

### Automated Tests
```bash
# Unit tests for patterns module (includes request-response)
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
- [ ] RequestResponse struct added to `src/messaging/patterns.rs`
- [ ] RequestResponse::request() implemented
- [ ] UUID-based correlation ID generation
- [ ] Integration with MessageSender and CorrelationManager traits
- [ ] Unit tests pass (15+ total tests including WASM-TASK-041)
- [ ] MockCorrelationManager for testing
- [ ] Unique correlation IDs verified
- [ ] `cargo clippy -p airssys-wasm --lib -- -D warnings` passes
- [ ] No forbidden imports (architecture compliance)
- [ ] §2.1 3-Layer imports verified
- [ ] §2.2 No FQN verified
