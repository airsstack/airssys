//! Core abstractions for the airssys-wasm framework.
//!
//! This module provides foundational types, traits, and error definitions used
//! throughout the airssys-wasm crate. It maintains **zero internal dependencies**
//! within airssys-wasm to prevent circular dependency issues.
//!
//! # Architecture Overview
//!
//! The core module implements a two-tier architecture separating universal
//! abstractions from domain-specific contracts. This design follows the
//! principles documented in ADR-WASM-012 (Comprehensive Core Abstractions).
//!
//! ## Tier 1: Universal Abstractions
//!
//! Foundation types used across all framework layers:
//!
//! - **[`component`]** - Component identification, metadata, I/O types, and state management
//! - **[`capability`]** - Capability-based security primitives and pattern matching
//! - **[`error`]** - Comprehensive error types with context and error handling utilities
//! - **[`config`]** - Runtime, security, and storage configuration types
//!
//! ## Tier 2: Domain-Specific Abstractions
//!
//! Trait contracts defining behavior for specialized subsystems:
//!
//! ### Runtime & Interface (Phase 6)
//! - **[`runtime`]** - Runtime engine traits, execution context, and resource tracking
//! - **[`interface`]** - WIT interface metadata and WebAssembly type descriptors
//!
//! ### Actor & Security (Phase 7)
//! - **[`actor`]** - Actor model integration with message envelopes and supervision
//! - **[`security`]** - Security policy contracts, isolation boundaries, and trust levels
//!
//! ### Messaging & Storage (Phase 8)
//! - **[`messaging`]** - Inter-component messaging protocols with delivery guarantees
//! - **[`storage`]** - Storage backend abstractions for component persistence
//!
//! ### Lifecycle & Management (Phase 9)
//! - **[`lifecycle`]** - Component lifecycle state machines and update strategies
//! - **[`management`]** - Component registry and installation metadata management
//!
//! ### Bridge & Observability (Phase 10)
//! - **[`bridge`]** - Host function traits and capability mapping for OSL integration
//! - **[`observability`]** - Metrics collection, health monitoring, and event tracking
//!
//! # Usage Examples
//!
//! ## Basic Component Definition
//!
//! ```rust
//! use airssys_wasm::core::{Component, ComponentId, ComponentMetadata, ComponentConfig};
//! use airssys_wasm::core::{ComponentInput, ComponentOutput, WasmError, ResourceLimits};
//! use std::collections::HashMap;
//!
//! struct MyComponent {
//!     metadata: ComponentMetadata,
//! }
//!
//! impl Component for MyComponent {
//!     fn init(&mut self, config: ComponentConfig) -> Result<(), WasmError> {
//!         // Component initialization logic
//!         Ok(())
//!     }
//!
//!     fn execute(&self, input: ComponentInput) -> Result<ComponentOutput, WasmError> {
//!         // Component execution logic
//!         Ok(ComponentOutput {
//!             data: Vec::new(),
//!             codec: 0x0200, // JSON multicodec
//!             metadata: HashMap::new(),
//!         })
//!     }
//!
//!     fn shutdown(&mut self) -> Result<(), WasmError> {
//!         // Cleanup logic
//!         Ok(())
//!     }
//!
//!     fn metadata(&self) -> &ComponentMetadata {
//!         &self.metadata
//!     }
//! }
//! ```
//!
//! ## Capability-Based Security
//!
//! ```rust
//! use airssys_wasm::core::{Capability, CapabilitySet, PathPattern, DomainPattern};
//!
//! let mut caps = CapabilitySet::new();
//! caps.grant(Capability::FileRead(PathPattern::new("/data/config.json")));
//! caps.grant(Capability::FileWrite(PathPattern::new("/data/output.log")));
//! caps.grant(Capability::NetworkOutbound(DomainPattern::new("api.example.com")));
//!
//! assert!(caps.has(&Capability::FileRead(PathPattern::new("/data/config.json"))));
//! ```
//!
//! ## Runtime Configuration
//!
//! ```rust
//! use airssys_wasm::core::{RuntimeConfig, SecurityConfig, SecurityMode};
//!
//! let config = RuntimeConfig {
//!     default_max_fuel: 2_000_000,
//!     default_execution_timeout_ms: 200,
//!     ..Default::default()
//! };
//!
//! let security = SecurityConfig {
//!     mode: SecurityMode::Strict,
//!     ..Default::default()
//! };
//! ```
//!
//! ## Error Handling
//!
//! ```rust
//! use airssys_wasm::core::{WasmError, WasmResult};
//!
//! fn load_component(id: &str) -> WasmResult<()> {
//!     if id.is_empty() {
//!         return Err(WasmError::component_not_found("Component ID cannot be empty"));
//!     }
//!     Ok(())
//! }
//! ```
//!
//! # Design Principles
//!
//! This module adheres to strict design principles for maintainability:
//!
//! 1. **Zero Internal Dependencies** - Core module depends only on external crates
//!    (std, serde, chrono, thiserror). No dependencies on other airssys-wasm modules.
//!
//! 2. **YAGNI (You Aren't Gonna Need It)** - Types included only when needed by
//!    three or more modules, preventing speculative abstractions (§6.1).
//!
//! 3. **Type Safety First** - Newtype pattern for identifiers, enums for variants,
//!    compile-time guarantees over runtime checks.
//!
//! 4. **Stability Contract** - Core types change rarely. Breaking changes require
//!    major version increment. Backward compatibility prioritized.
//!
//! 5. **Trait-Centric Design** - Behavior defined through trait contracts enabling
//!    testing, mocking, and multiple implementations (Microsoft Guidelines M-MOCKABLE-SYSCALLS).
//!
//! 6. **Import Organization** - Follows §2.1 three-layer import structure:
//!    standard library, external crates, internal modules.
//!
//! # Module Organization
//!
//! Per workspace standard §4.3, this `mod.rs` file contains **only** module
//! declarations and re-exports. Implementation code resides in individual module files.
//!
//! # References
//!
//! - **ADR-WASM-011**: Module Structure Organization
//! - **ADR-WASM-012**: Comprehensive Core Abstractions Strategy
//! - **Workspace Standards**: §2.1 (Imports), §4.3 (Modules), §6.1 (YAGNI), §6.2 (Avoid dyn)
//! - **Microsoft Rust Guidelines**: M-DESIGN-FOR-AI, M-DI-HIERARCHY, M-MOCKABLE-SYSCALLS

