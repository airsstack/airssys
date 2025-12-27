//! Message routing for inter-component communication.
//!
//! This module provides message routing functionality using MessageBroker.

use std::sync::Arc;

// ============================================================================
// TESTING STRATEGY
// ============================================================================
//
// This module contains placeholder types for message routing.
// Full functional testing will be implemented in Task 1.2 when actual
// implementation is moved from runtime/messaging.rs.
//
// Current tests verify:
// - Types can be instantiated
// - Types have Default impls
//
// Task 1.2 will add:
// - Functional tests for MessageRouter
// - Integration tests with MessageBroker
// ============================================================================

// Placeholder type definitions

/// Message router for routing messages between components.
#[derive(Debug, Clone)]
pub struct MessageRouter {
    _inner: Arc<()>, // Placeholder
}

impl MessageRouter {
    /// Create a new message router.
    pub fn new() -> Self {
        Self {
            _inner: Arc::new(()),
        }
    }
}

impl Default for MessageRouter {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for message routing.
#[derive(Debug, Clone, Default)]
pub struct RoutingStats {
    _route_count: u64, // Placeholder
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "Placeholder - to be implemented in Task 1.2"]
    fn test_router_creation() {
        let _router = MessageRouter::new();
        // Functional testing in Task 1.2
    }

    #[test]
    #[ignore = "Placeholder - to be implemented in Task 1.2"]
    fn test_router_default() {
        let _router = MessageRouter::default();
        // Functional testing in Task 1.2
    }

    #[test]
    #[ignore = "Placeholder - to be implemented in Task 1.2"]
    fn test_routing_stats_default() {
        let _stats = RoutingStats::default();
        // Functional testing in Task 1.2
    }
}
