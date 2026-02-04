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
//! The crate is organized into six root modules:
//!
//! - **core/** - Foundation: shared types and abstractions (imports nothing internal)
//! - **security/** - Security: capabilities, policies, validation (imports core/)
//! - **runtime/** - WASM Execution: Wasmtime integration (imports core/, security/)
//! - **component/** - Component system: lifecycle, supervision, orchestration (imports core/, security/, runtime/)
//! - **messaging/** - Inter-component communication: message types, routing, correlation (imports core/, security/, runtime/)
//! - **system/** - Top-level runtime management: lifecycle, configuration (imports all lower layers)
//!
//! ## Module Architecture
//!
//! ```text
//! Layer 4: system/
//!   ↓ imports
//! Layer 3B: messaging/
//!   ↓ imports
//! Layer 3A: component/
//!   ↓ imports
//! Layer 2: runtime/
//!   ↓ imports
//! Layer 1: security/
//!   ↓ imports
//! Layer 0: core/
//! ```
//!
//! Import restrictions are enforced by ADR-WASM-023.
//!
//! ## Getting Started
//!
//! ```rust,no_run
//! use airssys_wasm::prelude::*;
//!
//! // Components will be added in subsequent tasks
//! ```
//!
//! ## Module Documentation
//!
//! - [core] - Core types and abstractions
//! - [security] - Security capabilities and policies
//! - [runtime] - WASM execution engine
//! - [component] - Component system integration
//! - [messaging] - Inter-component communication
//! - [system] - Runtime management and lifecycle

// Layer 1: Standard library imports (per PROJECTS_STANDARD.md §2.1)

// Layer 2: Third-party crate imports (per PROJECTS_STANDARD.md §2.1)

// Layer 3: Internal module imports (per PROJECTS_STANDARD.md §2.1)

// ============================================================================
// WIT Bindings Generation (Phase 1, Task 12 - WASM-TASK-012)
// ============================================================================
//
// Generate Rust bindings for HOST-SIDE implementation using wasmtime's
// Component Model bindgen macro.
//
// ⚠️ IMPORTANT: This uses `wasmtime::component::bindgen!` NOT `wit_bindgen::generate!`
//
// - `wasmtime::component::bindgen!` = HOST-SIDE bindings (what we need here)
// - `wit_bindgen::generate!` = GUEST-SIDE bindings (for WASM components)
//
// For detailed explanation of the differences, see:
// - KNOWLEDGE-WASM-043: wit_bindgen vs wasmtime::component::bindgen!
//
// ## What This Macro Generates
//
// The `wasmtime::component::bindgen!` macro generates Rust bindings from the
// WIT world definition in `wit/core/world.wit`:
//
// 1. **RuntimeHost Module** with `add_to_linker()` helper function:
//    - Automatically registers ALL 18 host functions with wasmtime Linker
//    - One-line registration: `RuntimeHost::add_to_linker(linker, |state| state)`
//
// 2. **Host Trait Implementations** for imported interfaces:
//    - `airssys::core::host_messaging::Host` - 5 messaging functions
//    - `airssys::core::host_services::Host` - 6 service functions
//    - `airssys::core::storage::Host` - 6 storage functions
//    - These traits MUST be implemented on `HostState` in runtime/host_functions.rs
//
// 3. **Type Definitions** from WIT files:
//    - `ComponentId`, `MessagePayload`, `CorrelationId`, `Timestamp`, etc.
//    - All WIT records → Rust structs
//    - All WIT variants → Rust enums
//    - All WIT errors → Rust error types
//
// 4. **Export Types** for calling component functions:
//    - Typed function getters for component-lifecycle exports
//    - Type-safe bindings for `init`, `handle-message`, `handle-callback`
//
// ## Generated Code Location
//
// The macro generates code in-line at this location. The generated types are
// accessible throughout the crate as:
//
// ```rust
// use crate::RuntimeHost;  // For add_to_linker()
// use crate::airssys::core::host_messaging;  // For Host traits
// use crate::airssys::core::types::{ComponentId, MessagePayload};  // For types
// ```
//
// ## Why Not wit_bindgen::generate!?
//
// The standalone `wit_bindgen::generate!` macro is for GUEST components (WASM code).
// It generates:
// - Import functions as callable functions (not traits to implement)
// - Export interfaces as traits for the component to implement
// - No `add_to_linker()` helper (guests don't register functions)
//
// For hosts implementing imported functions, we MUST use `wasmtime::component::bindgen!`
// which provides the `RuntimeHost::add_to_linker()` automatic registration helper.
//
// ## Implementation Guide
//
// To implement host functions:
// 1. See `runtime/host_functions.rs` for trait implementations
// 2. Each imported WIT interface becomes a `Host` trait
// 3. Implement the trait on `HostState`
// 4. Registration is automatic via `RuntimeHost::add_to_linker()`
//
// For complete documentation, see KNOWLEDGE-WASM-043.
//
// ## Next Steps
//
// This completes Phase 1 (WIT Interface System). The generated bindings
// are now available for:
// - Phase 2 (Project Restructuring)
// - Phase 3 (Core Module implementation)
// - Phase 5 (Runtime Module implementation)

wasmtime::component::bindgen!({
    world: "runtime-host",
    path: "wit/core",
});

// Layer 0: Foundation types and abstractions
pub mod core;

// Layer 1: Security and permissions
pub mod security;

// Layer 2: WASM execution engine
pub mod runtime;

// Layer 3A: Component actor system
pub mod component;

// Layer 3B: Inter-component communication
pub mod messaging;

// Layer 4: System-level runtime management
pub mod system;

// Prelude - common re-exports for ergonomic API (per ADR-WASM-011)
pub mod prelude;

// Re-export core error types at crate root (will be populated in subsequent tasks)
