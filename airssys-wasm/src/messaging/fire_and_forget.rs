//! Fire-and-forget messaging pattern.
//!
//! This module provides fire-and-forget messaging functionality for
//! inter-component communication. Fire-and-forget messages are sent
//! without expecting a response.
//!
//! # Phase 2+ Work
//!
//! Fire-and-forget messaging infrastructure will be implemented in Phase 2
//! or later phases of WASM-TASK-006.
//!
//! Current implementation is handled by MessageBroker directly (airssys-rt).
//!
//! # Architecture (Planned for Phase 2)
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │         Fire-and-forget Message    │
//! │  • No correlation tracking             │
//! │  • No response expected              │
//! │  • Best-effort delivery            │
//! └─────────────────────────────────────────┘
//!           ↓ published to
//! ┌─────────────────────────────────────────┐
//! │        MessageBroker               │
//! │  • Routes to all subscribers        │
//! │  • No delivery confirmation        │
//! └─────────────────────────────────────────┘
//! ```
//!
//! # Usage Pattern (When Implemented)
//!
//! ```rust,ignore
//! use airssys_wasm::messaging::FireAndForget;
//!
//! // Send message without expecting response
//! FireAndForget::send(
//!     ComponentId::new("recipient"),
//!     b"hello",
//! ).await?;
//! ```
//!
//! # References
//!
//! - **KNOWLEDGE-WASM-005**: Messaging Architecture
//! - **KNOWLEDGE-WASM-029**: Messaging Patterns (Pattern 1: Fire-and-Forget)

// Layer 1: Standard library imports
use std::sync::Arc;

// ============================================================================
// TESTING STRATEGY
// ============================================================================
//
// This module contains placeholder types for fire-and-forget messaging.
// Full functional testing will be implemented in Phase 2+.
//
// Current tests verify:
// - Types can be instantiated
// - Types have Default impls
//
// Phase 2 will add:
// - Functional tests for FireAndForget
// - Integration tests with MessageBroker
// ============================================================================

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
    fn test_fire_and_forget_creation() {
        let _sender = FireAndForget::new();
        // Functional testing in Phase 2
    }

    #[test]
    fn test_fire_and_forget_default() {
        let _sender = FireAndForget::default();
        // Functional testing in Phase 2
    }
}
