//! # Security Module
//!
//! Core security abstractions for capability-based security.
//!
//! This module provides the foundation types and traits for the security system:
//! - [`capability`] - Capability types (Messaging, Storage, Filesystem, Network)
//! - [`errors`] - Security error types
//! - [`traits`] - Security validation and audit logging traits
//!
//! # Architecture
//!
//! This is part of **Layer 1** (core/). The security module contains ONLY
//! abstractions and types. Actual security enforcement is implemented in
//! the `security/` module (Layer 2A).
//!
//! # Usage
//!
//! ```rust
//! use airssys_wasm::core::security::capability::{Capability, MessagingCapability, MessagingAction};
//! use airssys_wasm::core::security::errors::SecurityError;
//! use airssys_wasm::core::security::traits::{SecurityValidator, SecurityEvent};
//! ```

// Module declarations (per PROJECTS_STANDARD.md §4.3)
pub mod capability;
pub mod errors;
pub mod traits;

// NOTE: No glob re-exports (pub use X::*) per module grouping policy.
// Callers use namespaced access: core::security::capability::Capability
