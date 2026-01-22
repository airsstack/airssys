# WASM-TASK-043: Implementation Plans

## References
- ADR-WASM-031 (Component & Messaging Module Design)
- ADR-WASM-026 (Implementation Roadmap)
- ADR-WASM-025 (Clean Slate Architecture)
- ADR-WASM-023 (Module Boundary Enforcement)
- ADR-WASM-009 (Component Communication Model)
- PROJECTS_STANDARD.md (§2.1, §2.2, §4.3)

---

## Actions

### Action 1: Implement CorrelationTrackerImpl

**Objective**: Create the concrete implementation of CorrelationTracker for request-response correlation tracking.

**Detailed Steps**:

#### Step 1.1: Create `src/messaging/correlation.rs`

```rust
//! Correlation tracking implementation for request-response patterns.
//!
//! Manages pending requests with timeouts and oneshot channel delivery.

// Layer 1: Standard library imports
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

// Layer 2: Third-party crate imports
use tokio::sync::oneshot;

// Layer 3: Internal module imports
use crate::core::component::MessagePayload;
use crate::core::errors::messaging::MessagingError;
use crate::core::messaging::traits::CorrelationTracker;

/// Implementation of CorrelationTracker for request-response patterns
///
/// Tracks pending requests using oneshot channels for response delivery.
/// Provides timeout-based expiration and cleanup.
pub struct CorrelationTrackerImpl {
    /// Pending requests tracked by correlation ID
    pending: RwLock<HashMap<String, PendingRequest>>,
}

/// Internal struct for tracking a pending request
struct PendingRequest {
    /// Oneshot sender for delivering the response
    sender: oneshot::Sender<MessagePayload>,
    /// Deadline for this request
    deadline: Instant,
}

impl CorrelationTrackerImpl {
    /// Create a new correlation tracker
    pub fn new() -> Self {
        Self {
            pending: RwLock::new(HashMap::new()),
        }
    }

    /// Create a new correlation and return a receiver for the response
    ///
    /// # Arguments
    /// * `correlation_id` - The correlation identifier
    /// * `timeout_ms` - Timeout in milliseconds
    ///
    /// # Returns
    /// Oneshot receiver that will receive the response
    pub fn create(&self, correlation_id: String, timeout_ms: u64) -> oneshot::Receiver<MessagePayload> {
        let (sender, receiver) = oneshot::channel();
        
        let request = PendingRequest {
            sender,
            deadline: Instant::now() + Duration::from_millis(timeout_ms),
        };

        let mut pending = self.pending.write().unwrap();
        pending.insert(correlation_id, request);

        receiver
    }

    /// Clean up expired correlations
    ///
    /// Removes all correlations that have exceeded their deadline.
    pub fn cleanup_expired(&self) {
        let now = Instant::now();
        let mut pending = self.pending.write().unwrap();
        pending.retain(|_, req| req.deadline > now);
    }
}

impl CorrelationTracker for CorrelationTrackerImpl {
    fn register(&self, correlation_id: &str, timeout_ms: u64) -> Result<(), MessagingError> {
        let (sender, _receiver) = oneshot::channel();
        
        let request = PendingRequest {
            sender,
            deadline: Instant::now() + Duration::from_millis(timeout_ms),
        };

        let mut pending = self.pending.write().unwrap();
        pending.insert(correlation_id.to_string(), request);
        
        Ok(())
    }

    fn complete(&self, correlation_id: &str, response: MessagePayload) -> Result<(), MessagingError> {
        let mut pending = self.pending.write().unwrap();
        
        if let Some(request) = pending.remove(correlation_id) {
            if request.deadline > Instant::now() {
                let _ = request.sender.send(response);
                Ok(())
            } else {
                Err(MessagingError::CorrelationTimeout(correlation_id.to_string()))
            }
        } else {
            Err(MessagingError::InvalidMessage(format!(
                "No pending request for correlation {}",
                correlation_id
            )))
        }
    }

    fn is_pending(&self, correlation_id: &str) -> bool {
        let pending = self.pending.read().unwrap();
        pending.contains_key(correlation_id)
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
        assert!(!tracker.is_pending("nonexistent"));
    }

    #[test]
    fn test_tracker_default() {
        let tracker = CorrelationTrackerImpl::default();
        assert!(!tracker.is_pending("nonexistent"));
    }

    #[test]
    fn test_register_correlation() {
        let tracker = CorrelationTrackerImpl::new();
        let correlation_id = "test-correlation-123";

        let result = tracker.register(correlation_id, 5000);
        assert!(result.is_ok());
        assert!(tracker.is_pending(correlation_id));
    }

    #[test]
    fn test_complete_correlation_success() {
        let tracker = CorrelationTrackerImpl::new();
        let correlation_id = "test-correlation-456";

        tracker.register(correlation_id, 5000).unwrap();
        assert!(tracker.is_pending(correlation_id));

        let response = MessagePayload::Text("response data".to_string());
        let result = tracker.complete(correlation_id, response);
        assert!(result.is_ok());
        assert!(!tracker.is_pending(correlation_id));
    }

    #[test]
    fn test_complete_nonexistent_correlation() {
        let tracker = CorrelationTrackerImpl::new();
        let correlation_id = "nonexistent-correlation";

        let response = MessagePayload::Text("response".to_string());
        let result = tracker.complete(correlation_id, response);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_and_receive() {
        let tracker = CorrelationTrackerImpl::new();
        let correlation_id = "test-correlation-789".to_string();

        let mut receiver = tracker.create(correlation_id.clone(), 5000);
        assert!(tracker.is_pending(&correlation_id));

        // Complete the correlation
        let response = MessagePayload::Binary(vec![1, 2, 3]);
        tracker.complete(&correlation_id, response.clone()).unwrap();

        // Verify receiver got the response
        let received = receiver.try_recv();
        assert!(received.is_ok());
    }

    #[test]
    fn test_is_pending_after_complete() {
        let tracker = CorrelationTrackerImpl::new();
        let correlation_id = "test-correlation-complete";

        tracker.register(correlation_id, 5000).unwrap();
        assert!(tracker.is_pending(correlation_id));

        let response = MessagePayload::Text("done".to_string());
        tracker.complete(correlation_id, response).unwrap();

        assert!(!tracker.is_pending(correlation_id));
    }

    #[test]
    fn test_multiple_correlations() {
        let tracker = CorrelationTrackerImpl::new();
        let id1 = "correlation-1";
        let id2 = "correlation-2";
        let id3 = "correlation-3";

        tracker.register(id1, 5000).unwrap();
        tracker.register(id2, 5000).unwrap();
        tracker.register(id3, 5000).unwrap();

        assert!(tracker.is_pending(id1));
        assert!(tracker.is_pending(id2));
        assert!(tracker.is_pending(id3));

        // Complete one
        tracker.complete(id2, MessagePayload::Text("done".to_string())).unwrap();

        assert!(tracker.is_pending(id1));
        assert!(!tracker.is_pending(id2));
        assert!(tracker.is_pending(id3));
    }

    #[test]
    fn test_cleanup_expired_with_valid_correlations() {
        let tracker = CorrelationTrackerImpl::new();
        let correlation_id = "test-correlation-valid";

        // Register with long timeout (won't expire)
        tracker.register(correlation_id, 60_000).unwrap();
        assert!(tracker.is_pending(correlation_id));

        // Cleanup should not remove valid correlation
        tracker.cleanup_expired();
        assert!(tracker.is_pending(correlation_id));
    }

    #[tokio::test]
    async fn test_cleanup_expired_with_expired_correlations() {
        let tracker = CorrelationTrackerImpl::new();
        let correlation_id = "test-correlation-expired";

        // Register with very short timeout
        tracker.register(correlation_id, 1).unwrap();
        assert!(tracker.is_pending(correlation_id));

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Cleanup should remove expired correlation
        tracker.cleanup_expired();
        assert!(!tracker.is_pending(correlation_id));
    }

    #[tokio::test]
    async fn test_complete_expired_correlation_fails() {
        let tracker = CorrelationTrackerImpl::new();
        let correlation_id = "test-correlation-timeout";

        // Register with very short timeout
        tracker.register(correlation_id, 1).unwrap();

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Try to complete expired correlation
        let response = MessagePayload::Text("late response".to_string());
        let result = tracker.complete(correlation_id, response);
        assert!(result.is_err());

        match result {
            Err(MessagingError::CorrelationTimeout(_)) => {}
            _ => panic!("Expected CorrelationTimeout error"),
        }
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
                let correlation_id = format!("correlation-{}", i);
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
            let correlation_id = format!("correlation-{}", i);
            assert!(tracker.is_pending(&correlation_id));
        }
    }
}
```

