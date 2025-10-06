//! Utility types and helpers for the actor system

pub mod ids;
pub mod serde_helpers;

pub use ids::{ActorAddress, ActorId, MessageId};
pub use serde_helpers::duration_serde;
