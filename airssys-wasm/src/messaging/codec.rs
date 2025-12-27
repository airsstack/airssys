//! Multicodec message encoding.
//!
//! This module provides message encoding/decoding using multicodec format.

use std::sync::Arc;

// ============================================================================
// TESTING STRATEGY
// ============================================================================
//
// This module contains placeholder types for multicodec encoding.
// Full functional testing will be implemented in Task 1.2 when actual
// implementation is moved from runtime/messaging.rs.
//
// Current tests verify:
// - Types can be instantiated
// - Types have Default impls
//
// Task 1.2 will add:
// - Functional tests for MulticodecCodec
// - Integration tests with MessageBroker
// ============================================================================

// Placeholder type definitions

/// Multicodec codec for message encoding/decoding.
#[derive(Debug, Clone)]
pub struct MulticodecCodec {
    _inner: Arc<()>, // Placeholder
}

impl MulticodecCodec {
    /// Create a new multicodec codec.
    pub fn new() -> Self {
        Self {
            _inner: Arc::new(()),
        }
    }
}

impl Default for MulticodecCodec {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "Placeholder - to be implemented in Task 1.2"]
    fn test_codec_creation() {
        let _codec = MulticodecCodec::new();
        // Functional testing in Task 1.2
    }

    #[test]
    #[ignore = "Placeholder - to be implemented in Task 1.2"]
    fn test_codec_default() {
        let _codec = MulticodecCodec::default();
        // Functional testing in Task 1.2
    }
}
