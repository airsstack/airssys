# WASM-TASK-022: Implementation Plans

## Plan References
- **ADR-WASM-028:** Core Module Structure (primary specification, lines 489-583)
- **ADR-WASM-025:** Clean-Slate Rebuild Architecture
- **ADR-WASM-026:** Implementation Roadmap (Phase 3 - lines 109-123)
- **KNOWLEDGE-WASM-037:** Rebuild Architecture Clean-Slate
- **PROJECTS_STANDARD.md §4.3:** Module Architecture Patterns, §6.1 Error Handling Strategy

## Target Structure Reference

Per ADR-WASM-028:
```
core/errors/
├── mod.rs           # Module declarations only (no glob re-exports)
├── wasm.rs          # WasmError enum (thiserror-based)
├── security.rs      # SecurityError enum (thiserror-based)
├── messaging.rs     # MessagingError enum (thiserror-based)
└── storage.rs       # StorageError enum (thiserror-based)
```

## Key Design Decisions

### Using `thiserror` Crate
Per PROJECTS_STANDARD.md "Error Handling Strategy":
> Use `thiserror` for error definitions

This simplifies error implementation and follows workspace conventions:
- Automatic `Display` and `std::error::Error` implementations
- Cleaner, more maintainable code
- Consistent with other AirsSys crates

### Module Re-export Pattern
Per ADR-WASM-028 updated policy and user preference:
- Use `pub mod X;` for module declarations
- **Avoid** glob re-exports (`pub use X::*`) in parent `mod.rs`
- Callers use namespaced access: `core::errors::WasmError`
- Provides clear type grouping and prevents namespace pollution

---

## Implementation Actions

### Action 1: Create core/errors/wasm.rs

**Objective:** Implement `WasmError` enum using `thiserror`

**Reference:** ADR-WASM-028 lines 489-521

**Implementation:**
```rust
//! WASM execution error types.

use thiserror::Error;

/// WASM execution errors.
///
/// Represents all possible error conditions during WASM component
/// loading, instantiation, and execution.
#[derive(Debug, Clone, Error)]
pub enum WasmError {
    /// Component with the given ID was not found.
    #[error("Component not found: {0}")]
    ComponentNotFound(String),

    /// Failed to instantiate the WASM component.
    #[error("Instantiation failed: {0}")]
    InstantiationFailed(String),

    /// Required export was not found in the component.
    #[error("Export not found: {0}")]
    ExportNotFound(String),

    /// Execution exceeded the configured timeout.
    #[error("Execution timeout")]
    Timeout,

    /// Execution exceeded resource limits (memory, fuel, etc.).
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),

    /// Component binary is invalid or malformed.
    #[error("Invalid component: {0}")]
    InvalidComponent(String),

    /// Runtime error during execution.
    #[error("Runtime error: {0}")]
    RuntimeError(String),
}
```

**Unit Tests Required:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_error_display() {
        let err = WasmError::ComponentNotFound("test/comp/001".into());
        assert_eq!(err.to_string(), "Component not found: test/comp/001");
    }

    #[test]
    fn test_wasm_error_is_error() {
        let err: Box<dyn std::error::Error> = Box::new(WasmError::Timeout);
        assert!(err.to_string().contains("timeout"));
    }

    #[test]
    fn test_wasm_error_clone() {
        let err = WasmError::RuntimeError("test".into());
        let cloned = err.clone();
        assert_eq!(err.to_string(), cloned.to_string());
    }
}
```

**Verification:**
```bash
cargo build -p airssys-wasm
cargo test -p airssys-wasm --lib wasm_error
```

---

### Action 2: Create core/errors/security.rs

**Objective:** Implement `SecurityError` enum using `thiserror`

**Reference:** ADR-WASM-028 lines 525-551

**Implementation:**
```rust
//! Security-related error types.

use thiserror::Error;

/// Security-related errors.
///
/// Represents errors during capability validation, policy enforcement,
/// and security context operations.
#[derive(Debug, Clone, Error)]
pub enum SecurityError {
    /// Required capability was denied.
    #[error("Capability denied: {0}")]
    CapabilityDenied(String),

    /// Operation violates configured security policy.
    #[error("Policy violation: {0}")]
    PolicyViolation(String),

    /// Security context is invalid or missing.
    #[error("Invalid context: {0}")]
    InvalidContext(String),

    /// Permission denied for the requested operation.
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}
```

**Unit Tests Required:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_error_display() {
        let err = SecurityError::CapabilityDenied("messaging:send".into());
        assert_eq!(err.to_string(), "Capability denied: messaging:send");
    }

    #[test]
    fn test_security_error_is_error() {
        let err: Box<dyn std::error::Error> = Box::new(
            SecurityError::PolicyViolation("access denied".into())
        );
        assert!(err.to_string().contains("Policy violation"));
    }
}
```

---

### Action 3: Create core/errors/messaging.rs

**Objective:** Implement `MessagingError` enum using `thiserror`

**Reference:** ADR-WASM-028 lines 555-583

