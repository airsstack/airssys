//! Monitoring infrastructure for observing runtime events and metrics.
//!
//! This module provides comprehensive monitoring capabilities for tracking actor lifecycle,
//! message routing, supervision events, mailbox metrics, and system-level operations across
//! the entire airssys-rt runtime. Essential for production observability, debugging, and
//! performance tuning.
//!
//! # Components
//!
//! - [`Monitor`] - Core trait for event recording and snapshot retrieval
//! - [`InMemoryMonitor`] - Production monitor with atomic counters and ring buffer
//! - [`NoopMonitor`] - Zero-overhead no-op monitor (compiles away when disabled)
//! - [`MonitoringEvent`] - Trait for all recordable events
//! - [`EventSeverity`] - Event severity levels (Debug, Info, Warning, Error, Critical)
//!
//! # Event Types
//!
//! ## SupervisionEvent
//! - Child started, stopped, failed, restarted
//! - Supervision strategy applied
//! - Restart policy enforcement
//! - Health check results
//!
//! ## ActorEvent  
//! - Actor created, started, stopped, failed
//! - Message received, processed, dropped
//! - Lifecycle state transitions
//!
//! ## BrokerEvent
//! - Message routed, published
//! - Actor registered, deregistered
//! - Topic subscriptions
//!
//! ## MailboxEvent
//! - Message sent, received, dropped
//! - Backpressure applied
//! - Queue depth changes
//!
//! ## SystemEvent
//! - ActorSystem startup, shutdown
//! - Configuration changes
//! - Resource allocation
//!
//! # Design Philosophy
//!
//! - **Zero-cost abstraction**: `NoopMonitor` compiles to nothing (zero runtime overhead)
//! - **Type safety**: `MonitoringEvent` trait ensures compile-time event correctness
//! - **Lock-free recording**: Atomic counters for concurrent event tracking
//! - **Generic constraints**: `Monitor<E>` parameterized over event type (§6.2)
//! - **Observability-first**: Comprehensive metrics for production systems
//!
//! # Performance Characteristics
//!
//! - **Event recording**: ~100ns per event (InMemoryMonitor with atomic increments)
//! - **Snapshot retrieval**: <1μs (atomic reads + Vec allocation)
//! - **Memory per event**: ~128 bytes (event type + timestamp + metadata)
//! - **Ring buffer**: Configurable size (default 10,000 events)
//! - **NoopMonitor**: 0ns overhead (compiles away entirely)
//!
//! # Monitor Implementations
//!
//! ## InMemoryMonitor (Production)
//! - **Lock-free**: Atomic counters and RwLock for event buffer
//! - **Ring buffer**: Configurable capacity, oldest events evicted
//! - **Snapshot support**: Export events for external analysis
//! - **Use case**: Production monitoring, debugging, metrics collection
//!
//! ## NoopMonitor (Development/Testing)
//! - **Zero overhead**: All operations compile to no-ops
//! - **No allocations**: No memory usage
/// - **Use case**: Performance testing, monitoring-disabled builds
///
/// # Quick Start Examples
///
/// ## Example 1: Basic Monitoring Setup
///
/// ```rust,ignore
/// use airssys_rt::monitoring::{InMemoryMonitor, MonitoringConfig, ActorEvent};
///
/// // Create monitor with default config (10,000 event ring buffer)
/// let config = MonitoringConfig::default();
/// let monitor = InMemoryMonitor::new(config);
///
/// // Record actor lifecycle events
/// monitor.record(ActorEvent::Started {
///     actor_id: "actor-123",
///     timestamp: Utc::now(),
/// }).await?;
///
/// // Take snapshot for analysis
/// let snapshot = monitor.snapshot().await?;
/// println!("Total events: {}", snapshot.total_events);
/// println!("Actor events: {}", snapshot.actor_event_count);
/// ```
///
/// ## Example 2: Supervision Monitoring
///
/// ```rust,ignore
/// use airssys_rt::monitoring::{InMemoryMonitor, SupervisionEvent};
///
/// let monitor = InMemoryMonitor::new(MonitoringConfig::default());
///
/// // Record supervision events
/// monitor.record(SupervisionEvent::ChildFailed {
///     child_id: "worker-1",
///     error: "Connection timeout",
///     restart_count: 3,
/// }).await?;
///
/// monitor.record(SupervisionEvent::ChildRestarted {
///     child_id: "worker-1",
///     strategy: "OneForOne",
/// }).await?;
///
/// // Analyze failures
/// let snapshot = monitor.snapshot().await?;
/// for event in snapshot.recent_failures() {
///     println!("Failure: {:?}", event);
/// }
/// ```
///
/// ## Example 3: Mailbox Backpressure Monitoring
///
/// ```rust,ignore
/// use airssys_rt::monitoring::{InMemoryMonitor, MailboxEvent};
///
/// let monitor = InMemoryMonitor::new(MonitoringConfig::default());
///
/// // Record mailbox events
/// monitor.record(MailboxEvent::MessageDropped {
///     actor_id: "actor-456",
///     reason: "Mailbox full",
///     queue_depth: 1000,
/// }).await?;
///
/// monitor.record(MailboxEvent::BackpressureApplied {
///     actor_id: "actor-456",
///     strategy: "Drop",
/// }).await?;
///
/// // Check for backpressure issues
/// let snapshot = monitor.snapshot().await?;
/// println!("Messages dropped: {}", snapshot.mailbox_drops_count);
/// ```
///
/// ## Example 4: NoopMonitor for Testing
///
/// ```rust,ignore
/// use airssys_rt::monitoring::{NoopMonitor, ActorEvent};
///
/// // Zero-overhead monitoring (compiles to no-ops)
/// let monitor = NoopMonitor;
///
/// // These calls compile away to nothing
/// monitor.record(ActorEvent::Started { /* ... */ }).await?;  // 0ns
/// let snapshot = monitor.snapshot().await?;  // 0ns, empty snapshot
/// ```
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
