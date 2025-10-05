//! Core mailbox traits and supporting types for actor message queuing.
//!
//! This module provides the foundational traits and types for the mailbox system:
//! - `MailboxReceiver<M>`: Generic mailbox trait for receiving messages
//! - `MailboxSender<M>`: Generic sender trait for sending messages
//! - `MailboxCapacity`: Capacity configuration (bounded/unbounded)
//! - `MailboxError`: Comprehensive error types
//! - `MailboxMetrics`: Message tracking and monitoring
//!
//! # Design Principles
//!
//! - **Zero-cost abstractions**: Generic constraints instead of trait objects (§6.2)
//! - **Type safety**: Compile-time message type verification
//! - **Async support**: Full async/await integration with tokio
//! - **Metrics tracking**: Built-in monitoring for observability
//!
//! # Example
//!
//! ```ignore
//! // Full example will be available after Phase 2 (BoundedMailbox implementation)
//! use airssys_rt::mailbox::{MailboxReceiver, BoundedMailbox};
//! use airssys_rt::message::Message;
//!
//! #[derive(Debug, Clone)]
//! struct MyMessage { data: String }
//!
//! impl Message for MyMessage {
//!     const MESSAGE_TYPE: &'static str = "my_message";
//! }
//!
//! # async fn example() {
//! // Create bounded mailbox with capacity 100
//! let (receiver, sender) = BoundedMailbox::<MyMessage>::new(100);
//! # }
//! ```

// Layer 1: Standard library imports
use std::error::Error;

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use chrono::{DateTime, Utc}; // §3.2 MANDATORY

// Layer 3: Internal module imports
use crate::message::{Message, MessageEnvelope};

/// Mailbox receiver trait with generic constraints for zero-cost abstractions.
///
/// This trait defines the interface for receiving messages from actor mailboxes.
/// The receiver is owned by a single actor and is responsible for processing
/// incoming messages from the queue.
///
/// # Type Safety
///
/// The receiver is generic over the message type M, ensuring compile-time
/// type safety without runtime dispatch or type erasure (§6.2).
///
/// # Ownership
///
/// Unlike `MailboxSender` which is `Clone`, the receiver is NOT cloneable
/// and is owned exclusively by one actor, following the actor model pattern.
///
/// # Example
///
/// ```ignore
/// // Full example will be available after Phase 2 (BoundedMailbox implementation)
/// use airssys_rt::mailbox::{MailboxReceiver, BoundedMailbox};
/// use airssys_rt::message::Message;
///
/// #[derive(Debug, Clone)]
/// struct MyMessage { data: String }
///
/// impl Message for MyMessage {
///     const MESSAGE_TYPE: &'static str = "my_message";
/// }
///
/// # async fn example() {
/// // Type-safe mailbox creation
/// let (receiver, sender) = BoundedMailbox::<MyMessage>::new(100);
/// assert_eq!(receiver.capacity(), MailboxCapacity::Bounded(100));
/// # }
/// ```
#[async_trait]
pub trait MailboxReceiver<M: Message>: Send + Sync {
    /// Error type for mailbox operations
    type Error: Error + Send + Sync + 'static;

    /// Receive the next message from the mailbox (async)
    ///
    /// Returns None if the mailbox is closed and empty.
    /// This is the primary method for actors to receive messages.
    async fn recv(&mut self) -> Option<MessageEnvelope<M>>;

    /// Try to receive a message without blocking
    ///
    /// Returns `TryRecvError::Empty` if no messages are available.
    /// Returns `TryRecvError::Closed` if the mailbox is closed.
    fn try_recv(&mut self) -> Result<MessageEnvelope<M>, TryRecvError>;

    /// Get the mailbox capacity configuration
    fn capacity(&self) -> MailboxCapacity;

    /// Get the current number of messages in the mailbox
    ///
    /// Note: This is an approximation based on sent/received counters.
    fn len(&self) -> usize;

    /// Check if the mailbox is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Sender interface for mailboxes with backpressure support.
///
/// The sender is cloneable and can be shared across multiple actors
/// for message delivery to a single mailbox.
///
/// # Cloning
///
/// Senders implement `Clone` cheaply (typically via `Arc` internally),
/// allowing multiple references to the same mailbox.
///
/// # Example
///
/// ```ignore
/// // Full example will be available after Phase 2 (BoundedMailbox implementation)
/// use airssys_rt::mailbox::{BoundedMailbox, MailboxSender};
/// use airssys_rt::message::{Message, MessageEnvelope};
///
/// #[derive(Debug, Clone)]
/// struct MyMessage;
/// impl Message for MyMessage {
///     const MESSAGE_TYPE: &'static str = "my_message";
/// }
///
/// # async fn example() {
/// let (mailbox, sender) = BoundedMailbox::<MyMessage>::new(100);
/// let sender_clone = sender.clone(); // Can be shared
///
/// // Send from cloned sender
/// sender_clone.send(MessageEnvelope::new(MyMessage)).await.unwrap();
/// # }
/// ```
#[async_trait]
pub trait MailboxSender<M: Message>: Send + Sync + Clone {
    /// Error type for send operations
    type Error: Error + Send + Sync + 'static;

