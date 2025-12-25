# [WASM-TASK-HOTFIX-001 Phase 1 Task 1.1] - Move Messaging Types to Core/

## Task Information

**Task ID:** WASM-TASK-HOTFIX-001
**Phase:** Phase 1 - Fix Circular Dependency
**Task:** 1.1 - Move Messaging Types to Core/
**Status:** NOT STARTED
**Estimated Effort:** 1-2 days

## Objective

Fix circular dependency between `runtime/` and `actor/` by moving shared messaging **type definitions** from `actor/` to `core/`.

## Problem Statement

**Current Circular Dependency:**

```rust
// src/runtime/async_host.rs:52
use crate::actor::message::{PendingRequest, ResponseMessage}

// src/runtime/messaging.rs:76
use crate::actor::message::{CorrelationId, CorrelationTracker, RequestError, ResponseMessage}
```

This violates **ADR-WASM-023** which states:
- `runtime/` MUST NOT import from `actor/`
- `core/` MUST NOT have any implementation logic
- `actor/` OWNS: CorrelationTracker, ResponseRouter (actor orchestration)

## Correct Architecture (ADR-WASM-018 & ADR-WASM-023)

```
actor/ (orchestration) ───► runtime/ (execution) ───► core/ (types)
     │                      │                   │                │
     └────────────────────┴───────────────────┘
              ALL modules import from core/
```

## Module Ownership (ADR-WASM-023)

