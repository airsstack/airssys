//! Actor system core with zero-cost abstractions.
//!
//! This module provides the foundational actor system components for building
//! concurrent, fault-tolerant applications using the actor model.
//!
//! # Components
//!
//! - [`Actor`] - Core trait that all actors must implement
//! - [`ActorContext`] - Actor metadata and messaging interface
//! - [`ActorLifecycle`] - State management and restart tracking
//! - [`ActorState`] - Lifecycle state enum (Starting, Running, Stopping, etc.)
//! - [`ErrorAction`] - Supervision decision enum (Stop, Resume, Restart, Escalate)
//!
//! # Design Philosophy
//!
//! - **Zero-cost abstractions**: Generic constraints instead of trait objects (§6.2)
//! - **Type safety**: Associated types for Message and Error (compile-time verification)
//! - **Supervision**: Built-in fault tolerance with supervisor trees
//! - **Isolation**: Each actor has independent state (no shared mutable state)
//!
//! # Performance Characteristics
//!
//! Based on RT-TASK-008 baseline measurements (Oct 16, 2025):
//!
//! - **Actor spawn**: ~625ns (single actor)
//! - **Batch spawn**: ~681ns/actor (10 actors, only 9% overhead)
//! - **Message processing**: ~31.5ns/message (direct handling)
//! - **Scaling**: Linear with 6% overhead from 1→50 actors
//!
//! Source: `BENCHMARKING.md` §6.1
//!
//! # Actor Model Basics
//!
//! Actors are independent units of computation that:
//! - Maintain isolated state (no shared memory)
//! - Communicate via asynchronous message passing
//! - Process messages sequentially (one at a time)
//! - Follow a defined lifecycle (pre_start → handle_message → post_stop)
//!
//! # Quick Start Example
//!
//! ```rust,ignore
//! use airssys_rt::prelude::*;
//! use async_trait::async_trait;
//!
//! // 1. Define your message type
//! #[derive(Debug, Clone)]
//! enum CounterMessage {
//!     Increment,
//!     GetCount(tokio::sync::oneshot::Sender<u64>),
//! }
//!
//! impl Message for CounterMessage {
//!     const MESSAGE_TYPE: &'static str = "counter";
//! }
//!
//! // 2. Define your actor
//! struct CounterActor {
//!     count: u64,
//! }
//!
//! // 3. Implement the Actor trait
//! #[async_trait]
//! impl Actor for CounterActor {
//!     type Message = CounterMessage;
//!     type Error = std::io::Error;
//!     
//!     async fn handle_message<B: MessageBroker<Self::Message>>(
//!         &mut self,
//!         msg: Self::Message,
//!         ctx: &mut ActorContext<Self::Message, B>,
//!     ) -> Result<(), Self::Error> {
//!         match msg {
//!             CounterMessage::Increment => self.count += 1,
//!             CounterMessage::GetCount(reply) => {
//!                 let _ = reply.send(self.count);
//!             }
//!         }
//!         Ok(())
//!     }
//! }
//! ```
//!
//! # Module Organization (§4.3)
//!
//! This mod.rs file contains ONLY module declarations and re-exports.
//! Implementation code is in individual module files:
//!
//! - `traits.rs` - Actor trait and ErrorAction enum
//! - `context.rs` - ActorContext implementation
//! - `lifecycle.rs` - ActorLifecycle and ActorState
//!
//! # See Also
//!
//! - [`message`](crate::message) - Message system for actor communication
//! - [`supervisor`](crate::supervisor) - Supervisor trees for fault tolerance
//! - [`broker`](crate::broker) - Message broker for pub/sub patterns

pub mod context;
pub mod lifecycle;
pub mod traits;

pub use context::ActorContext;
pub use lifecycle::{ActorLifecycle, ActorState};
pub use traits::{Actor, ErrorAction};
