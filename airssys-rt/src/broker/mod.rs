//! Message broker infrastructure for type-safe actor message routing.
//!
//! The broker module provides the core message routing infrastructure that enables
//! actors to communicate with each other. This is infrastructure-level machinery
//! managed by the ActorSystem and completely hidden from actor implementations.
//!
//! # Architecture
//!
//! - **Separation of Concerns**: Actors remain isolated from broker complexity
//! - **Zero-Cost Abstractions**: Full generic constraints, no trait objects in actor APIs
//! - **Lock-Free Registry**: DashMap-based concurrent routing table
//! - **Zero-Copy Routing**: Ownership transfer pattern for message delivery
//!
//! # Modules
//!
//! - [`error`]: Comprehensive broker error types
//! - [`traits`]: Generic MessageBroker<M> trait definition
//! - [`registry`]: Actor registry with lock-free routing table
//! - [`in_memory`]: Default InMemoryMessageBroker implementation
//!
//! # Example (System-Level Usage)
//!
//! ```ignore
//! use airssys_rt::broker::{InMemoryMessageBroker, MessageBroker};
//! use airssys_rt::message::MessageEnvelope;
//!
//! // System creates and manages broker
//! let broker = InMemoryMessageBroker::<MyMessage>::new();
//!
//! // System registers actors
//! broker.register_actor(address, mailbox_sender)?;
//!
//! // System routes messages
//! let envelope = MessageEnvelope::new(message).with_recipient(address);
//! broker.send(envelope).await?;
//! ```

pub mod error;
pub mod registry;
pub mod traits;

pub use error::BrokerError;
pub use registry::{ActorRegistry, PoolStrategy};
pub use traits::MessageBroker;
