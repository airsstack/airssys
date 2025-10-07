//! # Monitoring Module
//!
//! Universal monitoring infrastructure for observing and tracking events across all runtime components.
//!
//! ## Features
//! - **Generic Monitor<E> Trait**: Universal monitoring for any entity type
//! - **Zero-Cost Abstraction**: NoopMonitor compiles away when disabled
//! - **Lock-Free Recording**: Atomic counters for concurrent event tracking
//! - **Type Safety**: MonitoringEvent trait ensures compile-time correctness
//!
//! ## Event Types
//! - `SupervisionEvent`: Supervisor tree operations and failures
//! - `ActorEvent`: Actor lifecycle and message processing
//! - `SystemEvent`: Actor system-level events
//! - `BrokerEvent`: Message broker operations
//! - `MailboxEvent`: Mailbox operations and backpressure
//!
//! ## Monitor Implementations
//! - **InMemoryMonitor**: Production monitor with atomic counters and ring buffer
//! - **NoopMonitor**: Zero-overhead no-op monitor for disabled monitoring
//!
//! ## Examples
//! ```rust,ignore
//! use airssys_rt::monitoring::{InMemoryMonitor, MonitoringConfig, ActorEvent};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = MonitoringConfig::default();
//! let monitor = InMemoryMonitor::new(config);
//!
//! // Record events
//! // monitor.record(event).await?;
//!
//! // Take snapshots
//! // let snapshot = monitor.snapshot().await?;
//! # Ok(())
//! # }
//! ```

pub mod error;
pub mod in_memory;
pub mod noop;
pub mod traits;
pub mod types;

pub use error::MonitoringError;
pub use in_memory::InMemoryMonitor;
pub use noop::NoopMonitor;
pub use traits::{EventSeverity, Monitor, MonitoringEvent};
pub use types::{
    ActorEvent, ActorEventKind, BrokerEvent, BrokerEventKind, MailboxEvent, MailboxEventKind,
    MonitoringConfig, MonitoringSnapshot, SupervisionEvent, SupervisionEventKind, SystemEvent,
    SystemEventKind,
};
