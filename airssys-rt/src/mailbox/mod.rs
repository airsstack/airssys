//! Mailbox system for actor message queuing.
//!
//! This module provides the mailbox infrastructure for actors:
//! - Generic mailbox traits (MailboxReceiver, MailboxSender)
//! - Bounded mailbox implementation with capacity limits
//! - Unbounded mailbox implementation with unlimited capacity
//! - Backpressure strategies for flow control
//! - Metrics tracking for observability

pub mod backpressure;
pub mod bounded;
pub mod traits;
pub mod unbounded;

pub use backpressure::BackpressureStrategy;
pub use bounded::{BoundedMailbox, BoundedMailboxSender};
pub use traits::{
    MailboxCapacity, MailboxError, MailboxMetrics, MailboxReceiver, MailboxSender, TryRecvError,
};
pub use unbounded::{UnboundedMailbox, UnboundedMailboxSender};