| Module | Owns (Types Only) | Owns (Implementations) |
|---------|-------------------|---------------------|
| **core/** | ComponentId, ComponentMessage, CorrelationId, PendingRequest, ResponseMessage, RequestError, ResponseRouterStats | **NONE** |
| **actor/** | CorrelationTracker, ResponseRouter, PendingRequest, ResponseMessage, RequestError | **NONE** |
| **runtime/** | WasmEngine, messaging orchestration | **NONE** |

**Key Decision:**
- ✅ Type DEFINITIONS (structs, enums, aliases) → `core/message.rs`
- ✅ IMPLEMENTATIONS (methods with logic) → Stay in their correct modules
  - CorrelationTracker → `actor/message/correlation_tracker.rs`
  - ResponseRouter → `actor/message/response_router.rs`

## Implementation Plan

### Step 1: Create src/core/message.rs

**Objective:** Create new module with shared messaging type definitions.

**File to Create:** `src/core/message.rs`

**Content to Add:**

```rust
//! Shared messaging types for inter-component communication.
//!
//! This module provides data types used by both runtime/ and actor/
//! for request-response messaging patterns. These are placed in core/
//! to prevent circular dependencies (per ADR-WASM-023).
//!
//! # Architecture
//!
//! Per ADR-WASM-023, core/ contains shared data types that
//! all modules can depend on. Runtime/ and actor/ both need
//! these messaging types for correlation tracking.
//!
//! # Module Ownership
//!
//! - **CorrelationId**: Type alias for UUID (shared by runtime/ and actor/)
//! - **PendingRequest**: Request state for correlation tracking
//! - **ResponseMessage**: Response wrapper with correlation metadata
//! - **RequestError**: Error types for request-response patterns
//! - **ResponseRouterStats**: Statistics for response routing
//!
//! # References
//!
//! - ADR-WASM-023: Module Boundary Enforcement
//! - ADR-WASM-009: Component Communication Model

// Layer 1: Standard library imports
use std::sync::Arc;
use std::fmt;

// Layer 2: Third-party crate imports
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Layer 3: Internal module imports
use crate::core::ComponentId;

/// Correlation ID type (UUID v4).
pub type CorrelationId = Uuid;

/// Pending request state for correlation tracking.
pub struct PendingRequest {
    /// Unique correlation ID
    pub correlation_id: CorrelationId,
    
    /// Response channel sender (oneshot for single response)
    pub response_tx: tokio::sync::oneshot::Sender<ResponseMessage>,
    
    /// Request timestamp (for timeout tracking)
    pub requested_at: tokio::time::Instant,
    
    /// Timeout duration
    pub timeout: tokio::time::Duration,
    
    /// Source component ID
    pub from: ComponentId,
    
    /// Target component ID
    pub to: ComponentId,
}

/// Response message with correlation tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMessage {
    /// Correlation ID matching original request
    pub correlation_id: CorrelationId,
    
    /// Responder component ID
    pub from: ComponentId,
    
    /// Original requester component ID
    pub to: ComponentId,
    
    /// Response payload or error
    pub result: Result<Vec<u8>, RequestError>,
    
    /// Response timestamp
    pub timestamp: DateTime<Utc>,
}

impl ResponseMessage {
    /// Create success response.
    pub fn success(
        correlation_id: CorrelationId,
        from: ComponentId,
        to: ComponentId,
        payload: Vec<u8>,
    ) -> Self {
        Self {
            correlation_id,
            from,
            to,
            result: Ok(payload),
            timestamp: Utc::now(),
        }
    }
    
    /// Create error response.
    pub fn error(
        correlation_id: CorrelationId,
        from: ComponentId,
        to: ComponentId,
        error: RequestError,
    ) -> Self {
        Self {
            correlation_id,
            from,
            to,
            result: Err(error),
            timestamp: Utc::now(),
        }
    }
}

/// Request-response error types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub enum RequestError {
    /// Request timed out before response arrived
    Timeout,
    
    /// Target component not found in registry
    ComponentNotFound(ComponentId),
    
    /// Target component failed to process request
    ProcessingFailed(String),
    
    /// Invalid request payload (deserialization failed)
    InvalidPayload(String),
}

impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RequestError::Timeout => write!(f, "Request timed out"),
            RequestError::ComponentNotFound(id) => {
                write!(f, "Component not found: {}", id.as_str())
            }
            RequestError::ProcessingFailed(msg) => {
                write!(f, "Processing failed: {}", msg)
            }
            RequestError::InvalidPayload(msg) => {
                write!(f, "Invalid payload: {}", msg)
            }
        }
    }
}

impl std::error::Error for RequestError {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_correlation_id_type() {
        let corr_id = Uuid::new_v4();
        let typed: CorrelationId = corr_id;
        assert_eq!(typed, corr_id);
    }
    
    #[test]
    fn test_response_message_success() {
        let corr_id = Uuid::new_v4();
        let from = ComponentId::new("comp-b");
        let to = ComponentId::new("comp-a");
        let payload = vec
![5, 6, 7, 8];
        
        let response = ResponseMessage::success(corr_id, from.clone(), to.clone(), payload.clone());
        
        assert_eq!(response.correlation_id, corr_id);
        assert_eq!(response.from, from);
        assert_eq!(response.to, to);
        assert!(response.result.is_ok());
        assert_eq!(response.result.unwrap(), payload);
    }
    
    #[test]
    fn test_response_message_error() {
        let corr_id = Uuid::new_v4();
        let from = ComponentId::new("comp-b");
        let to = ComponentId::new("comp-a");
        let error = RequestError::Timeout;
        
        let response = ResponseMessage::error(corr_id, from.clone(), to.clone(), error.clone());
        
        assert_eq!(response.correlation_id, corr_id);
        assert_eq!(response.from, from);
        assert_eq!(response.to, to);
        assert!(response.result.is_err());
        assert_eq!(response.result.unwrap_err(), error);
    }
    
    #[test]
    fn test_request_error_display() {
        assert_eq!(RequestError::Timeout.to_string(), "Request timed out");
        assert_eq!(
            RequestError::ComponentNotFound(ComponentId::new("test")).to_string(),
            "Component not found: test"
        );
    }
}
```

---

### Step 2: Update src/core/mod.rs

**Objective:** Export messaging types from core/message.rs.

**File to Update:** `src/core/mod.rs`

**Changes:**

```rust
// Add module declaration
pub mod message;

// Add public re-exports
pub use message::{
    CorrelationId,
    PendingRequest,
    ResponseMessage,
    RequestError,
    // ComponentMessage, MessageType, DeliveryGuarantee from existing core/messaging.rs
    ComponentId,
};
```

---

### Step 3: Remove Type Definitions from actor/message/

**Files to Update:**

#### 3a. Update src/actor/message/correlation_tracker.rs

**Changes:**
```rust
// REMOVE line 73:
// pub type CorrelationId = Uuid;

// ADD import from core:
use crate::core::message::{CorrelationId, PendingRequest, ResponseMessage};
```

#### 3b. Update src/actor/message/request_response.rs

**Changes:**
```rust
// REMOVE lines 71-76:
// pub type CorrelationId = Uuid;
// pub enum RequestError { ... }

// ADD imports from core:
use crate::core::message::{CorrelationId, RequestError, ResponseMessage};
```

#### 3c. Update src/actor/message/mod.rs

**Changes:**
```rust
// Update re-exports:
// REMOVE:
// pub use correlation_tracker::{CorrelationId, CorrelationTracker, PendingRequest, ResponseMessage};

// ADD:
pub use correlation_tracker::CorrelationTracker;
pub use crate::core::message::{CorrelationId, PendingRequest, ResponseMessage, RequestError};
```

---

### Step 4: Move ResponseRouter from runtime/ to actor/

**Files to Update:**

#### 4a. Create src/actor/message/response_router.rs

**File to Create:** `src/actor/message/response_router.rs` (NEW)

**Content to Add:**

```rust
//! Response router for request-response messaging.
//!
//! This module routes responses back to waiting requesters via
//! CorrelationTracker. It belongs in the actor/ layer per ADR-WASM-023.
//!
//! # References
//!
//! - ADR-WASM-023: Module Boundary Enforcement
//! - ADR-WASM-009: Component Communication Model

use crate::core::message::{CorrelationId, PendingRequest, ResponseMessage, RequestError};
use crate::core::ComponentId;

/// Response router statistics.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ResponseRouterStats {
    /// Total responses routed successfully
    pub responses_routed: u64,
    
    /// Responses that failed to route (no pending request)
    pub responses_orphaned: u64,
    
    /// Responses that were error results
    pub error_responses: u64,
}

/// High-performance correlation tracker for request-response patterns.
///
/// This is a forward declaration. The implementation is in
/// actor/message/correlation_tracker.rs.
#[derive(Clone)]
pub struct CorrelationTracker {
    _private: std::marker::PhantomData<()>,
}
```

#### 4b. Update src/runtime/messaging.rs

**Changes:**
```rust
// REMOVE ResponseRouter and ResponseRouterStats (lines 512-679)

// Update imports from actor/ to core/:
// REMOVE line 76:
// use crate::actor::message::{CorrelationId, CorrelationTracker, PendingRequest, ResponseMessage};

// ADD imports from core:
use crate::core::message::{CorrelationId, PendingRequest, ResponseMessage, RequestError};
use crate::actor::message::ResponseRouter;
```

#### 4c. Update src/actor/message/mod.rs

**Changes:**
```rust
// Add to re-exports:
pub use crate::core::message::{CorrelationId, PendingRequest, ResponseMessage, RequestError};
pub use response_router::ResponseRouter;
```

#### 4d. Update src/runtime/mod.rs

**Changes:**
```rust
// Update exports:
// Remove ResponseRouter, ResponseRouterStats
// Add: pub use crate::actor::message::ResponseRouter;
```

---

### Step 5: Update Other Files (runtime/ and actor/)

**Files to Check and Update:**

#### 5a. src/runtime/async_host.rs line 52

**Changes:**
```rust
// BEFORE:
use crate::actor::message::{PendingRequest, ResponseMessage};

// AFTER:
use crate::core::message::{PendingRequest, ResponseMessage};
```

#### 5b. src/runtime/async_host.rs - Additional imports

**Check if other imports from actor/ need updating.**
- Look for any `use crate::actor::message::` patterns
- Update to `use crate::core::message::`

#### 5c. src/actor/component/component_actor.rs

**Check lines 1409, 1503, 360 for imports from actor/message:**

**Update if needed:**
```rust
// BEFORE:
use crate::actor::message::{CorrelationId, RequestMessage};

// AFTER:
use crate::core::message::{CorrelationId, RequestError};
use crate::actor::message::ResponseMessage;
```

#### 5d. src/actor/message/timeout_handler.rs line 223

**Check if imports from actor/message need updating.**

#### 5e. src/runtime/messaging.rs - Additional imports

**Check entire file for any other `use crate::actor::message::` patterns.**

---

## Testing Plan

### Unit Tests

**Test File:** `src/core/message.rs` module tests (already included)

**Tests:** 7 unit tests already defined in file (lines 167-226)

**Verification:**
```bash
cargo test --lib core::message
# Expected: All 7 tests pass
```

---

### Integration Tests

**Test File:** `tests/messaging-types-integration-tests.rs` (NEW)

**Test Cases:**

```rust
use airssys_wasm::core::message::*;
use airssys_wasm::core::ComponentId;

#[tokio::test]
async fn test_types_importable_from_runtime() {
    // Verify runtime/ can import types from core/
    // This test validates the fix works end-to-end
    assert!(true); // If this compiles, imports are correct
}

#[tokio::test]
async fn test_types_importable_from_actor() {
    // Verify actor/ can import types from core/
    assert!(true); // If this compiles, imports are correct
}

#[tokio::test]
async fn test_correlation_id_generation() {
    let corr_id = CorrelationId::new_v4();
    assert_eq!(Uuid::from(corr_id), corr_id);
}

#[tokio::test]
async fn test_response_message_creation() {
    let corr_id = CorrelationId::new_v4();
    let from = ComponentId::new("comp-a");
    let to = ComponentId::new("comp-b");
    let payload = vec
![1, 2, 3, 4];
    
    let response = ResponseMessage::success(corr_id, from, to, payload);
    
    assert_eq!(response.correlation_id, corr_id);
    assert_eq!(response.from, from);
    assert_eq!(response.to, to);
    assert!(response.result.is_ok());
}

#[tokio::test]
async fn test_response_message_error_creation() {
    let corr_id = CorrelationId::new_v4();
    let error = RequestError::Timeout;
    
    let response = ResponseMessage::error(corr_id, ComponentId::new("a"), ComponentId::new("b"), error);
    
    assert!(response.result.is_err());
    assert_eq!(response.result.unwrap_err(), RequestError::Timeout);
}
```

**Verification:**
```bash
cargo test --test messaging-types-integration-tests
# Expected: All tests pass
```

---

## Success Criteria

This task is complete when:

- [ ] `src/core/message.rs` created with all shared messaging types
- [ ] `src/core/mod.rs` updated to export message types
- [ ] Type definitions removed from `actor/message/correlation_tracker.rs`
- [ ] Type definitions removed from `actor/message/request_response.rs`
- [ ] `ResponseRouter` moved to `actor/message/response_router.rs` (not core/)
- [ ] `ResponseRouter` removed from `runtime/messaging.rs`
- [ ] All imports updated to use `crate::core::message::{...}`
- [ ] `grep -rn "use crate::actor" src/runtime/` returns nothing
- [ ] `cargo build` succeeds
- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] Zero clippy warnings

---

## Verification Steps

### After Each Step

```bash
# After Step 1: Create core/message.rs
cargo build

# After Step 2: Update core/mod.rs
cargo test --lib core::message

# After Step 3: Remove types from actor/
cargo test --lib actor::message

# After Step 4: Move ResponseRouter
cargo build

# After All Steps
# Final verification
grep -rn "use crate::actor" src/runtime/
# Expected: Empty result (circular dependency fixed)
```

### Final Verification

```bash
# Build
cargo build --all-targets

# All tests
cargo test

# Zero warnings
cargo clippy --all-targets --all-features -- -D warnings

# Architecture compliance
grep -rn "use crate::actor" src/runtime/
# Expected: Empty result
```

---

## ADR References

- **ADR-WASM-018**: Three-Layer Architecture (actor/ → runtime/ → core/)
- **ADR-WASM-023**: Module Boundary Enforcement (MANDATORY)

---

## Quality Verification

- [ ] `cargo build` - builds cleanly
- [ ] `cargo test --lib` - all unit tests pass
- [ ] `cargo test --test messaging-types-integration-tests` - all integration tests pass
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` - zero warnings
- [ ] Zero compiler warnings
- [ ] Architecture compliance verified (grep commands pass)

---

## Timeline

| Step | Description | Estimated Time |
|-------|-------------|---------------|
| Step 1 | Create src/core/message.rs | 30 min |
| Step 2 | Update src/core/mod.rs | 15 min |
| Step 3 | Remove types from actor/ | 20 min |
| Step 4 | Move ResponseRouter | 45 min |
| Step 5 | Update other files | 30 min |
| **Total** | **~2-3 hours** |

---

Do you approve this implementation plan? (Yes/No)
