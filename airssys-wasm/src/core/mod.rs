//! # Core Module
//!
//! Core data types and abstractions shared by ALL other modules.
//!
//! This module contains foundational types that prevent circular dependencies.
//! Any type that multiple modules need should be defined here.
//!
//! # Submodules
//!
//! - [`component`] - Component-related types (ComponentId, ComponentHandle, ComponentMessage, ComponentLifecycle)
//!
//! # Architecture
//!
//! This is **Layer 1** of the architecture. Core has zero internal dependencies.
//! All other modules (security/, runtime/, component/, messaging/, system/) depend on core/.
//!
//! # Usage
//!
//! ```
//! use airssys_wasm::core::component::id::ComponentId;
//! use airssys_wasm::core::component::message::ComponentMessage;
//! ```

// Module declarations (per PROJECTS_STANDARD.md §4.3)
pub mod component;

// NOTE: No glob re-exports (pub use X::*) per module grouping policy.
// Callers use namespaced access: core::component::id::ComponentId
