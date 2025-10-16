//! Mailbox system for actor message queuing with backpressure control.
//!
//! This module provides high-performance mailbox infrastructure for actor message queuing,
//! with configurable capacity limits, backpressure strategies, and comprehensive metrics
//! tracking for observability and system tuning.
//!
//! # Components
//!
//! - [`MailboxReceiver`] - Trait for receiving messages from mailbox
//! - [`MailboxSender`] - Trait for sending messages to mailbox
//! - [`BoundedMailbox`] - Fixed-capacity mailbox with backpressure
//! - [`UnboundedMailbox`] - Unlimited-capacity mailbox (use with caution)
//! - [`BackpressureStrategy`] - Flow control strategies (Block, Drop, Reject)
//! - [`MetricsRecorder`] - Message queue metrics tracking
//!
//! # Design Philosophy
//!
//! - **Generic constraints**: Zero-cost abstractions via trait-based design (§6.2)
//! - **Backpressure control**: Prevent unbounded memory growth
//! - **Performance**: Lock-free operations where possible, ~182ns per operation
//! - **Observability**: Comprehensive metrics for queue depth, throughput, and drops
//! - **Flexibility**: Multiple backpressure strategies for different use cases
//!
//! # Performance Characteristics
//!
//! Based on RT-TASK-008 baseline measurements (Oct 16, 2025):
//!
//! - **Send operation**: ~182ns/message (bounded mailbox with metrics)
//! - **Receive operation**: ~150ns/message (zero-copy message retrieval)
//! - **Capacity check**: <10ns (atomic read)
//! - **Memory per mailbox**: ~200 bytes + (message_size × capacity)
//! - **Metrics overhead**: ~32ns per operation (atomic increments)
//!
//! Source: `BENCHMARKING.md` §6.3 (mailbox operations)
//!
//! # Bounded vs Unbounded Mailboxes
//!
//! ## Bounded Mailbox (Recommended)
//!
//! Fixed capacity with backpressure control:
//! - **Memory safety**: Prevents unbounded memory growth
//! - **Backpressure**: Configurable strategies (block, drop, reject)
//! - **Predictable performance**: Fixed memory allocation
//! - **Use case**: Production systems, resource-constrained environments
//!
//! ## Unbounded Mailbox (Use with Caution)
//!
//! Unlimited capacity without backpressure:
//! - **Risk**: Can cause out-of-memory if producers overwhelm consumers
//! - **Performance**: Minimal overhead (~100ns per operation)
//! - **Use case**: Development, testing, guaranteed low-volume scenarios
//! - **Warning**: Monitor queue depth closely in production
//!
//! # Backpressure Strategies
//!
//! ## Block Strategy (Default)
//! ```text
//! Sender waits until mailbox has capacity
//! - Guarantees message delivery
//! - May block sender if receiver is slow
//! - Use for critical messages
//! ```
//!
//! ## Drop Strategy
//! ```text
//! Drop message if mailbox is full
//! - Non-blocking sends
//! - Messages may be lost
//! - Use for non-critical, high-volume telemetry
//! ```
//!
//! ## Reject Strategy
//! ```text
//! Return error if mailbox is full
//! - Non-blocking sends
//! - Caller handles backpressure
/// - Use when sender needs to know about capacity issues
/// ```
///
/// # Quick Start Examples
///
/// ## Example 1: Basic Bounded Mailbox
///
/// ```rust,ignore
/// use airssys_rt::mailbox::{BoundedMailbox, BackpressureStrategy};
///
/// // Create bounded mailbox with capacity 100
/// let (sender, receiver) = BoundedMailbox::new(
///     100,  // capacity
///     BackpressureStrategy::Block,  // block when full
/// );
///
/// // Send message (blocks if full)
/// sender.send(MyMessage { data: "hello" }).await?;
///
/// // Receive message (blocks if empty)
/// let msg = receiver.recv().await?;
/// ```
///
/// ## Example 2: Backpressure with Drop Strategy
///
/// ```rust,ignore
/// use airssys_rt::mailbox::{BoundedMailbox, BackpressureStrategy};
///
/// // Create mailbox that drops messages when full
/// let (sender, receiver) = BoundedMailbox::new(
///     1000,
///     BackpressureStrategy::Drop,  // drop oldest when full
/// );
///
/// // Send telemetry (may drop if consumer is slow)
/// for i in 0..10000 {
///     let _ = sender.send(TelemetryEvent { value: i }).await;
/// }
/// ```
///
/// ## Example 3: Metrics Tracking
///
/// ```rust,ignore
/// use airssys_rt::mailbox::{BoundedMailbox, BackpressureStrategy, MetricsRecorder};
///
/// // Create mailbox with metrics
/// let metrics = MetricsRecorder::new();
/// let (sender, receiver) = BoundedMailbox::with_metrics(
///     100,
///     BackpressureStrategy::Block,
///     metrics.clone(),
/// );
///
/// // Send messages
/// sender.send(msg1).await?;
/// sender.send(msg2).await?;
///
/// // Check metrics
/// println!("Queue depth: {}", metrics.queue_depth());
/// println!("Messages sent: {}", metrics.messages_sent());
/// println!("Messages received: {}", metrics.messages_received());
/// println!("Messages dropped: {}", metrics.messages_dropped());
/// ```
///
/// ## Example 4: Reject Strategy with Error Handling
///
/// ```rust,ignore
/// use airssys_rt::mailbox::{BoundedMailbox, BackpressureStrategy, MailboxError};
///
/// let (sender, receiver) = BoundedMailbox::new(
///     10,
///     BackpressureStrategy::Reject,  // return error when full
/// );
///
/// // Handle backpressure explicitly
/// match sender.try_send(msg).await {
///     Ok(()) => println!("Message sent"),
///     Err(MailboxError::Full) => {
///         // Backpressure detected - handle it
///         println!("Mailbox full, applying backoff");
///         tokio::time::sleep(Duration::from_millis(100)).await;
///         sender.try_send(msg).await?;  // retry
///     }
///     Err(e) => return Err(e),
/// }
/// ```
pub mod backpressure;
pub mod bounded;
pub mod metrics;
pub mod traits;
pub mod unbounded;

pub use backpressure::BackpressureStrategy;
pub use bounded::{BoundedMailbox, BoundedMailboxSender};
pub use metrics::{AtomicMetrics, MetricsRecorder};
pub use traits::{MailboxCapacity, MailboxError, MailboxReceiver, MailboxSender, TryRecvError};
pub use unbounded::{UnboundedMailbox, UnboundedMailboxSender};
