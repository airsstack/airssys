//! Messaging patterns for component communication.
//!
//! Provides high-level messaging patterns that abstract over
//! message routing and correlation:
//!
//! - [`fire_and_forget::FireAndForget`]: Send message without waiting for response
//! - [`request_response::RequestResponse`]: Send request and correlate response
//!
//! # Architecture
//!
//! This module is part of `messaging/` (Layer 3B). The patterns use
//! trait abstractions from `core/messaging/traits`:
//! - [`crate::core::messaging::traits::MessageSender`]: Routes messages to target components
//! - [`crate::core::messaging::traits::CorrelationManager`]: Registers and tracks pending requests
//!
//! Concrete trait implementations are provided by higher layers (e.g., `system/`),
//! and tests use mock implementations.
//!
//! # References
//!
//! - ADR-WASM-031: Component & Messaging Module Design
//! - ADR-WASM-009: Component Communication Model
//! - KNOWLEDGE-WASM-040: Messaging Module Comprehensive Reference

pub mod fire_and_forget;
pub mod request_response;
