//! Actor system core with zero-cost abstractions.
//!
//! This module provides the foundational actor system components:
//! - `Actor` trait with generic constraints
//! - `ActorContext` for actor metadata
//! - `ActorLifecycle` for state management
//! - `ErrorAction` for supervision decisions
//!
//! # Design Philosophy
//!
//! - **Zero-cost abstractions**: Generic constraints instead of trait objects (§6.2)
//! - **Type safety**: Associated types for Message and Error
//! - **Supervision**: Built-in fault tolerance with ErrorAction
//!
//! # Module Organization (§4.3)
//!
//! This mod.rs file contains ONLY module declarations and re-exports.
//! Implementation code is in individual module files.

pub mod context;
pub mod lifecycle;
pub mod traits;

pub use context::ActorContext;
pub use lifecycle::{ActorLifecycle, ActorState};
pub use traits::{Actor, ErrorAction};
