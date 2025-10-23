//! Prelude module for convenient imports.
//!
//! This module re-exports the most commonly used types and traits from the
//! airssys-wasm framework, allowing users to import frequently needed items
//! with a single use statement.
//!
//! # Usage
//!
//! ```rust
//! use airssys_wasm::prelude::*;
//!
//! // Now you have access to core types:
//! let id = ComponentId::new("my-component");
//! let mut caps = CapabilitySet::new();
//! caps.grant(Capability::FileRead(PathPattern::new("/data/config.json")));
//! ```
//!
//! # What's Included
//!
//! The prelude includes:
//!
//! - **Component Types**: [`ComponentId`], [`ComponentMetadata`], [`ComponentConfig`],
//!   [`ComponentInput`], [`ComponentOutput`], [`ComponentState`], [`ResourceLimits`]
//! - **Component Trait**: [`Component`] (core trait for component implementations)
//! - **Capability Types**: [`Capability`], [`CapabilitySet`], pattern types for matching
//! - **Error Handling**: [`WasmError`], [`WasmResult`] (standard error and result types)
//! - **Configuration**: [`RuntimeConfig`], [`SecurityConfig`], [`SecurityMode`], [`StorageConfig`]
//! - **Runtime Traits**: [`RuntimeEngine`], [`ExecutionContext`], [`ComponentHandle`]
//! - **Security Traits**: [`SecurityPolicy`], [`SecurityContext`], [`PermissionRequest`], [`TrustLevel`]
//! - **Lifecycle Types**: [`LifecycleState`], [`LifecycleEvent`], [`UpdateStrategy`], [`VersionInfo`]
//! - **Management Traits**: [`ComponentRegistry`], [`ComponentQuery`], [`RegistryOperation`]
//! - **Messaging Types**: [`MessageEnvelope`], [`MessageType`], [`DeliveryGuarantee`]
//! - **Storage Traits**: [`StorageBackend`], [`StorageOperation`]
//! - **Bridge Traits**: [`HostFunction`], [`CapabilityMapping`], [`HostCallContext`]
//! - **Observability Traits**: [`MetricsCollector`], [`Metric`], [`HealthStatus`], [`ObservabilityEvent`]
//!
//! # Design Rationale
//!
//! The prelude follows these principles:
//!
//! 1. **High-Frequency Items Only** - Includes types and traits used in 80%+ of
//!    user code, avoiding rarely-used items that clutter the namespace.
//!
//! 2. **No Name Conflicts** - Carefully curated to avoid ambiguous names that
//!    might conflict with standard library or common crate imports.
//!
//! 3. **Trait-First** - Includes trait contracts that define component behavior,
//!    enabling idiomatic Rust patterns.
//!
//! 4. **Opt-In Convenience** - Users can choose selective imports or prelude wildcard
//!    based on their preference. The prelude is never mandatory.
//!
//! # When Not to Use
//!
//! Avoid glob imports from the prelude in these situations:
//!
//! - Library crates that want explicit dependency visibility
//! - Code requiring precise control over imported names
//! - Contexts where name conflicts are likely
//!
//! In these cases, use selective imports from `airssys_wasm::core::*` modules.
//!
//! # References
//!
//! - **Workspace Standards**: §4.3 (Module Architecture)
//! - **Microsoft Rust Guidelines**: M-DESIGN-FOR-AI (Idiomatic APIs)

// Layer 1: Standard library imports
// (none required for re-exports)

// Layer 2: External crate imports
// (none required for re-exports)

// Layer 3: Internal module imports

// Core abstractions
pub use crate::core::{
    // Universal Abstractions
    
    // Component types and trait
    Component,
    ComponentConfig,
    ComponentId,
    ComponentInput,
    ComponentMetadata,
    ComponentOutput,
    ComponentState,
    InstallationSource,
    ResourceLimits,
    
    // Capability-based security
    Capability,
    CapabilitySet,
    DomainPattern,
    NamespacePattern,
    PathPattern,
    TopicPattern,
    
    // Error handling
    WasmError,
    WasmResult,
    
    // Configuration types
    RuntimeConfig,
    SecurityConfig,
    SecurityMode,
    StorageBackendType,
    StorageConfig,
    
    // Domain-Specific Abstractions
    
    // Runtime & Interface
    ComponentHandle,
    ExecutionContext,
    ExecutionState,
    FunctionSignature,
    ResourceUsage,
    RuntimeEngine,
    WitInterface,
    
    // Actor integration
    ActorMessage,
    ActorMetadata,
    ActorState,
    SupervisionStrategy,
    
    // Security
    IsolationBoundary,
    PermissionRequest,
    PermissionResult,
    SecurityContext,
    SecurityPolicy,
    TrustLevel,
    
    // Messaging
    DeliveryGuarantee,
    MessageEnvelope,
    MessageType,
    
    // Storage
    StorageBackend,
    StorageOperation,
    
    // Lifecycle
    LifecycleEvent,
    LifecycleState,
    UpdateStrategy,
    VersionInfo,
    
    // Management
    ComponentQuery,
    ComponentRegistry,
    InstallationMetadata,
    RegistryOperation,
    
    // Bridge
    CapabilityMapping,
    HostCallContext,
    HostFunction,
    HostFunctionCategory,
    
    // Observability
    EventSeverity,
    HealthStatus,
    Metric,
    MetricType,
    MetricsCollector,
    MetricsSnapshot,
    ObservabilityEvent,
};

// Runtime implementation (Block 1 - WASM-TASK-002)
pub use crate::runtime::{ComponentLoader, WasmEngine};
