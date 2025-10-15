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
pub mod broker;
pub mod mailbox;
pub mod message;
pub mod monitoring;
pub mod supervisor;
pub mod system;
pub mod util;

// Re-export commonly used types
pub use actor::{Actor, ActorContext, ActorLifecycle, ActorState, ErrorAction};
pub use broker::{ActorRegistry, BrokerError, InMemoryMessageBroker, MessageBroker, PoolStrategy};
pub use mailbox::{
    BackpressureStrategy, BoundedMailbox, BoundedMailboxSender, MailboxReceiver, MailboxSender,
};
pub use message::{Message, MessageEnvelope, MessagePriority};
pub use monitoring::{
    ActorEvent, ActorEventKind, BrokerEvent, BrokerEventKind, EventSeverity, InMemoryMonitor,
    MailboxEvent, MailboxEventKind, Monitor, MonitoringConfig, MonitoringError, MonitoringEvent,
    MonitoringSnapshot, NoopMonitor, SupervisionEvent, SupervisionEventKind, SystemEvent,
    SystemEventKind,
};
pub use supervisor::{
    Child, ChildHandle, ChildHealth, ChildId, ChildSpec, ChildState, OneForAll, OneForOne,
    RestForOne, RestartBackoff, RestartPolicy, ShutdownPolicy, SupervisionDecision,
    SupervisionStrategy, Supervisor, SupervisorError, SupervisorId, SupervisorNode, SupervisorTree,
};
pub use system::{SystemConfig, SystemError};
pub use util::{ActorAddress, ActorId, MessageId};
