//! # Messaging Module (Layer 3B)
//!
//! Inter-component communication patterns and message routing.
//!
//! ## Responsibilities
//!
//! - Message types (fire-and-forget, request-response)
//! - Correlation tracking for request-response patterns
//! - Message routing via ResponseRouter
//! - Mailbox management via ComponentSubscriber
//!
//! ## Module Position
//!
//! This is **Layer 3B** of the 6-layer architecture:
//!
//! ```text
//! Layer 4: system/
//!   | imports
//! Layer 3B: messaging/      <-- THIS MODULE
//! Layer 3A: component/
//! Layer 2: runtime/
//! Layer 1: security/
//! Layer 0: core/
//! ```
//!
//! ## Import Restrictions
//!
//! This module MUST NOT import from:
//! - `component/` (Layer 3A peer -- depends on core/ traits instead)
//! - `runtime/` (Layer 2B)
//! - `security/` (Layer 2A)
//! - `system/` (Layer 4)
//!
//! This module MAY import from:
//! - `core/` (Layer 0) -- traits and types only
//! - `airssys-rt` (external crate)
//!
//! ## Architecture References
//!
//! - ADR-WASM-031: Component Messaging Design
//! - ADR-WASM-009: Component Communication Model
//! - KNOWLEDGE-WASM-037: Dependency Inversion Principle

pub mod correlation;
pub mod patterns;
pub mod router;
pub mod subscriber;

// NOTE: No re-exports per PROJECTS_STANDARD.md section 4.3.
// Callers use namespaced access: messaging::router::ResponseRouter
