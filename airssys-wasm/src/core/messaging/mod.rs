//! Messaging abstractions for inter-component communication.
//!
//! This module contains types, traits, and errors for message routing
//! and correlation tracking between WASM components.
//!
//! # Architecture
//!
//! This module is part of the **core/** foundation (Layer 1). It contains:
//!
//! - **Traits**: `MessageRouter`, `CorrelationTracker` (abstractions)
//! - **Types**: `CorrelationId` (data)
//! - **Errors**: `MessagingError` (co-located)
//!
//! Concrete implementations live in the `messaging/` module (Layer 3).
//!
//! # Design
//!
//! The messaging module follows the Dependency Inversion Principle:
//!
//! - Layer 1 (`core/messaging/`): Defines traits and types
//! - Layer 3 (`messaging/`): Implements traits
//! - Layer 4 (`system/`): Injects implementations
//!
//! # Submodules
//!
//! - [`correlation`] - `CorrelationId` type for request-response tracking
//! - [`errors`] - `MessagingError` enum (co-located with messaging)
//! - [`traits`] - `MessageRouter` and `CorrelationTracker` traits
//!
//! # Usage
//!
//! ```rust
//! use airssys_wasm::core::messaging::correlation::CorrelationId;
//! use airssys_wasm::core::messaging::errors::MessagingError;
//! use airssys_wasm::core::messaging::traits::{MessageRouter, CorrelationTracker};
//!
//! // Generate a new correlation ID
//! let correlation_id = CorrelationId::generate();
//!
//! // Create an error
//! let error = MessagingError::TargetNotFound("app/service/001".to_string());
//! ```

// Module declarations (per PROJECTS_STANDARD.md §4.3)
pub mod correlation;
pub mod errors;
pub mod traits;

// NOTE: No glob re-exports per module grouping policy.
// Callers use namespaced access: core::messaging::correlation::CorrelationId
