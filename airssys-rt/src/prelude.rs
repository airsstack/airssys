//! Prelude module for convenient imports.
//!
//! This module re-exports the most commonly used types and traits for building
//! actor systems with airssys-rt. Import this module to get started quickly:
//!
//! ```rust
//! use airssys_rt::prelude::*;
//! ```
//!
//! # What's Included
//!
//! ## Core Actor System
//! - [`Actor`] - Core trait for actors
//! - [`ActorContext`] - Actor execution context
//! - [`ActorLifecycle`] - Actor lifecycle tracking
//! - [`ActorState`] - Lifecycle state enum
//! - [`ErrorAction`] - Supervision decision enum
//!
//! ## Messaging
//! - [`Message`] - Core trait for messages
//! - [`MessageEnvelope`] - Message wrapper with metadata
//! - [`MessagePriority`] - Priority levels (High, Normal, Low)
//!
//! ## Mailbox
//! - [`BoundedMailbox`] - Capacity-limited mailbox
//! - [`UnboundedMailbox`] - Unlimited capacity mailbox
//! - [`BackpressureStrategy`] - Flow control strategies
//! - [`MailboxReceiver`] - Trait for receiving messages
//! - [`MailboxSender`] - Trait for sending messages
//!
//! ## Message Broker
//! - [`MessageBroker`] - Core routing trait
//! - [`InMemoryMessageBroker`] - Default broker implementation
//!
//! ## Supervision
//! - [`Child`] - Trait for supervised entities
//! - [`Supervisor`] - Supervisor trait
//! - [`SupervisorNode`] - Supervisor implementation
//! - [`OneForOne`] - Independent child strategy
//! - [`OneForAll`] - Restart all strategy
//! - [`RestForOne`] - Restart following strategy
//! - [`RestartPolicy`] - When to restart (Permanent, Transient, Temporary)
//! - [`ShutdownPolicy`] - How to stop (Graceful, Immediate, Infinity)
//! - [`ChildSpec`] - Child specification
//! - [`ChildHealth`] - Health status enum
//! - [`ChildState`] - Child lifecycle state
//!
//! ## Monitoring
//! - [`Monitor`] - Core monitoring trait
//! - [`InMemoryMonitor`] - Production monitor
//! - [`NoopMonitor`] - Zero-overhead no-op monitor
//! - [`MonitoringEvent`] - Trait for events
//! - [`EventSeverity`] - Event severity levels
//! - [`ActorEvent`] - Actor lifecycle events
//! - [`SupervisionEvent`] - Supervision events
//! - [`MailboxEvent`] - Mailbox events
//! - [`BrokerEvent`] - Broker events
//!
//! ## System
//! - [`SystemConfig`] - Actor system configuration
//!
//! ## Utilities
//! - [`ActorAddress`] - Actor address type
//! - [`ActorId`] - Actor identifier
//! - [`MessageId`] - Message identifier
//!
//! # Example
//!
//! ```rust,ignore
//! use airssys_rt::prelude::*;
//! use async_trait::async_trait;
//!
//! #[derive(Debug, Clone)]
//! struct MyMessage {
//!     data: String,
//! }
//!
//! impl Message for MyMessage {
//!     const MESSAGE_TYPE: &'static str = "my_message";
//! }
//!
//! struct MyActor {
//!     count: u64,
//! }
//!
//! #[async_trait]
//! impl Actor for MyActor {
//!     type Message = MyMessage;
//!     type Error = std::io::Error;
//!     
//!     async fn handle_message<B: MessageBroker<Self::Message>>(
//!         &mut self,
//!         msg: Self::Message,
//!         ctx: &mut ActorContext<Self::Message, B>,
//!     ) -> Result<(), Self::Error> {
//!         self.count += 1;
//!         println!("Received: {}", msg.data);
//!         Ok(())
//!     }
//! }
//! ```

// Core actor system
pub use crate::actor::{Actor, ActorContext, ActorLifecycle, ActorState, ErrorAction};

// Messaging
pub use crate::message::{Message, MessageEnvelope, MessagePriority};

// Mailbox
pub use crate::mailbox::{
    BackpressureStrategy, BoundedMailbox, BoundedMailboxSender, MailboxReceiver, MailboxSender,
    UnboundedMailbox, UnboundedMailboxSender,
};

// Message broker
pub use crate::broker::{InMemoryMessageBroker, MessageBroker};

// Supervision
pub use crate::supervisor::{
    Child, ChildHealth, ChildId, ChildSpec, ChildState, OneForAll, OneForOne, RestForOne,
    RestartPolicy, ShutdownPolicy, Supervisor, SupervisorNode,
};

// Monitoring
pub use crate::monitoring::{
    ActorEvent, BrokerEvent, EventSeverity, InMemoryMonitor, MailboxEvent, Monitor,
    MonitoringEvent, NoopMonitor, SupervisionEvent,
};

// System
pub use crate::system::SystemConfig;

// Utilities
pub use crate::util::{ActorAddress, ActorId, MessageId};
