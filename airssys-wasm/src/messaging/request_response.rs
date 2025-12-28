//! Request-response messaging pattern.
//!
//! This module provides request-response messaging capabilities
//! with correlation tracking and response routing.
//!
//! # Current Implementation
//!
//! RequestError is currently defined here and used throughout the messaging
//! system. ResponseRouter (in router.rs) handles the actual routing of
//! request-response messages.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │         Request-Response Message     │
//! │  • Correlation tracking              │
//! │  • Response expected                │
//! │  • Timeout handling                 │
//! └─────────────────────────────────────────┘
//!           ↓ handled by
//! ┌─────────────────────────────────────────┐
//! │        ResponseRouter              │
//! │  • Matches responses to requests   │
//! │  • Delivers via CorrelationTracker│
//! └─────────────────────────────────────────┘
//! ```
//!
//! # Usage Pattern
//!
//! ```rust,ignore
//! use airssys_wasm::messaging::RequestResponse;
//! use airssys_wasm::messaging::RequestError;
//!
//! // Send request with correlation tracking
//! match RequestResponse::send(
//!     ComponentId::new("recipient"),
//!     b"request",
//! ).await {
//!     Ok(response) => println!("Response: {:?}", response),
//!     Err(RequestError::Timeout) => println!("Request timed out"),
//!     Err(e) => println!("Error: {}", e),
//! }
//! ```
//!
//! # References
//!
//! - **KNOWLEDGE-WASM-005**: Messaging Architecture
//! - **KNOWLEDGE-WASM-029**: Messaging Patterns (Pattern 2: Request-Response)

// Layer 1: Standard library imports
use thiserror::Error;

// ============================================================================
// TESTING STRATEGY
// ============================================================================
//
// This module contains RequestError type and placeholder RequestResponse type.
// RequestError is actively used throughout the messaging system.
// RequestResponse is a placeholder for future Phase 2+ work.
//
// Current tests verify:
// - Error types have correct Display impls
// - Types can be instantiated
// - Types have Default impls
//
// Phase 2 will add:
// - Functional tests for RequestResponse
// - Integration tests with ResponseRouter
// - Integration tests with MessageBroker
// ============================================================================

/// Error type for request-response messaging.
#[derive(Debug, Error)]
pub enum RequestError {
    #[error("Request timed out")]
    Timeout,
    #[error("Component not found: {0}")]
    ComponentNotFound(String),
    #[error("Response routing failed: {0}")]
    RoutingFailed(String),
}

/// Request-response messaging handler (Phase 2+).
#[derive(Debug, Clone)]
pub struct RequestResponse {
    _inner: (), // Placeholder
}

impl RequestResponse {
    /// Create a new request-response handler.
    pub fn new() -> Self {
        Self { _inner: () }
    }
}

impl Default for RequestResponse {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::too_many_arguments,
    clippy::type_complexity,
    reason = "test code"
)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_response_creation() {
        let _handler = RequestResponse::new();
        // Functional testing in Phase 2
    }

    #[test]
    fn test_request_response_default() {
        let _handler = RequestResponse::default();
        // Functional testing in Phase 2
    }

    #[test]
    fn test_request_error_display() {
        let err = RequestError::Timeout;
        let msg = format!("{}", err);
        assert!(msg.contains("timed out"));
    }
}
