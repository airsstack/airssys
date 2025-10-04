//! Message system with zero-cost abstractions
//!
//! Provides core message traits and types for type-safe message passing
//! in the actor system. Built on compile-time type identification and
//! generic constraints for maximum performance.

pub mod traits;

pub use traits::{Message, MessagePriority};
