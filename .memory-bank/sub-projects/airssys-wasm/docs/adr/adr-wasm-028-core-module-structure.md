# ADR-WASM-028: Core Module Structure

**ADR ID:** ADR-WASM-028  
**Created:** 2026-01-05  
**Status:** Accepted  
**Deciders:** Architecture Team  
**Category:** Module Design / Core Abstractions  
**Parent:** [ADR-WASM-026](adr-wasm-026-implementation-roadmap-clean-slate-rebuild.md) (Phase 3)

---

## Title

Core Module Structure for Layer-Organized Abstractions

---

## Context

The `core/` module is **Layer 1** of the architecture. It contains ONLY:
- Foundation types (data structures)
- Trait definitions (abstractions)
- Error types

**Critical Rule:** `core/` has **zero internal dependencies** (no imports from other airssys-wasm modules). External utility crates are allowed:
- ✅ `std` (standard library)
- ✅ `thiserror` (error derive macros)
- ✅ `serde` (serialization, if needed)
- ✅ `chrono` (time types, if needed)
- ❌ Any internal `crate::` imports

All other modules depend on `core/`, but `core/` depends on nothing internal.

### Design Principle

Layer-organized structure where each outer module has a corresponding `core/<module>/` containing its abstractions. This enables proper Dependency Inversion.

### References

- [ADR-WASM-025](adr-wasm-025-clean-slate-rebuild-architecture.md): Clean-Slate Architecture
- [KNOWLEDGE-WASM-037](../knowledges/knowledge-wasm-037-rebuild-architecture-clean-slate.md): Technical Reference

---

## Decision

### Core Module Structure

```
core/
├── mod.rs
├── component/
│   ├── mod.rs
│   ├── id.rs           # ComponentId
│   ├── handle.rs       # ComponentHandle
│   ├── message.rs      # ComponentMessage, MessageMetadata, MessagePayload
│   ├── errors.rs       # ComponentError (co-located)
│   └── traits.rs       # Component-related traits
├── runtime/
│   ├── mod.rs
│   ├── errors.rs       # WasmError (co-located with runtime)
│   ├── traits.rs       # RuntimeEngine, ComponentLoader
│   └── limits.rs       # ResourceLimits
├── messaging/
│   ├── mod.rs
│   ├── errors.rs       # MessagingError (co-located with messaging)
│   ├── correlation.rs  # CorrelationId
│   └── traits.rs       # MessageRouter, CorrelationTracker traits
├── security/
│   ├── mod.rs
│   ├── errors.rs       # SecurityError (co-located with security)
│   ├── capability.rs   # Capability types
│   └── traits.rs       # SecurityValidator trait
├── storage/
│   ├── mod.rs
│   ├── errors.rs       # StorageError (co-located with storage)
│   └── traits.rs       # ComponentStorage trait
└── config/
    ├── mod.rs
    └── component.rs    # ComponentConfig, ConfigValidationError
```

> **Design Decision: Co-located Errors**
> 
> Each module contains its own error types in an `errors.rs` file. This provides:
> - **Module isolation** - Everything a module needs is self-contained
> - **No cross-dependencies** - No awkward imports from `core/errors/`
> - **Better cohesion** - Related types live together
> - **Simpler mental model** - Look in one module to see everything

---

## Detailed Specifications

### core/component/id.rs

```rust
use std::fmt;

/// Unique identifier for a component instance
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ComponentId {
    pub namespace: String,
    pub name: String,
    pub instance: String,
}

impl ComponentId {
    pub fn new(namespace: impl Into<String>, name: impl Into<String>, instance: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            name: name.into(),
            instance: instance.into(),
        }
    }

    pub fn to_string_id(&self) -> String {
        format!("{}/{}/{}", self.namespace, self.name, self.instance)
    }
}

impl fmt::Display for ComponentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string_id())
    }
}
```

---

### core/component/handle.rs

```rust
use super::id::ComponentId;

/// Opaque handle to a loaded component instance
#[derive(Debug, Clone)]
pub struct ComponentHandle {
    id: ComponentId,
    handle_id: u64,
}

impl ComponentHandle {
    pub fn new(id: ComponentId, handle_id: u64) -> Self {
        Self { id, handle_id }
    }

    pub fn id(&self) -> &ComponentId {
        &self.id
    }

    pub fn handle_id(&self) -> u64 {
        self.handle_id
    }
}
```

