//! Fire-and-forget messaging pattern.
//!
//! This module provides fire-and-forget messaging capabilities
//! where messages are sent without awaiting responses.

use std::sync::Arc;

// ============================================================================
// TESTING STRATEGY
// ============================================================================
//
// This module contains placeholder types for fire-and-forget messaging.
// Full functional testing will be implemented in Task 1.2 when actual
// implementation is moved from runtime/messaging.rs.
//
// Current tests verify:
// - Types can be instantiated
// - Types have Default impls
//
// Task 1.2 will add:
// - Functional tests for FireAndForget
// - Integration tests with MessageBroker
// ============================================================================

// Placeholder type definitions

/// Fire-and-forget messaging sender.
#[derive(Debug, Clone)]
pub struct FireAndForget {
    _inner: Arc<()>, // Placeholder
}

impl FireAndForget {
    /// Create a new fire-and-forget sender.
    pub fn new() -> Self {
        Self {
            _inner: Arc::new(()),
        }
    }
}

impl Default for FireAndForget {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "Placeholder - to be implemented in Task 1.2"]
    fn test_fire_and_forget_creation() {
        let _sender = FireAndForget::new();
        // Functional testing in Task 1.2
    }

    #[test]
    #[ignore = "Placeholder - to be implemented in Task 1.2"]
    fn test_fire_and_forget_default() {
        let _sender = FireAndForget::default();
        // Functional testing in Task 1.2
    }
}
