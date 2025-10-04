//! # airssys-rt - Lightweight Erlang-Actor Model Runtime
//!
//! Zero-cost actor system with compile-time type safety and BEAM-inspired patterns.
//!
//! ## Features
//! - **Zero-Cost Abstractions**: No runtime overhead from generic constraints
//! - **Type Safety**: Compile-time message type verification
//! - **BEAM-Inspired**: Supervision trees and fault tolerance patterns
//! - **High Performance**: Designed for 10,000+ concurrent actors

pub mod actor;
pub mod message;
pub mod util;

// Re-export commonly used types
pub use actor::{Actor, ActorContext, ErrorAction};
pub use message::{Message, MessageEnvelope, MessagePriority};
pub use util::{ActorAddress, ActorId, MessageId};
