//! Message system with zero-cost abstractions
//!
//! Provides core message traits and types for type-safe message passing
//! in the actor system. Built on compile-time type identification and
//! generic constraints for maximum performance.

pub mod envelope;
pub mod traits;

pub use envelope::MessageEnvelope;
pub use traits::{Message, MessagePriority};