---

### core/component/message.rs

```rust
use super::id::ComponentId;

/// Message payload wrapper (raw bytes, typically multicodec-encoded).
/// 
/// MessagePayload wraps raw bytes for inter-component communication.
/// This type lives in `core/component/` because it is a fundamental
/// data type used by ComponentMessage, not a messaging behavior.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessagePayload(Vec<u8>);

impl MessagePayload {
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }
    
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
    
    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
    
    pub fn len(&self) -> usize {
        self.0.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<Vec<u8>> for MessagePayload {
    fn from(data: Vec<u8>) -> Self {
        Self::new(data)
    }
}

/// Metadata for a component message
#[derive(Debug, Clone)]
pub struct MessageMetadata {
    pub correlation_id: Option<String>,
    pub reply_to: Option<ComponentId>,
    pub timestamp_ms: u64,
    pub content_type: Option<String>,
}

impl Default for MessageMetadata {
    fn default() -> Self {
        Self {
            correlation_id: None,
            reply_to: None,
            timestamp_ms: 0,
            content_type: None,
        }
    }
}

/// Complete message envelope for component communication
#[derive(Debug, Clone)]
pub struct ComponentMessage {
    pub sender: ComponentId,
    pub payload: MessagePayload,
    pub metadata: MessageMetadata,
}
```

---

### core/component/errors.rs

```rust
//! Component error types.

use thiserror::Error;

/// Component-related errors for lifecycle and operations.
#[derive(Debug, Clone, Error)]
pub enum ComponentError {
    /// Component initialization failed.
    #[error("Component initialization failed: {0}")]
    InitializationFailed(String),

    /// Component shutdown failed.
    #[error("Component shutdown failed: {0}")]
    ShutdownFailed(String),

    /// Component not found.
    #[error("Component not found: {0}")]
    NotFound(String),

    /// Component already exists.
    #[error("Component already exists: {0}")]
    AlreadyExists(String),

    /// Invalid component state.
    #[error("Invalid component state: {0}")]
    InvalidState(String),
}
```

---

### core/component/traits.rs

```rust
use super::errors::ComponentError;

/// Trait for component lifecycle management
pub trait ComponentLifecycle: Send + Sync {
    /// Initialize the component
    fn initialize(&mut self) -> Result<(), ComponentError>;
    
    /// Shutdown the component
    fn shutdown(&mut self) -> Result<(), ComponentError>;
    
    /// Check component health
    fn health_check(&self) -> bool;
}
```

---

### core/runtime/traits.rs

```rust
use crate::core::component::{handle::ComponentHandle, id::ComponentId, message::ComponentMessage};
use crate::core::errors::wasm::WasmError;
use crate::core::messaging::payload::MessagePayload;

/// Trait for WASM runtime engine abstraction
/// Implemented by runtime/ module, consumed by component/ module
pub trait RuntimeEngine: Send + Sync {
    /// Load a WASM component from bytes
    fn load_component(&self, id: &ComponentId, bytes: &[u8]) -> Result<ComponentHandle, WasmError>;
    
    /// Unload a component
    fn unload_component(&self, handle: &ComponentHandle) -> Result<(), WasmError>;
    
    /// Call handle_message on a component
    fn call_handle_message(
        &self,
        handle: &ComponentHandle,
        msg: &ComponentMessage,
    ) -> Result<Option<MessagePayload>, WasmError>;
    
    /// Call handle_callback on a component
    fn call_handle_callback(
        &self,
        handle: &ComponentHandle,
        msg: &ComponentMessage,
    ) -> Result<(), WasmError>;
}

/// Trait for loading component binaries
pub trait ComponentLoader: Send + Sync {
    /// Load component bytes from a path or identifier
    fn load_bytes(&self, id: &ComponentId) -> Result<Vec<u8>, WasmError>;
    
    /// Validate component bytes before loading
    fn validate(&self, bytes: &[u8]) -> Result<(), WasmError>;
}
```

---

### core/runtime/limits.rs

```rust
/// Resource limits for component execution
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub max_memory_bytes: u64,
    pub max_execution_time_ms: u64,
    pub max_fuel: Option<u64>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 64 * 1024 * 1024, // 64MB
            max_execution_time_ms: 30_000,       // 30 seconds
            max_fuel: None,
        }
    }
}
```

