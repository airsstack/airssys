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
//! ## Basic Actor Supervision
//!
//! ```rust
//! use airssys_rt::{
//!     Actor, ActorContext, InMemoryMonitor, MonitoringConfig,
//!     RestartPolicy, ShutdownPolicy, ChildSpec,
//! };
//! use async_trait::async_trait;
//! use std::time::Duration;
//!
//! // Define an actor (automatically implements Child via blanket impl)
//! struct CounterActor {
//!     count: u32,
//! }
//!
//! # #[derive(Debug, Clone)]
//! # struct CounterMsg { delta: u32 }
//! # impl airssys_rt::Message for CounterMsg {
//! #     const MESSAGE_TYPE: &'static str = "counter";
//! # }
//! # #[derive(Debug)]
//! # struct CounterError;
//! # impl std::fmt::Display for CounterError {
//! #     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//! #         write!(f, "Counter error")
//! #     }
//! # }
//! # impl std::error::Error for CounterError {}
//! #
//! #[async_trait]
//! impl Actor for CounterActor {
//!     type Message = CounterMsg;
//!     type Error = CounterError;
//!     
//!     async fn handle_message(
//!         &mut self,
//!         msg: Self::Message,
//!         _ctx: &mut ActorContext<Self::Message>,
//!     ) -> Result<(), Self::Error> {
//!         self.count += msg.delta;
//!         Ok(())
//!     }
//! }
//!
//! // ✅ CounterActor is now automatically supervisable!
//! // Use it with supervisors in RT-TASK-007 Phase 3
//! ```
//!
//! ## Custom Child Implementation
//!
//! ```rust
//! use airssys_rt::supervisor::{Child, ChildHealth};
//! use async_trait::async_trait;
//! use std::time::Duration;
//!
//! // Background worker (not an actor)
//! struct BackgroundWorker {
//!     name: String,
//!     running: bool,
//! }
//!
//! #[derive(Debug)]
//! struct WorkerError;
//!
//! impl std::fmt::Display for WorkerError {
//!     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//!         write!(f, "Worker error")
//!     }
//! }
//!
//! impl std::error::Error for WorkerError {}
//!
//! #[async_trait]
//! impl Child for BackgroundWorker {
//!     type Error = WorkerError;
//!     
//!     async fn start(&mut self) -> Result<(), Self::Error> {
//!         println!("[{}] Starting worker", self.name);
//!         self.running = true;
//!         Ok(())
//!     }
//!     
//!     async fn stop(&mut self, _timeout: Duration) -> Result<(), Self::Error> {
//!         println!("[{}] Stopping worker", self.name);
//!         self.running = false;
//!         Ok(())
//!     }
//!     
//!     async fn health_check(&self) -> ChildHealth {
//!         if self.running {
//!             ChildHealth::Healthy
//!         } else {
//!             ChildHealth::Failed("Worker not running".into())
//!         }
//!     }
//! }
//! ```
//!
//! # Module Structure
//!
//! - [`traits`]: Core traits (`Child`, `Supervisor`, `SupervisionStrategy`)
//! - [`types`]: Type definitions (`ChildSpec`, `RestartPolicy`, `ChildHealth`, etc.)
//! - [`error`]: Error types for supervision operations

// Module declarations
pub mod error;
pub mod traits;
pub mod types;

// Re-exports for convenient access
pub use error::SupervisorError;
pub use traits::{Child, SupervisionStrategy, Supervisor};
pub use types::{
    ChildHealth, ChildId, ChildSpec, ChildState, RestartPolicy, ShutdownPolicy, SupervisionDecision,
};
