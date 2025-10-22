//! Core abstractions for airssys-wasm framework.
//!
//! This module contains foundational types, traits, and error definitions
//! used throughout the entire airssys-wasm crate. It has **ZERO internal
//! dependencies** within airssys-wasm to prevent circular dependencies.
//!
//! # Architecture
//!
//! The core module follows a two-tier structure:
//!
//! ## Universal Abstractions (Phase 1-5: Complete)
//! - `component` - Component types, metadata, input/output
//! - `capability` - Capability-based security primitives
//! - `error` - Error types and result aliases
//! - `config` - Configuration types and defaults
//!
//! ## Domain-Specific Abstractions (Phase 6+: Complete through Phase 8)
//! - `runtime` - Runtime engine traits and execution context (Phase 6.1: ✅ Complete)
//! - `interface` - WIT interface metadata and type descriptors (Phase 6.2: ✅ Complete)
//! - `actor` - Actor integration message envelopes (Phase 7.1: ✅ Complete)
//! - `security` - Security policy traits and permission types (Phase 7.2: ✅ Complete)
//! - `messaging` - Inter-component messaging protocols (Phase 8.1: ✅ Complete)
//! - `storage` - Storage backend traits and operations (Phase 8.2: ✅ Complete)
//! - `lifecycle` - Lifecycle state machines and transitions (Phase 9: Planned)
//! - `management` - Component registry and management abstractions (Phase 9: Planned)
//! - `bridge` - OSL bridge traits and capability mapping (Phase 10: Planned)
//! - `observability` - Metrics collection and monitoring traits (Phase 10: Planned)
//!
//! # Design Principles
//!
//! 1. **Zero Internal Dependencies** - Core depends ONLY on external crates
//! 2. **Minimalism (YAGNI)** - Include only types needed by 3+ modules
//! 3. **Type Safety** - Newtype pattern for IDs, enums for variants
//! 4. **Stability First** - Core types rarely change (breaking = major version)
//! 5. **Trait-Centric** - Behavior contracts via traits for testability
//!
//! # References
//!
//! - **ADR-WASM-011**: Module Structure Organization
//! - **ADR-WASM-012**: Comprehensive Core Abstractions Strategy
//! - **Workspace Standards**: §4.3 (Module Architecture), §6.1 (YAGNI)

// Universal Abstractions (Phase 1-5: Complete)
pub mod capability;
pub mod component;
pub mod config;
pub mod error;

// Domain-Specific Abstractions (Phase 6+: Complete through Phase 8)
pub mod interface;
pub mod runtime;
pub mod actor;  // Phase 7.1: Complete
pub mod security;  // Phase 7.2: Complete
pub mod messaging;  // Phase 8.1: Complete
pub mod storage;  // Phase 8.2: Complete

// Future phases (will be uncommented as implemented)
 // Phase 9: lifecycle, management
 // Phase 10: bridge, observability

// Re-exports for public API
pub use capability::{Capability, CapabilitySet, DomainPattern, NamespacePattern, PathPattern, TopicPattern};
pub use component::{Component, ComponentConfig, ComponentId, ComponentInput, ComponentMetadata, ComponentOutput, ComponentState, InstallationSource, ResourceLimits};
pub use config::{RuntimeConfig, SecurityConfig, SecurityMode, StorageBackend as StorageBackendType, StorageConfig};
pub use error::{WasmError, WasmResult};
pub use interface::{FunctionSignature, WitInterface};
pub use runtime::{ComponentHandle, ExecutionContext, ExecutionState, ResourceUsage, RuntimeEngine};
pub use actor::{ActorMessage, ActorMetadata, ActorState, SupervisionStrategy};
pub use security::{IsolationBoundary, PermissionRequest, PermissionResult, SecurityContext, SecurityPolicy, TrustLevel};
pub use messaging::{DeliveryGuarantee, MessageEnvelope, MessageType, RoutingStrategy};
pub use storage::{StorageBackend, StorageOperation, StorageTransaction};
