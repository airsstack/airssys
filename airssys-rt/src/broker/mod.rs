//! Message broker infrastructure for type-safe actor message routing and pub/sub.
//!
//! The broker module provides high-performance message routing infrastructure that enables
//! actors to communicate via direct addressing (point-to-point) and topic-based pub/sub
//! patterns. This is infrastructure-level machinery managed by the ActorSystem and
//! typically hidden from actor implementations.
//!
//! # Components
//!
//! - [`MessageBroker`] - Core trait for message routing and delivery
//! - [`InMemoryMessageBroker`] - Default lock-free in-memory implementation
//! - [`ActorRegistry`] - Concurrent actor address resolution and routing
//! - [`MessageStream`] - Async stream for topic-based subscriptions
//! - [`BrokerError`] - Comprehensive error types for routing failures
//!
//! # Architecture
//!
//! - **Separation of Concerns**: Actors remain isolated from broker complexity
//! - **Zero-Cost Abstractions**: Full generic constraints, no `Box<dyn>` in actor APIs (§6.2)
//! - **Lock-Free Registry**: DashMap-based concurrent routing table
//! - **Zero-Copy Routing**: Ownership transfer pattern for message delivery
//! - **Type Safety**: Generic over message type `M` for compile-time routing verification
//!
//! # Performance Characteristics
//!
//! Based on RT-TASK-008 baseline measurements (Oct 16, 2025):
//!
//! - **Broker routing**: ~212ns/message (registry lookup + mailbox send)
//! - **Direct send**: ~182ns/message (mailbox send only, no broker)
//! - **Registry lookup**: ~30ns (DashMap concurrent read)
//! - **Pub/sub overhead**: ~40ns per subscriber (broadcast fan-out)
//! - **Memory per actor**: ~64 bytes (ActorAddress + mailbox sender)
//!
//! Source: `BENCHMARKING.md` §6.2
//!
//! # Routing Patterns
//!
//! ## Point-to-Point (Direct Addressing)
//! ```text
//! Actor A --[message]--> Broker --[route]--> Actor B's Mailbox
//!
//! - 1:1 message delivery
//! - Address-based routing via ActorRegistry
//! - ~212ns routing latency
//! ```
//!
//! ## Pub/Sub (Topic-Based)
//! ```text
//! Publisher --[message]--> Broker --[broadcast]--> Subscriber 1
//!                                             \---> Subscriber 2
//!                                             \---> Subscriber N
//!
//! - 1:N message delivery
//! - Topic-based subscriptions
//! - ~212ns + (40ns × N subscribers)
//! ```
//!
//! # Design Decisions
//!
//! ## Why Generic MessageBroker<M>?
//!
//! - **Type safety**: Compile-time verification of message types
//! - **Zero-cost**: Monomorphization eliminates runtime dispatch overhead
//! - **Actor isolation**: No `dyn Message` trait objects in actor code
//!
//! ## Why Lock-Free Registry?
//!
//! - **Scalability**: Concurrent reads without blocking
//! - **Performance**: Sub-50ns lookups with DashMap
//! - **Reliability**: No deadlocks or lock contention
//!
//! # System-Level Usage
//!
//! The broker is typically created and managed by ActorSystem. Actors interact
//! with the broker indirectly via ActorContext methods (`send()`, `request()`).
//!
//! ## Example: System-Level Broker Setup
//!
//! ```rust,ignore
//! use airssys_rt::broker::{InMemoryMessageBroker, MessageBroker};
//! use airssys_rt::message::MessageEnvelope;
//!
//! // System creates and manages broker
//! let broker = InMemoryMessageBroker::<MyMessage>::new();
//!
//! // System registers actors with their mailbox senders
//! broker.register_actor(address, mailbox_sender)?;
//!
//! // System routes messages via broker
//! let envelope = MessageEnvelope::new(message).with_recipient(address);
//! broker.send(envelope).await?;
//!
//! // Actors never interact with broker directly
//! // They use ActorContext::send() which internally uses the broker
//! ```
//!
//! ## Example: Pub/Sub Pattern
//!
//! ```rust,ignore
//! use airssys_rt::broker::{InMemoryMessageBroker, MessageBroker};
//!
//! let broker = InMemoryMessageBroker::<MyMessage>::new();
//!
//! // Subscribe actors to topic
//! let stream1 = broker.subscribe("events").await?;
//! let stream2 = broker.subscribe("events").await?;
//!
//! // Publish message to topic
//! broker.publish("events", message).await?;
//! // Both stream1 and stream2 receive the message
//! ```
//!
//! # Modules (§4.3)
//!
//! This mod.rs file contains ONLY module declarations and re-exports.
//! Implementation code is in individual module files:
//!
//! - [`error`]: Comprehensive broker error types
//! - [`traits`]: Generic `MessageBroker<M>` trait definition
//! - [`registry`]: Actor registry with lock-free routing table
//! - [`in_memory`]: Default `InMemoryMessageBroker` implementation
//!
//! # See Also
//!
//! - [`actor`](crate::actor) - Actor system that uses broker for message routing
//! - [`message`](crate::message) - Message and MessageEnvelope types
//! - [`mailbox`](crate::mailbox) - Mailbox system for message queuing

pub mod error;
pub mod in_memory;
pub mod registry;
pub mod traits;

pub use error::BrokerError;
pub use in_memory::InMemoryMessageBroker;
pub use registry::{ActorRegistry, PoolStrategy};
pub use traits::{MessageBroker, MessageStream};