---

### core/messaging/traits.rs

```rust
use crate::core::component::{ComponentId, MessagePayload};
use crate::core::errors::messaging::MessagingError;

/// Trait for message routing between components
/// Implemented by messaging/ module
pub trait MessageRouter: Send + Sync {
    /// Send fire-and-forget message
    fn send(&self, target: &ComponentId, payload: MessagePayload) -> Result<(), MessagingError>;
    
    /// Send request expecting response
    fn request(
        &self,
        target: &ComponentId,
        payload: MessagePayload,
        timeout_ms: u64,
    ) -> Result<String, MessagingError>;
    
    /// Cancel pending request
    fn cancel_request(&self, request_id: &str) -> Result<(), MessagingError>;
}

/// Trait for tracking request-response correlations
pub trait CorrelationTracker: Send + Sync {
    /// Register a pending request
    fn register(&self, correlation_id: &str, timeout_ms: u64) -> Result<(), MessagingError>;
    
    /// Complete a pending request with response
    fn complete(&self, correlation_id: &str, response: MessagePayload) -> Result<(), MessagingError>;
    
    /// Check if a correlation is pending
    fn is_pending(&self, correlation_id: &str) -> bool;
}
```

> **Note:** `MessagePayload` is now imported from `core/component/` instead of `core/messaging/payload.rs`.
> This eliminates the circular dependency between `core/component/` and `core/messaging/`.

---

### core/security/traits.rs

```rust
use crate::core::component::id::ComponentId;
use crate::core::errors::security::SecurityError;
use crate::core::security::capability::Capability;

/// Trait for validating component capabilities
/// Implemented by security/ module
pub trait SecurityValidator: Send + Sync {
    /// Validate if component has required capability
    fn validate_capability(
        &self,
        component: &ComponentId,
        capability: &Capability,
    ) -> Result<(), SecurityError>;
    
    /// Check if component can send message to target
    fn can_send_to(
        &self,
        sender: &ComponentId,
        target: &ComponentId,
    ) -> Result<(), SecurityError>;
}

/// Trait for security audit logging
pub trait SecurityAuditLogger: Send + Sync {
    /// Log a security event
    fn log_event(&self, event: SecurityEvent);
}

/// Security event for audit logging
#[derive(Debug, Clone)]
pub struct SecurityEvent {
    pub component: ComponentId,
    pub action: String,
    pub resource: String,
    pub granted: bool,
    pub timestamp_ms: u64,
}
```

---

### core/security/capability.rs

```rust
/// Capability types for security validation
#[derive(Debug, Clone)]
pub enum Capability {
    Messaging(MessagingCapability),
    Storage(StorageCapability),
    Filesystem(FilesystemCapability),
    Network(NetworkCapability),
}

#[derive(Debug, Clone)]
pub struct MessagingCapability {
    pub action: MessagingAction,
    pub target_pattern: String,
}

#[derive(Debug, Clone)]
pub enum MessagingAction {
    Send,
    Request,
    Broadcast,
}

#[derive(Debug, Clone)]
pub struct StorageCapability {
    pub action: StorageAction,
    pub namespace_pattern: String,
}

#[derive(Debug, Clone)]
pub enum StorageAction {
    Read,
    Write,
    Delete,
}

#[derive(Debug, Clone)]
pub struct FilesystemCapability {
    pub action: FilesystemAction,
    pub path_pattern: String,
}

#[derive(Debug, Clone)]
pub enum FilesystemAction {
    Read,
    Write,
    Delete,
    ListDir,
}

#[derive(Debug, Clone)]
pub struct NetworkCapability {
    pub action: NetworkAction,
    pub host_pattern: String,
    pub port: Option<u16>,
}

#[derive(Debug, Clone)]
pub enum NetworkAction {
    Outbound,
    Inbound,
}
```

---

### core/storage/traits.rs

