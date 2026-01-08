//! # Core Module
//!
//! Core data types and abstractions shared by ALL other modules.
//!
//! This module contains foundational types that prevent circular dependencies.
//! Any type that multiple modules need should be defined here.
//!
//! # Submodules
//!
//! - **component/** - Component-related types (ComponentId, ComponentHandle, ComponentMessage, ComponentLifecycle)
//!
//! # Architecture
//!
//! This is **Layer 1** of the architecture. Core imports NOTHING except `std`.
//! All other modules (security/, runtime/, component/, messaging/, system/) depend on core/.

// Module declarations (per PROJECTS_STANDARD.md §4.3)
pub mod component;

// Re-exports for ergonomic API (per PROJECTS_STANDARD.md §4.3)
pub use component::*;
