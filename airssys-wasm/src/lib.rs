//! # airssys-wasm - WASM Plugin/Extension Platform for AirsStack
//!
//! This crate provides a WebAssembly-based plugin/extension platform where:
//! - Each WASM component is treated as an Actor (via airssys-rt)
//! - Components have isolated storage (like smart contracts on NEAR/Polkadot)
//! - Communication happens via Erlang-style mailbox messaging
//! - Security follows deny-by-default capability model
//!
//! ## Architecture
//!
//! The crate is organized into four root modules:
//!
//! - **core/** - Foundation: shared types and abstractions (imports nothing internal)
//! - **security/** - Security: capabilities, policies, validation (imports core/)
//! - **runtime/** - WASM Execution: Wasmtime integration (imports core/, security/)
//! - **actor/** - Integration: actor system, messaging, lifecycle (imports core/, security/, runtime/)
//!
//! ## Dependency Rules (ADR-WASM-023 - MANDATORY)
//!
//! ```
//! actor/ ──► runtime/ ──► security/ ──► core/
//!    │            │              │               │
//!    └────────────┴──────────────┴───────────────┘
//!                      All can import from core/
//! ```
//!
//! **ALLOWED:**
//! - ✅ actor/    → runtime/
//! - ✅ actor/    → security/
//! - ✅ actor/    → core/
//! - ✅ runtime/  → security/
//! - ✅ runtime/  → core/
//! - ✅ security/ → core/
//!
//! **FORBIDDEN (never):**
//! - ❌ runtime/  → actor/
//! - ❌ security/ → actor/
//! - ❌ security/ → runtime/
//! - ❌ core/     → ANY MODULE
//!
//! ## Getting Started
//!
//! ```rust,no_run
//! use airssys_wasm::prelude::*;
//! use airssys_wasm::actor::ComponentActor;
//!
//! // Components will be added in subsequent tasks
//! ```
//!
//! ## Module Documentation
//!
//! - [core](core) - Core types and abstractions
//! - [security](security) - Security capabilities and policies
//! - [runtime](runtime) - WASM execution engine
//! - [actor](actor) - Actor system integration

// Layer 1: Standard library imports (per PROJECTS_STANDARD.md §2.1)

// Layer 2: Third-party crate imports (per PROJECTS_STANDARD.md §2.1)

// Layer 3: Internal module imports (per PROJECTS_STANDARD.md §2.1)

// ============================================================================
// WIT Bindings Generation (Phase 1, Task 12 - WASM-TASK-012)
// ============================================================================
// Generate Rust bindings from WIT interfaces (macro-based, no build.rs)
// Reference: ADR-WASM-027 lines 506-515, KNOWLEDGE-WASM-037 lines 265-273
//
// The `wit_bindgen::generate!` macro generates Rust bindings for the WIT
// interfaces defined in `wit/core/`. These bindings provide types and traits
// that will be used throughout the airssys-wasm crate.
//
// **Generated Types Location:**
// The macro generates code in-line at this location. The generated types are
// accessible from anywhere in the crate.
//
// **Key Generated Modules and Types:**
//
// The `wit_bindgen::generate!` macro generates bindings based on the WIT world
// definition in `wit/core/world.wit`. The generated code includes:
//
// 1. **Type Definitions** (from `types.wit`):
//    - `ComponentId` - Unique component identifier with namespace, name, instance
//    - `ComponentHandle` - Opaque runtime handle (u64)
//    - `CorrelationId` - Correlation ID for request-response (String)
//    - `MessagePayload` - Raw bytes for message content (Vec<u8>)
//    - `Timestamp` - High-precision timestamp
//    - `MessageMetadata` - Message metadata with correlation-id, reply-to, etc.
//    - `ComponentMessage` - Complete message envelope
//    - `ResourceLimits` - Execution resource constraints
//    - `ComponentConfig` - Initialization configuration
//    - `LogLevel` - Logging level enum (trace, debug, info, warn, error)
//    - `HealthStatus` - Health status enum
//    - `ExecutionStatus` - Execution status enum
//
// 2. **Error Types** (from `errors.wit`):
//    - `WasmError` - WASM execution errors
//    - `ComponentError` - Component lifecycle errors
//    - `SecurityError` - Security-related errors
//    - `MessagingError` - Messaging errors
//    - `StorageError` - Storage errors
//    - `ExecutionError` - RPC execution errors
//
// 3. **Capability Types** (from `capabilities.wit`):
//    - `FilesystemPermission` - Filesystem access permissions
//    - `NetworkPermission` - Network access permissions
//    - `StoragePermission` - Storage access permissions
//    - `MessagingPermission` - Messaging permissions
//    - `RequestedPermissions` - Complete permission set
//    - `CapabilityGrant` - Granted capability result
//
// 4. **Host Interfaces** (imported by components):
//    - `HostMessaging` - Inter-component messaging functions
//    - `HostServices` - General host services (logging, time, etc.)
//    - `Storage` - Component-isolated storage functions
//
// 5. **Guest Interface** (exported by components):
//    - `GuestComponentLifecycle` - Component lifecycle functions
//    - `ComponentMetadata` - Component metadata structure
//
// **Accessing Generated Types:**
//
// Generated types are available directly from the crate root. Examples:
//
// ```rust
// use airssys_wasm::ComponentId;
// use airssys_wasm::ComponentMessage;
// use airssys_wasm::WasmError;
// use airssys_wasm::Capabilities;
//
// // Create component ID
// let id = ComponentId {
//     namespace: "example".to_string(),
//     name: "my-component".to_string(),
//     instance: "v1".to_string(),
// };
// ```
//
// **Integration with Core Module:**
//
// In Phase 3 (WASM-TASK-017 onwards), these generated types will be:
// - Re-exported from appropriate core/ submodules
// - Used as foundations for airssys-wasm's type system
// - Integrated with custom error types and traits
//
// **Phase 2 Preparation:**
//
// This completes Phase 1 (WIT Interface System). The generated bindings
// are now available for Phase 2 (Project Restructuring) and Phase 3
// (Core Module implementation).

wit_bindgen::generate!({
    world: "component",
    path: "wit/core",
});

// Foundation layer (no internal dependencies)
pub mod core;

// Security layer (imports from core/)
pub mod security;

// WASM execution layer (imports from core/, security/)
pub mod runtime;

// Actor integration layer (imports from core/, security/, runtime/)
pub mod actor;

// Prelude - common re-exports for ergonomic API (per ADR-WASM-011)
pub mod prelude;

// Re-export core error types at crate root (will be populated in subsequent tasks)
