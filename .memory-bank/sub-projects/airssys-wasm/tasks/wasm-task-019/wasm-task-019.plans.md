# WASM-TASK-019: Implementation Plans

## Plan References
- **KNOWLEDGE-WASM-040:** Messaging Module - Comprehensive Reference 🔴 **PRIMARY**
- **ADR-WASM-028:** Core Module Structure (updated 2026-01-09)
- **ADR-WASM-025:** Clean-Slate Rebuild Architecture
- **PROJECTS_STANDARD.md §2.1:** 3-Layer Import Organization
- **PROJECTS_STANDARD.md §4.3:** Module Architecture Patterns

## Target Structure Reference

```
core/messaging/
├── mod.rs           # Module declarations only
├── errors.rs        # MessagingError (co-located)
├── correlation.rs   # CorrelationId type
└── traits.rs        # MessageRouter, CorrelationTracker traits
```

## Dependencies

- **Upstream:**
  - WASM-TASK-017 (core/component/) ✅ COMPLETE
- **External Crate:** `uuid` for CorrelationId generation

---

## Implementation Actions

### Action 1: Create core/messaging/errors.rs

**Objective:** Define MessagingError (co-located with messaging module)

**Implementation:**
```rust
//! Messaging error types.

// Layer 1: Standard library
// (none)

// Layer 2: External crates
use thiserror::Error;

// Layer 3: Internal modules
// (none - errors have no internal dependencies)

/// Messaging errors for inter-component communication.
#[derive(Debug, Clone, Error)]
pub enum MessagingError {
    /// Message delivery failed.
    #[error("Message delivery failed: {0}")]
    DeliveryFailed(String),

    /// Correlation timeout - response not received in time.
    #[error("Correlation timeout: {0}")]
    CorrelationTimeout(String),

    /// Invalid message format or content.
    #[error("Invalid message: {0}")]
    InvalidMessage(String),

    /// Message queue is full.
    #[error("Message queue is full")]
    QueueFull,

    /// Target component not found.
    #[error("Target component not found: {0}")]
    TargetNotFound(String),
}
```

**Unit Tests:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delivery_failed_display() {
        let err = MessagingError::DeliveryFailed("connection lost".to_string());
        assert_eq!(format!("{}", err), "Message delivery failed: connection lost");
    }

    #[test]
    fn test_correlation_timeout_display() {
        let err = MessagingError::CorrelationTimeout("corr-123".to_string());
        assert_eq!(format!("{}", err), "Correlation timeout: corr-123");
    }

    #[test]
    fn test_queue_full_display() {
        let err = MessagingError::QueueFull;
        assert_eq!(format!("{}", err), "Message queue is full");
    }

    #[test]
    fn test_target_not_found_display() {
        let err = MessagingError::TargetNotFound("app/service/001".to_string());
        assert_eq!(format!("{}", err), "Target component not found: app/service/001");
    }

    #[test]
    fn test_error_is_clone() {
        let err = MessagingError::QueueFull;
        let cloned = err.clone();
        assert!(matches!(cloned, MessagingError::QueueFull));
    }
}
```

---

### Action 2: Create core/messaging/correlation.rs

**Implementation:**
```rust
//! Correlation ID types for request-response patterns.

// Layer 1: Standard library
use std::fmt;

// Layer 2: External crates
use uuid::Uuid;

// Layer 3: Internal modules
// (none)

/// Unique identifier for correlating request-response pairs.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CorrelationId(String);

