//! Supervisor framework for fault-tolerant actor supervision.
//!
//! This module provides BEAM/Erlang-inspired supervision capabilities for building
//! fault-tolerant systems. Supervisors manage child processes (actors, tasks, or any
//! entity implementing the `Child` trait) and implement restart strategies to recover
//! from failures.
//!
//! # Architecture
//!
//! The supervisor framework is built on several core concepts:
//!
//! ## Child Trait
//!
//! The [`Child`] trait defines the lifecycle interface for supervised entities.
//! Any entity implementing this trait can be placed under supervision. This includes:
//! - **Actors**: Automatically implement `Child` via blanket implementation
//! - **Background Tasks**: Custom `Child` implementations for compute workers
//! - **I/O Handlers**: File watchers, network listeners, resource managers
//! - **System Services**: Monitoring daemons, connection pools, caches
//!
//! ## Supervision Strategies
//!
//! The framework supports three BEAM-inspired restart strategies:
//! - **OneForOne**: Restart only the failed child
//! - **OneForAll**: Restart all children when one fails
//! - **RestForOne**: Restart the failed child and all children started after it
//!
//! ## Restart Policies
//!
//! Children can be configured with different restart policies:
//! - **Permanent**: Always restart the child
//! - **Transient**: Restart only if the child exits abnormally
//! - **Temporary**: Never restart the child
//!
//! # Examples
//!
//! See the individual strategy documentation for detailed examples:
//! - [`OneForOne`] - Independent children
//! - [`OneForAll`] - Interdependent children  
//! - [`RestForOne`] - Dependent startup sequences
//!
//! See also the examples in the repository:
//! - `examples/supervisor_automatic_health.rs` - Automatic health monitoring
//! - `examples/supervisor_basic.rs` - Basic supervision patterns
//!
//! For detailed examples of implementing the `Child` trait, see:
//! - [`Child`] trait documentation
//! - [`traits::Child`] for implementation details
//!
//! # Module Structure
//!
//! - [`traits`]: Core traits (`Child`, `Supervisor`, `SupervisionStrategy`)
//! - [`types`]: Type definitions (`ChildSpec`, `RestartPolicy`, `ChildHealth`, etc.)
//! - [`error`]: Error types for supervision operations

// Module declarations
pub mod backoff;
pub mod builder;
pub mod error;
pub mod health_monitor;
pub mod node;
pub mod strategy;
pub mod traits;
pub mod tree;
pub mod types;

// Re-exports for convenient access
pub use backoff::RestartBackoff;
pub use builder::{
    SingleChildBuilder, DEFAULT_RESTART_POLICY, DEFAULT_SHUTDOWN_POLICY, DEFAULT_SHUTDOWN_TIMEOUT,
    DEFAULT_START_TIMEOUT,
};
pub use error::SupervisorError;
pub use node::{ChildHandle, HealthConfig, SupervisorNode};
pub use strategy::{should_restart, should_restart_any, OneForAll, OneForOne, RestForOne};
pub use traits::{Child, SupervisionStrategy, Supervisor};
pub use tree::{SupervisorId, SupervisorTree};
pub use types::{
    ChildHealth, ChildId, ChildSpec, ChildState, RestartPolicy, ShutdownPolicy, StrategyContext,
    SupervisionDecision,
};
