//! # airssys-rt - Lightweight Erlang-Actor Model Runtime
//!
//! High-performance actor system with zero-cost abstractions, compile-time type safety,
//! and BEAM-inspired supervision for building fault-tolerant concurrent applications.
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use airssys_rt::prelude::*;
//! use async_trait::async_trait;
//!
//! // 1. Define your message type
//! #[derive(Debug, Clone)]
//! enum CounterMsg {
//!     Increment,
//!     GetCount(tokio::sync::oneshot::Sender<u64>),
//! }
//!
//! impl Message for CounterMsg {
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
//!     type Message = CounterMsg;
//!     type Error = std::io::Error;
//!     
//!     async fn handle_message<B: MessageBroker<Self::Message>>(
//!         &mut self,
//!         msg: Self::Message,
//!         ctx: &mut ActorContext<Self::Message, B>,
//!     ) -> Result<(), Self::Error> {
//!         match msg {
//!             CounterMsg::Increment => self.count += 1,
//!             CounterMsg::GetCount(reply) => {
//!                 let _ = reply.send(self.count);
//!             }
//!         }
//!         Ok(())
//!     }
//! }
//!
//! // 4. Spawn and use your actor
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let system = ActorSystem::new(SystemConfig::default());
//!     let actor = CounterActor { count: 0 };
//!     let address = system.spawn(actor).await?;
//!     
//!     // Send messages
//!     system.send(address, CounterMsg::Increment).await?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! # Core Features
//!
//! ## Zero-Cost Abstractions
//! - **Generic constraints** instead of trait objects (§6.2)
//! - **Compile-time dispatch** via monomorphization
//! - **No heap allocations** for message passing (stack-based envelopes)
//! - **Performance**: ~625ns actor spawn, 31.5ns/message, 4.7M msgs/sec throughput
//!
//! ## Type Safety
//! - **Compile-time message type verification** via `Message` trait
//! - **Associated types** prevent runtime type errors
//! - **No `dyn` traits** in public APIs (except where explicitly needed)
//! - **Generic `MessageBroker<M>`** ensures type-safe routing
//!
//! ## BEAM-Inspired Supervision
//! - **Supervision trees** for fault tolerance and automatic recovery
//! - **Three strategies**: OneForOne, OneForAll, RestForOne
//! - **Restart policies**: Permanent, Transient, Temporary
//! - **Health monitoring**: Proactive failure detection
//!
//! ## High Performance
//! - **10,000+ concurrent actors** with linear scaling
//! - **4.7M messages/sec** throughput (4.7x target)
//! - **Lock-free operations** where possible (DashMap registry, atomic counters)
//! - **Async/await** with Tokio runtime
//!
//! # Performance Characteristics
//!
//! Based on RT-TASK-008 baseline measurements (Oct 16, 2025):
//!
//! - **Actor spawn**: ~625ns (single), ~681ns/actor (batch of 10)
//! - **Message creation**: ~737ns (with envelope and metadata)
//! - **Message processing**: ~31.5ns/message (direct actor handling)
//! - **Broker routing**: ~212ns/message (registry lookup + mailbox send)
//! - **Mailbox operations**: ~182ns/message (bounded mailbox with metrics)
//! - **Message throughput**: ~4.7M messages/sec (4.7x target of 1M/sec)
//! - **Scaling**: Linear with 6% overhead (1→50 actors)
//!
//! Source: `BENCHMARKING.md` §6.1-§6.3
//!
//! # Module Organization
//!
//! ## Core Actor System
//! - [`actor`] - Actor trait, lifecycle, and context for message handling
//! - [`message`] - Message trait, envelopes, and priority system
//! - [`mailbox`] - Message queuing with backpressure control
//! - [`broker`] - Message routing and pub/sub infrastructure
//!
//! ## Fault Tolerance
//! - [`supervisor`] - Supervision trees and restart strategies
//! - [`monitoring`] - Event tracking and metrics for observability
//!
//! ## Infrastructure
//! - [`system`] - ActorSystem configuration and management
//! - [`util`] - Utilities (ActorAddress, ActorId, MessageId)
//!
//! # Architecture Principles
//!
//! ## Separation of Concerns (§4.3)
//! - **Actor**: Defines message handling logic only
//! - **Child**: Defines supervision lifecycle (explicit implementation required)
//! - **Message**: Defines message type and metadata
//! - **Mailbox**: Manages message queuing and backpressure
//! - **Broker**: Routes messages between actors (hidden from actors)
//!
//! ## Dependency Injection (ADR-006)
//! - Generic `MessageBroker<M>` parameter in `ActorContext`
//! - Allows testing with mock brokers
//! - Enables different broker implementations
//!
//! ## YAGNI Principles (§6.1)
//! - Build only what's needed for current requirements
//! - Avoid speculative generalization
//! - Simple solutions first, complexity when proven necessary
//!
//! # Examples
//!
//! See the `examples/` directory for comprehensive examples:
//! - `actor_basic.rs` - Basic actor creation and message passing
//! - `actor_lifecycle.rs` - Actor lifecycle hooks (pre_start, post_stop)
//! - `supervisor_basic.rs` - Supervision trees and restart strategies
//! - `supervisor_automatic_health.rs` - Health monitoring and proactive restarts
//! - `monitoring_basic.rs` - Event monitoring and metrics collection
//!
//! # Standards Compliance
//!
//! This crate follows strict workspace standards documented in memory bank:
//! - **§2.1**: 3-layer import organization (std → third-party → internal)
//! - **§3.2**: chrono DateTime<Utc> for all timestamps
//! - **§4.3**: Module architecture (mod.rs only declarations)
//! - **§6.2**: Avoid `dyn` patterns (prefer generic constraints)
//! - **§7.2-§7.3**: Professional documentation (Diátaxis framework)
//! - **Microsoft Rust Guidelines**: Complete compliance (AI-optimized, type hierarchies)
//!
//! # See Also
//!
//! - [Erlang/OTP Documentation](https://www.erlang.org/doc/) - Inspiration for supervision
//! - [Actor Model (Wikipedia)](https://en.wikipedia.org/wiki/Actor_model) - Theoretical foundation
//! - Memory Bank: `.copilot/memory_bank/sub_projects/airssys-rt/` - Architecture decisions

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