#### Step 1.2: Update `src/messaging/mod.rs`

```rust
//! Messaging module - Inter-component communication.
//!
//! Provides messaging patterns, correlation tracking, and routing.

pub mod correlation;
pub mod patterns;

pub use correlation::CorrelationTrackerImpl;
pub use patterns::{CorrelationManager, FireAndForget, MessageSender, RequestResponse};
```

**Deliverables**:
- `src/messaging/correlation.rs` with CorrelationTrackerImpl
- PendingRequest internal struct
- CorrelationTracker trait implementation
- create() method returning oneshot::Receiver
- cleanup_expired() method
- Default trait implementation
- Comprehensive unit tests (13+ tests)
- Module export in `src/messaging/mod.rs`

**Constraints**:
- Must not import from `runtime/`, `security/`, or `system/`
- Must follow §2.1 3-Layer imports
- Must follow §2.2 No FQN in type annotations
- Must use RwLock for thread safety
- Must use oneshot channels for response delivery

---

## Verification Section

### Automated Tests
```bash
# Unit tests for correlation module
cargo test -p airssys-wasm --lib -- messaging::correlation

# All messaging module tests
cargo test -p airssys-wasm --lib -- messaging

# Build verification
cargo build -p airssys-wasm

# Clippy
cargo clippy -p airssys-wasm --lib -- -D warnings
```

### Architecture Compliance
```bash
# Verify no forbidden imports in correlation.rs
grep -rn "use crate::runtime" src/messaging/correlation.rs  # Should be empty
grep -rn "use crate::security" src/messaging/correlation.rs  # Should be empty
grep -rn "use crate::system" src/messaging/correlation.rs    # Should be empty

# Verify no FQN in type annotations
grep -rn "std::" src/messaging/correlation.rs | grep -v "^.*use " | grep "::"  # Should be empty
```

---

## Success Criteria
- [ ] `src/messaging/correlation.rs` exists and compiles
- [ ] CorrelationTrackerImpl struct
- [ ] PendingRequest internal struct
- [ ] CorrelationTracker trait implementation
- [ ] create() method with oneshot::Receiver
- [ ] cleanup_expired() method
- [ ] Default trait implementation
- [ ] Unit tests pass (13+ tests including timeout/expiration tests)
- [ ] Thread-safety verified (concurrent access test)
- [ ] `cargo clippy -p airssys-wasm --lib -- -D warnings` passes
- [ ] No forbidden imports (architecture compliance)
- [ ] §2.1 3-Layer imports verified
- [ ] §2.2 No FQN verified