    /// Send a message (async, may block with backpressure)
    ///
    /// Behavior depends on the backpressure strategy:
    /// - Block: Wait for space to become available
    /// - DropOldest/DropNewest: Drop messages according to strategy
    /// - Error: Return error immediately if full
    ///
    /// # Errors
    ///
    /// Returns `Self::Error` if the mailbox is closed or send fails.
    async fn send(&self, envelope: MessageEnvelope<M>) -> Result<(), Self::Error>;

    /// Try to send a message without blocking
    ///
    /// Returns an error immediately if the mailbox is full or closed.
    ///
    /// # Errors
    ///
    /// Returns `Self::Error` if the mailbox is full or closed.
    fn try_send(&self, envelope: MessageEnvelope<M>) -> Result<(), Self::Error>;
}

/// Mailbox capacity configuration
///
/// Defines whether a mailbox has a bounded or unbounded capacity.
///
/// # Example
///
/// ```rust
/// use airssys_rt::mailbox::MailboxCapacity;
///
/// let bounded = MailboxCapacity::Bounded(100);
/// let unbounded = MailboxCapacity::Unbounded;
///
/// assert_eq!(bounded, MailboxCapacity::Bounded(100));
/// assert_eq!(unbounded, MailboxCapacity::Unbounded);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MailboxCapacity {
    /// Bounded mailbox with maximum capacity
    Bounded(usize),

    /// Unbounded mailbox (no capacity limit)
    Unbounded,
}

/// Mailbox error types
///
/// Comprehensive error types for mailbox operations with contextual information.
///
/// # Examples
///
/// ```rust
/// use airssys_rt::mailbox::MailboxError;
/// use chrono::Utc;
///
/// let err = MailboxError::Full { capacity: 100 };
/// assert!(err.to_string().contains("full"));
/// assert!(err.to_string().contains("100"));
///
/// let err = MailboxError::Closed;
/// assert_eq!(err.to_string(), "Mailbox is closed");
/// ```
#[derive(Debug, thiserror::Error)]
pub enum MailboxError {
    /// Mailbox is full (bounded mailboxes only)
    #[error("Mailbox is full (capacity: {capacity})")]
    Full { capacity: usize },

    /// Mailbox is closed (receiver dropped)
    #[error("Mailbox is closed")]
    Closed,

    /// Backpressure strategy was applied
    #[error("Backpressure applied: {strategy:?}")]
    BackpressureApplied {
        strategy: crate::mailbox::BackpressureStrategy,
    },

    /// Message TTL expired (§3.2 chrono DateTime<Utc>)
    #[error("TTL expired for message at {timestamp}")]
    TtlExpired { timestamp: DateTime<Utc> },
}

/// Try receive error types
///
/// Error types for non-blocking receive operations.
///
/// # Examples
///
/// ```rust
/// use airssys_rt::mailbox::TryRecvError;
///
/// let err = TryRecvError::Empty;
/// assert_eq!(err.to_string(), "Mailbox is empty");
///
/// let err = TryRecvError::Closed;
/// assert_eq!(err.to_string(), "Mailbox is closed");
/// ```
#[derive(Debug, thiserror::Error)]
pub enum TryRecvError {
    /// Mailbox is empty (no messages available)
    #[error("Mailbox is empty")]
    Empty,

    /// Mailbox is closed (receiver dropped)
    #[error("Mailbox is closed")]
    Closed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mailbox_capacity_bounded() {
        let cap = MailboxCapacity::Bounded(100);
        assert_eq!(cap, MailboxCapacity::Bounded(100));

        match cap {
            MailboxCapacity::Bounded(size) => assert_eq!(size, 100),
            MailboxCapacity::Unbounded => {
                // Fail test if we get Unbounded instead of Bounded
                assert_eq!(
                    cap,
                    MailboxCapacity::Bounded(100),
                    "Expected Bounded variant"
                );
            }
        }
    }

    #[test]
    fn test_mailbox_capacity_unbounded() {
        let cap = MailboxCapacity::Unbounded;
        assert_eq!(cap, MailboxCapacity::Unbounded);
        assert_ne!(cap, MailboxCapacity::Bounded(100));
    }

    #[test]
    fn test_mailbox_capacity_equality() {
        assert_eq!(MailboxCapacity::Bounded(100), MailboxCapacity::Bounded(100));
        assert_ne!(MailboxCapacity::Bounded(100), MailboxCapacity::Bounded(200));
        assert_eq!(MailboxCapacity::Unbounded, MailboxCapacity::Unbounded);
    }

    #[test]
    fn test_mailbox_error_full() {
        let err = MailboxError::Full { capacity: 100 };
        let msg = err.to_string();
        assert!(msg.contains("full"));
        assert!(msg.contains("100"));
    }

    #[test]
    fn test_mailbox_error_closed() {
        let err = MailboxError::Closed;
        assert_eq!(err.to_string(), "Mailbox is closed");
    }

    #[test]
    fn test_mailbox_error_ttl_expired() {
        let now = Utc::now();
        let err = MailboxError::TtlExpired { timestamp: now };
        let msg = err.to_string();
        assert!(msg.contains("TTL expired"));
    }

    #[test]
    fn test_try_recv_error_empty() {
        let err = TryRecvError::Empty;
        assert_eq!(err.to_string(), "Mailbox is empty");
    }

    #[test]
    fn test_try_recv_error_closed() {
        let err = TryRecvError::Closed;
        assert_eq!(err.to_string(), "Mailbox is closed");
    }
}
