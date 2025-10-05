//! Backpressure strategies for handling full mailboxes.
//!
//! This module will be fully implemented in Phase 3 of RT-TASK-003.
//! For now, we provide the enum definition needed by MailboxError.

// Layer 1: Standard library imports
use std::fmt;

// Layer 2: Third-party crate imports
use serde::{Deserialize, Serialize};

// Layer 3: Internal module imports
// (none yet - full implementation in Phase 3)

/// Backpressure strategies for handling full mailboxes.
///
/// When a bounded mailbox reaches capacity, the backpressure strategy
/// determines how the system handles additional incoming messages.
///
/// Full implementation and documentation will be added in Phase 3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackpressureStrategy {
    /// Block sender until space becomes available (async wait).
    Block,

    /// Drop the oldest message in the queue to make room for new message.
    DropOldest,

    /// Drop the incoming (newest) message.
    DropNewest,

    /// Return an error to the sender immediately.
    Error,
}

impl Default for BackpressureStrategy {
    fn default() -> Self {
        Self::Error
    }
}

impl fmt::Display for BackpressureStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Block => write!(f, "Block"),
            Self::DropOldest => write!(f, "DropOldest"),
            Self::DropNewest => write!(f, "DropNewest"),
            Self::Error => write!(f, "Error"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backpressure_strategy_default() {
        assert_eq!(BackpressureStrategy::default(), BackpressureStrategy::Error);
    }

    #[test]
    fn test_backpressure_strategy_display() {
        assert_eq!(BackpressureStrategy::Block.to_string(), "Block");
        assert_eq!(BackpressureStrategy::DropOldest.to_string(), "DropOldest");
        assert_eq!(BackpressureStrategy::DropNewest.to_string(), "DropNewest");
        assert_eq!(BackpressureStrategy::Error.to_string(), "Error");
    }
}