```rust
use crate::core::component::MessagePayload;
use crate::core::errors::storage::StorageError;

/// Trait for component-isolated storage
/// Implemented by storage system, consumed by components via host functions
pub trait ComponentStorage: Send + Sync {
    /// Get value by key
    fn get(&self, key: &str) -> Result<Option<MessagePayload>, StorageError>;
    
    /// Set value by key
    fn set(&self, key: &str, value: MessagePayload) -> Result<(), StorageError>;
    
    /// Delete value by key
    fn delete(&self, key: &str) -> Result<(), StorageError>;
    
    /// Check if key exists
    fn exists(&self, key: &str) -> Result<bool, StorageError>;
    
    /// List keys with optional prefix
    fn list_keys(&self, prefix: Option<&str>) -> Result<Vec<String>, StorageError>;
}
```

> **Note:** `MessagePayload` is imported from `core/component/` instead of `core/messaging/`.

---

### core/errors/wasm.rs

```rust
use std::fmt;

/// WASM execution errors
#[derive(Debug, Clone)]
pub enum WasmError {
    ComponentNotFound(String),
    InstantiationFailed(String),
    ExportNotFound(String),
    Timeout,
    ResourceLimitExceeded(String),
    InvalidComponent(String),
    RuntimeError(String),
}

impl fmt::Display for WasmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ComponentNotFound(id) => write!(f, "Component not found: {}", id),
            Self::InstantiationFailed(msg) => write!(f, "Instantiation failed: {}", msg),
            Self::ExportNotFound(name) => write!(f, "Export not found: {}", name),
            Self::Timeout => write!(f, "Execution timeout"),
            Self::ResourceLimitExceeded(msg) => write!(f, "Resource limit exceeded: {}", msg),
            Self::InvalidComponent(msg) => write!(f, "Invalid component: {}", msg),
            Self::RuntimeError(msg) => write!(f, "Runtime error: {}", msg),
        }
    }
}

impl std::error::Error for WasmError {}
```

---

### core/errors/security.rs

```rust
use std::fmt;

/// Security-related errors
#[derive(Debug, Clone)]
pub enum SecurityError {
    CapabilityDenied(String),
    PolicyViolation(String),
    InvalidContext(String),
    PermissionDenied(String),
}

impl fmt::Display for SecurityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CapabilityDenied(msg) => write!(f, "Capability denied: {}", msg),
            Self::PolicyViolation(msg) => write!(f, "Policy violation: {}", msg),
            Self::InvalidContext(msg) => write!(f, "Invalid context: {}", msg),
            Self::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
        }
    }
}

impl std::error::Error for SecurityError {}
```

---

### core/errors/messaging.rs

```rust
use std::fmt;

/// Messaging errors
#[derive(Debug, Clone)]
pub enum MessagingError {
    DeliveryFailed(String),
    CorrelationTimeout(String),
    InvalidMessage(String),
    QueueFull,
    TargetNotFound(String),
}

impl fmt::Display for MessagingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DeliveryFailed(msg) => write!(f, "Delivery failed: {}", msg),
            Self::CorrelationTimeout(id) => write!(f, "Correlation timeout: {}", id),
            Self::InvalidMessage(msg) => write!(f, "Invalid message: {}", msg),
            Self::QueueFull => write!(f, "Message queue full"),
            Self::TargetNotFound(id) => write!(f, "Target not found: {}", id),
        }
    }
}

impl std::error::Error for MessagingError {}
```

---

## Dependency Inversion Example

```rust
// component/wrapper.rs - CONSUMER (depends on abstraction)
use crate::core::runtime::traits::RuntimeEngine;

pub struct ComponentWrapper {
    engine: Arc<dyn RuntimeEngine>,  // Injected by system/
}

// runtime/engine.rs - IMPLEMENTATION
use crate::core::runtime::traits::RuntimeEngine;

pub struct WasmtimeEngine { /* ... */ }

impl RuntimeEngine for WasmtimeEngine {
    fn call_handle_message(...) -> Result<...> {
        // Real implementation
    }
}

// system/manager.rs - COORDINATOR (injects concrete)
use crate::runtime::WasmtimeEngine;
use crate::component::ComponentWrapper;

let engine = Arc::new(WasmtimeEngine::new());
let wrapper = ComponentWrapper::new(engine);  // Inject
```

---

## History

| Date | Version | Change |
|------|---------|--------|
| 2026-01-08 | 1.1 | Moved MessagePayload to core/component/message.rs, removed core/messaging/payload.rs |
| 2026-01-05 | 1.0 | Initial core module structure |

---

**This ADR defines the complete core/ module structure for Phase 3 of the rebuild.**