// Universal Abstractions (Phase 1-5: Complete)
pub mod capability;
pub mod component;
pub mod config;
pub mod error;
pub mod multicodec;  // Phase 1 Task 1.3: Multicodec support for message serialization
pub mod rate_limiter;  // DEBT-WASM-004 Item #3: Rate limiting for message security

// Permission System (WASM-TASK-003 Phase 3 Task 3.2: Complete)
pub mod permission;
pub mod manifest;
pub mod permission_checker;
pub mod permission_wit;  // WIT Permission Integration Layer

// Domain-Specific Abstractions (Phase 6+: Complete through Phase 9)
pub mod interface;
pub mod runtime;
pub mod actor;  // Phase 7.1: Complete
pub mod security;  // Phase 7.2: Complete
pub mod messaging;  // Phase 8.1: Complete
pub mod storage;  // Phase 8.2: Complete
pub mod lifecycle;  // Phase 9.1: Complete
pub mod management;  // Phase 9.2: Complete
pub mod bridge;  // Phase 10.1: Complete
pub mod observability;  // Phase 10.2: Complete

// Re-exports for public API
pub use capability::{Capability, CapabilitySet, DomainPattern, NamespacePattern, PathPattern, TopicPattern};
pub use component::{Component, ComponentConfig, ComponentId, ComponentInput, ComponentMetadata, ComponentOutput, ComponentState, InstallationSource, ResourceLimits};
pub use config::{
    ComponentConfigToml, ComponentMetadataToml, ConfigError, CpuConfigToml, MemoryConfigToml,
    ResourcesConfigToml, RuntimeConfig, SecurityConfig, SecurityMode,
    StorageBackend as StorageBackendType, StorageConfig,
    DEFAULT_MAX_MESSAGE_SIZE,
};
pub use error::{WasmError, WasmResult};
pub use multicodec::{Codec, decode_multicodec, encode_multicodec};
pub use rate_limiter::{MessageRateLimiter, RateLimiterConfig, DEFAULT_RATE_LIMIT};
pub use permission::{PermissionManifest, FilesystemPermissions, NetworkPermissions, NetworkEndpoint, StoragePermissions};
pub use manifest::{ComponentManifest, PackageInfo, RuntimeConfig as ManifestRuntimeConfig};
pub use permission_checker::PermissionChecker;
pub use permission_wit::{
    WitComponentId, WitPermissionManifest, WitFilesystemPermissions, WitNetworkPermissions,
    WitNetworkEndpoint, WitStoragePermissions, WitPermissionResult,
    check_file_read_wit, check_file_write_wit, check_file_delete_wit,
    check_directory_list_wit, check_network_outbound_wit, check_storage_access_wit,
};
pub use interface::{FunctionSignature, WitInterface};
pub use runtime::{ComponentHandle, ExecutionContext, ExecutionState, ResourceUsage, RuntimeEngine};
pub use actor::{ActorMessage, ActorMetadata, ActorState, SupervisionStrategy};
pub use security::{IsolationBoundary, PermissionRequest, PermissionResult, SecurityContext, SecurityPolicy, TrustLevel};
pub use messaging::{DeliveryGuarantee, MessageEnvelope, MessageType};
pub use storage::{StorageBackend, StorageOperation};
pub use lifecycle::{LifecycleEvent, LifecycleState, UpdateStrategy, VersionInfo};
pub use management::{ComponentQuery, ComponentRegistry, InstallationMetadata, RegistryOperation};
pub use bridge::{CapabilityMapping, HostCallContext, HostFunction, HostFunctionCategory};
pub use observability::{EventSeverity, HealthStatus, Metric, MetricType, MetricsCollector, MetricsSnapshot, ObservabilityEvent};