impl CorrelationId {
    /// Create a new CorrelationId from a string.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Generate a new unique CorrelationId using UUID v4.
    pub fn generate() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// Returns the correlation ID as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CorrelationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for CorrelationId {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for CorrelationId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}
```

**Unit Tests:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_from_string() {
        let id = CorrelationId::new("test-123");
        assert_eq!(id.as_str(), "test-123");
    }

    #[test]
    fn test_generate_unique() {
        let id1 = CorrelationId::generate();
        let id2 = CorrelationId::generate();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_generate_is_valid_uuid() {
        let id = CorrelationId::generate();
        assert!(id.as_str().contains('-'));
        assert_eq!(id.as_str().len(), 36);
    }

    #[test]
    fn test_from_conversions() {
        let id1: CorrelationId = "from-str".into();
        let id2: CorrelationId = String::from("from-string").into();
        assert_eq!(id1.as_str(), "from-str");
        assert_eq!(id2.as_str(), "from-string");
    }

    #[test]
    fn test_hash_and_eq() {
        use std::collections::HashSet;

        let id1 = CorrelationId::new("same");
        let id2 = CorrelationId::new("same");

        assert_eq!(id1, id2);

        let mut set = HashSet::new();
        set.insert(id1);
        assert!(set.contains(&id2));
    }
}
```

---

### Action 3: Create core/messaging/traits.rs

**Implementation:**
```rust
//! Messaging trait abstractions.

// Layer 1: Standard library
// (none)

// Layer 2: External crates
// (none)

// Layer 3: Internal modules
use crate::core::component::id::ComponentId;
use crate::core::component::message::MessagePayload;
use super::correlation::CorrelationId;
use super::errors::MessagingError;

/// Trait for message routing between components.
pub trait MessageRouter: Send + Sync {
    /// Send a fire-and-forget message.
    fn send(
        &self,
        target: &ComponentId,
        payload: MessagePayload,
    ) -> Result<(), MessagingError>;

    /// Send a request expecting a response.
    fn request(
        &self,
        target: &ComponentId,
        payload: MessagePayload,
        timeout_ms: u64,
    ) -> Result<CorrelationId, MessagingError>;

    /// Cancel a pending request.
    fn cancel_request(
        &self,
        correlation_id: &CorrelationId,
    ) -> Result<(), MessagingError>;
}

/// Trait for tracking request-response correlations.
pub trait CorrelationTracker: Send + Sync {
    /// Register a pending request.
    fn register(
        &self,
        correlation_id: &CorrelationId,
        timeout_ms: u64,
    ) -> Result<(), MessagingError>;

    /// Complete a pending request with a response.
    fn complete(
        &self,
        correlation_id: &CorrelationId,
        response: MessagePayload,
    ) -> Result<(), MessagingError>;

    /// Check if a correlation is still pending.
    fn is_pending(&self, correlation_id: &CorrelationId) -> bool;

    /// Remove a correlation without completing it.
    fn remove(
        &self,
        correlation_id: &CorrelationId,
    ) -> Result<(), MessagingError>;
}
```

**Unit Tests:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trait_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<dyn MessageRouter>();
        assert_send_sync::<dyn CorrelationTracker>();
    }
}
```

---

### Action 4: Create core/messaging/mod.rs

```rust
//! Messaging abstractions for inter-component communication.
//!
//! This module contains types, traits, and errors for message
//! routing and correlation tracking between WASM components.
//!
//! # Design
//!
//! These are **abstractions** (Layer 1). Concrete implementations
//! live in the `messaging/` module (Layer 3).

pub mod correlation;
pub mod errors;
pub mod traits;
```

---

### Action 5: Update core/mod.rs

```rust
// Add to core/mod.rs
pub mod messaging;
```

---

### Action 6: Add uuid dependency

```toml
# Add to Cargo.toml
[dependencies]
uuid = { version = "1", features = ["v4"] }
```

---

## Verification Commands

```bash
cargo build -p airssys-wasm
cargo clippy -p airssys-wasm --all-targets -- -D warnings
cargo test -p airssys-wasm --lib messaging
```

## Success Criteria

- `MessagingError` co-located in `core/messaging/errors.rs`
- `CorrelationId` with UUID generation
- `MessageRouter` and `CorrelationTracker` traits
- All imports use `super::` for same-module access (no cross-module errors)
- Build passes with zero warnings
- Unit tests pass

## Design Summary

```
core/messaging/
├── errors.rs
│   └── MessagingError
│       ├── DeliveryFailed(String)
│       ├── CorrelationTimeout(String)
│       ├── InvalidMessage(String)
│       ├── QueueFull
│       └── TargetNotFound(String)
├── correlation.rs
│   └── CorrelationId
│       ├── new(impl Into<String>) -> Self
│       ├── generate() -> Self
│       ├── as_str() -> &str
│       └── impl Display, Clone, PartialEq, Eq, Hash, From
└── traits.rs
    ├── MessageRouter (Send + Sync)
    │   ├── send(target, payload) -> Result<(), MessagingError>
    │   ├── request(target, payload, timeout) -> Result<CorrelationId, MessagingError>
    │   └── cancel_request(correlation_id) -> Result<(), MessagingError>
    └── CorrelationTracker (Send + Sync)
        ├── register(correlation_id, timeout) -> Result<(), MessagingError>
        ├── complete(correlation_id, response) -> Result<(), MessagingError>
        ├── is_pending(correlation_id) -> bool
        └── remove(correlation_id) -> Result<(), MessagingError>
```
