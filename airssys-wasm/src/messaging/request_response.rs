//! Request-response messaging pattern.
//!
//! This module provides request-response messaging capabilities
//! with correlation tracking and response routing.

use thiserror::Error;

// ============================================================================
// TESTING STRATEGY
// ============================================================================
//
// This module contains placeholder types for request-response messaging.
// Full functional testing will be implemented in Task 1.2 when actual
// implementation is moved from runtime/messaging.rs.
//
// Current tests verify:
// - Error types have correct Display impls
// - Types can be instantiated
// - Types have Default impls
//
// Task 1.2 will add:
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

// Placeholder type definitions

/// Request-response messaging handler.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "Placeholder - to be implemented in Task 1.2"]
    fn test_request_response_creation() {
        let _handler = RequestResponse::new();
        // Functional testing in Task 1.2
    }

    #[test]
    #[ignore = "Placeholder - to be implemented in Task 1.2"]
    fn test_request_response_default() {
        let _handler = RequestResponse::default();
        // Functional testing in Task 1.2
    }

    #[test]
    fn test_request_error_display() {
        let err = RequestError::Timeout;
        let msg = format!("{}", err);
        assert!(msg.contains("timed out"));
    }
}
