//! # Messaging Module (Layer 3B)
//!
//! Inter-component communication patterns and message routing.
//!
//! ## Responsibilities
//!
//! - Message types (fire-and-forget, request-response)
//! - Correlation tracking for request-response patterns
//! - Message routing and delivery
//!
//! ## Module Position
//!
//! This is **Layer 3B** of the 6-layer architecture:
//!
//! ```text
//! Layer 4: system/
//!   | imports
//! Layer 3B: messaging/      <-- THIS MODULE
//!   | imports
//! Layer 3A: component/
//! Layer 2: runtime/
//! Layer 1: security/
//! Layer 0: core/
//! ```
//!
//! ## Import Restrictions
//!
//! This module MUST NOT import from:
//! - `component/` (Layer 3A)
//! - `system/` (Layer 4)
//!
//! This module MAY import from:
//! - `core/` (Layer 0)
//! - `security/` (Layer 1)
//! - `runtime/` (Layer 2)
//!
//! ## Architecture References
//!
//! - ADR-WASM-031: Component Messaging Design
//! - ADR-WASM-009: Component Communication Model

pub mod correlation;
pub mod patterns;
pub mod routing;
pub mod types;

// NOTE: No re-exports per module grouping policy.
// Callers use namespaced access: messaging::patterns::FireAndForget
