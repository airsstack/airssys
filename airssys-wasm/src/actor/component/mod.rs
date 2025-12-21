//! Component actor implementation and management.
//!
//! This module contains all component-centric logic including the core
//! ComponentActor implementation, lifecycle management, registry, spawning,
//! and component-specific supervision.
//!
//! # Architecture
//!
//! The component module follows the dual-trait pattern:
//! - **Actor trait**: Handles inter-component messages via mailbox
//! - **Child trait**: Manages WASM runtime lifecycle under supervisor control
//!
//! # Module Organization
//!
//! - `component_actor` - Core ComponentActor struct and types
//! - `actor_impl` - Actor trait implementation (message handling)
//! - `child_impl` - Child trait implementation (WASM lifecycle)
//! - `type_conversion` - WASM parameter conversion utilities
//! - `component_registry` - Component lookup and tracking
//! - `component_spawner` - Component spawning logic
//! - `component_supervisor` - Component-specific supervision

// Module declarations
pub mod actor_impl;
pub mod child_impl;
pub mod component_actor;
pub mod component_registry;
pub mod component_spawner;
pub mod component_supervisor;
pub mod type_conversion;

// Public re-exports
#[doc(inline)]
pub use crate::core::{ComponentMessage, ComponentHealthStatus as HealthStatus};
#[doc(inline)]
pub use component_actor::{
    ActorState, ComponentActor, MessageReceptionConfig,
};
#[doc(inline)]
pub use component_registry::ComponentRegistry;
#[doc(inline)]
pub use component_spawner::ComponentSpawner;
#[doc(inline)]
pub use component_supervisor::{
    ComponentSupervisor, RestartDecision, SupervisionHandle, SupervisionState,
    SupervisionStatistics, SupervisionTree, SupervisionTreeNode,
};
#[doc(inline)]
pub use type_conversion::{extract_wasm_results, prepare_wasm_params};