**Implementation:**
```rust
//! Messaging error types.

use thiserror::Error;

/// Messaging errors.
///
/// Represents errors during inter-component message routing,
/// delivery, and correlation tracking.
#[derive(Debug, Clone, Error)]
pub enum MessagingError {
    /// Message delivery failed.
    #[error("Delivery failed: {0}")]
    DeliveryFailed(String),

    /// Request-response correlation timed out.
    #[error("Correlation timeout: {0}")]
    CorrelationTimeout(String),

    /// Message format is invalid.
    #[error("Invalid message: {0}")]
    InvalidMessage(String),

    /// Message queue is full, cannot accept more messages.
    #[error("Message queue full")]
    QueueFull,

    /// Target component not found.
    #[error("Target not found: {0}")]
    TargetNotFound(String),
}
```

**Unit Tests Required:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_messaging_error_display() {
        let err = MessagingError::QueueFull;
        assert_eq!(err.to_string(), "Message queue full");
    }

    #[test]
    fn test_target_not_found() {
        let err = MessagingError::TargetNotFound("app/service/001".into());
        assert!(err.to_string().contains("app/service/001"));
    }
}
```

---

### Action 4: Create core/errors/storage.rs

**Objective:** Implement `StorageError` enum using `thiserror`

**Reference:** ADR-WASM-028 (implied by core/storage/traits.rs usage)

**Implementation:**
```rust
//! Storage operation error types.

use thiserror::Error;

/// Storage operation errors.
///
/// Represents errors during component-isolated storage operations
/// (get, set, delete, list).
#[derive(Debug, Clone, Error)]
pub enum StorageError {
    /// Key was not found in storage.
    #[error("Key not found: {0}")]
    KeyNotFound(String),

    /// Failed to write to storage.
    #[error("Write error: {0}")]
    WriteError(String),

    /// Failed to read from storage.
    #[error("Read error: {0}")]
    ReadError(String),

    /// Storage quota exceeded.
    #[error("Quota exceeded: {0}")]
    QuotaExceeded(String),

    /// Key format is invalid.
    #[error("Invalid key: {0}")]
    InvalidKey(String),
}
```

**Unit Tests Required:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_error_display() {
        let err = StorageError::KeyNotFound("config.json".into());
        assert_eq!(err.to_string(), "Key not found: config.json");
    }

    #[test]
    fn test_quota_exceeded() {
        let err = StorageError::QuotaExceeded("10MB limit".into());
        assert!(err.to_string().contains("Quota exceeded"));
    }
}
```

---

### Action 5: Create core/errors/mod.rs

**Objective:** Module declarations only (no glob re-exports per user preference)

**Implementation:**
```rust
//! Core error types for airssys-wasm.
//!
//! This module contains all error types used across the crate.
//! All error types use `thiserror` for derive macros and implement
//! `std::error::Error` and `Display`.
//!
//! # Modules
//!
//! - [`wasm`] - WASM execution errors
//! - [`security`] - Security-related errors
//! - [`messaging`] - Messaging errors
//! - [`storage`] - Storage operation errors
//!
//! # Usage
//!
//! ```
//! use airssys_wasm::core::errors::wasm::WasmError;
//! use airssys_wasm::core::errors::messaging::MessagingError;
//! ```

pub mod messaging;
pub mod security;
pub mod storage;
pub mod wasm;

// NOTE: No glob re-exports (pub use X::*) per PROJECTS_STANDARD.md
// Callers use namespaced access: core::errors::wasm::WasmError
```

---

### Action 6: Update core/mod.rs

**Objective:** Add errors submodule declaration (no glob re-export)

**Changes:**
```rust
// Add to core/mod.rs
pub mod errors;

// NOTE: No `pub use errors::*;` per module grouping policy
// Callers use: crate::core::errors::wasm::WasmError
```

---

## Verification Commands

Run after ALL actions complete:
```bash
# 1. Build check
cargo build -p airssys-wasm

# 2. Lint check
cargo clippy -p airssys-wasm --all-targets -- -D warnings

# 3. Unit tests
cargo test -p airssys-wasm --lib

# 4. Verify thiserror usage
grep -rn "use thiserror::Error" airssys-wasm/src/core/errors/
# Should show 4 files using thiserror

# 5. Verify no glob re-exports
grep -rn "pub use.*::\*" airssys-wasm/src/core/
# Should return empty - no glob re-exports

# 6. Verify std::error::Error implementation
cargo test -p airssys-wasm --lib -- --test error
```

## Success Criteria
- All 4 error types from ADR-WASM-028 implemented with `thiserror`
- Build passes with zero warnings
- All error types derive `thiserror::Error` (automatic std::error::Error + Display)
- Unit tests cover error creation and formatting
- Module architecture follows §4.3 (mod.rs only declarations)
- No glob re-exports (`pub use X::*`) used

## Dependencies Note
This task requires adding `thiserror` to Cargo.toml dependencies:
```toml
[dependencies]
thiserror = { workspace = true }
```
