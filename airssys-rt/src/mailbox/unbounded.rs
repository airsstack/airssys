//! Unbounded mailbox implementation with unlimited capacity.
//!
//! This module provides an unbounded mailbox that can grow without limit,
//! using tokio's unbounded channel. This is suitable for actors that need
//! to handle bursts of messages without backpressure.
//!
//! # Warning
//!
//! cannot keep up with incoming messages. Use with caution and prefer
//! bounded mailboxes with appropriate backpressure strategies.
//!
//! # Use Cases
//!
//! - System actors that must never block message delivery
//! - Supervisors that need to receive all child actor messages
//! - High-priority control plane actors
//! - Actors with known finite message sources
//!
//! # Example
//!
//! ```ignore
//! use airssys_rt::mailbox::UnboundedMailbox;
//! use airssys_rt::message::{Message, MessageEnvelope};
//!
//! #[derive(Debug, Clone)]
//! struct MyMessage;
//! impl Message for MyMessage {
//!     const MESSAGE_TYPE: &'static str = "my_message";
//! }
//!
//! // Create unbounded mailbox
//! let (mailbox, sender) = UnboundedMailbox::<MyMessage>::new();
//! ```

// Layer 1: Standard library imports
use std::sync::atomic::Ordering;
use std::sync::Arc;

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use chrono::Utc;
use tokio::sync::mpsc;

// Layer 3: Internal module imports
use super::traits::{
    MailboxCapacity, MailboxError, MailboxMetrics, MailboxReceiver, MailboxSender, TryRecvError,
};
use crate::message::{Message, MessageEnvelope};

/// Unbounded mailbox with unlimited capacity.
///
/// UnboundedMailbox uses tokio's unbounded mpsc channel for async message passing
/// without capacity limits. Messages are never dropped or blocked, but this can
/// lead to unbounded memory growth if not managed carefully.
///
/// # Memory Safety
///
/// While the mailbox itself is unbounded, system memory is finite. Monitor
/// mailbox metrics to detect potential memory issues.
///
/// # Example
///
/// ```ignore
/// use airssys_rt::mailbox::UnboundedMailbox;
/// use airssys_rt::message::{Message, MessageEnvelope};
///
/// #[derive(Debug, Clone)]
/// struct MyMessage;
/// impl Message for MyMessage {
///     const MESSAGE_TYPE: &'static str = "my_message";
/// }
///
/// // Create unbounded mailbox
/// let (mailbox, sender) = UnboundedMailbox::<MyMessage>::new();
/// ```
pub struct UnboundedMailbox<M: Message> {
    receiver: mpsc::UnboundedReceiver<MessageEnvelope<M>>,
    metrics: Arc<MailboxMetrics>,
}

/// Sender for unbounded mailbox.
///
/// The sender can send messages without ever blocking or failing due to
/// capacity limits. Clone the sender to share it across multiple tasks.
#[derive(Clone)]
pub struct UnboundedMailboxSender<M: Message> {
    sender: mpsc::UnboundedSender<MessageEnvelope<M>>,
    metrics: Arc<MailboxMetrics>,
}

impl<M: Message> UnboundedMailbox<M> {
    /// Create a new unbounded mailbox.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use airssys_rt::mailbox::UnboundedMailbox;
    /// use airssys_rt::message::Message;
    ///
    /// #[derive(Debug, Clone)]
    /// struct MyMsg;
    /// impl Message for MyMsg {
    ///     const MESSAGE_TYPE: &'static str = "my_msg";
    /// }
    ///
    /// let (mailbox, sender) = UnboundedMailbox::<MyMsg>::new();
    /// ```
    pub fn new() -> (Self, UnboundedMailboxSender<M>) {
        let (sender, receiver) = mpsc::unbounded_channel();
        let metrics = Arc::new(MailboxMetrics::default());

        let mailbox = Self {
            receiver,
            metrics: Arc::clone(&metrics),
        };

        let sender = UnboundedMailboxSender { sender, metrics };

        (mailbox, sender)
    }
}

#[async_trait]
impl<M: Message> MailboxReceiver<M> for UnboundedMailbox<M> {
    type Error = MailboxError;

    async fn recv(&mut self) -> Option<MessageEnvelope<M>> {
        match self.receiver.recv().await {
            Some(envelope) => {
                // Check TTL expiration (§3.2 chrono)
                if let Some(ttl) = envelope.ttl {
                    let elapsed = Utc::now()
                        .signed_duration_since(envelope.timestamp)
                        .num_seconds() as u64;

                    if elapsed > ttl {
                        // Message expired, skip it
                        self.metrics
                            .messages_dropped
                            .fetch_add(1, Ordering::Relaxed);
                        return self.recv().await; // Try next message (recursive)
                    }
                }

                // Update metrics
                self.metrics
                    .messages_received
                    .fetch_add(1, Ordering::Relaxed);
                *self.metrics.last_message_at.write() = Some(Utc::now()); // §3.2

                Some(envelope)
            }
            None => None,
        }
    }

    fn try_recv(&mut self) -> Result<MessageEnvelope<M>, TryRecvError> {
        match self.receiver.try_recv() {
            Ok(envelope) => {
                // Check TTL expiration (§3.2 chrono)
                if let Some(ttl) = envelope.ttl {
                    let elapsed = Utc::now()
                        .signed_duration_since(envelope.timestamp)
                        .num_seconds() as u64;

                    if elapsed > ttl {
                        // Message expired, try next
                        self.metrics
                            .messages_dropped
                            .fetch_add(1, Ordering::Relaxed);
                        return self.try_recv(); // Try next message (recursive)
                    }
                }

                // Update metrics
                self.metrics
                    .messages_received
                    .fetch_add(1, Ordering::Relaxed);
                *self.metrics.last_message_at.write() = Some(Utc::now()); // §3.2

                Ok(envelope)
            }
            Err(mpsc::error::TryRecvError::Empty) => Err(TryRecvError::Empty),
            Err(mpsc::error::TryRecvError::Disconnected) => Err(TryRecvError::Closed),
        }
    }

    fn capacity(&self) -> MailboxCapacity {
        MailboxCapacity::Unbounded
    }

    fn len(&self) -> usize {
        // Note: unbounded channels don't provide accurate len()
        // We approximate using sent - received metrics
        let sent = self.metrics.messages_sent.load(Ordering::Relaxed);
        let received = self.metrics.messages_received.load(Ordering::Relaxed);
        sent.saturating_sub(received) as usize
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn metrics(&self) -> &MailboxMetrics {
        &self.metrics
    }
}

#[async_trait]
impl<M: Message> MailboxSender<M> for UnboundedMailboxSender<M> {
    type Error = MailboxError;

    async fn send(&self, envelope: MessageEnvelope<M>) -> Result<(), Self::Error> {
        // Update metrics before sending
        self.metrics.messages_sent.fetch_add(1, Ordering::Relaxed);

        // Unbounded send never blocks or fails due to capacity
        self.sender.send(envelope).map_err(|_| MailboxError::Closed)
    }

    fn try_send(&self, envelope: MessageEnvelope<M>) -> Result<(), Self::Error> {
        // Update metrics before sending
        self.metrics.messages_sent.fetch_add(1, Ordering::Relaxed);

        // Unbounded send never fails due to capacity, only if closed
        self.sender.send(envelope).map_err(|_| MailboxError::Closed)
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::message::Message;

    #[derive(Debug, Clone, PartialEq)]
    struct TestMessage {
        data: String,
    }

    impl Message for TestMessage {
        const MESSAGE_TYPE: &'static str = "test_message";
    }

    #[tokio::test]
    async fn test_unbounded_mailbox_new() {
        let (mailbox, _sender) = UnboundedMailbox::<TestMessage>::new();
        assert_eq!(mailbox.capacity(), MailboxCapacity::Unbounded);
        assert_eq!(mailbox.len(), 0);
        assert!(mailbox.is_empty());
    }

    #[tokio::test]
    async fn test_unbounded_send_and_recv() {
        let (mut mailbox, sender) = UnboundedMailbox::<TestMessage>::new();
        let msg = TestMessage {
            data: "test".to_string(),
        };

        let envelope = MessageEnvelope::new(msg.clone());
        sender.send(envelope).await.unwrap();

        let received = mailbox.recv().await.unwrap();
        assert_eq!(received.payload.data, "test");
    }

    #[tokio::test]
    async fn test_unbounded_try_send() {
        let (mut mailbox, sender) = UnboundedMailbox::<TestMessage>::new();
        let msg = TestMessage {
            data: "test".to_string(),
        };

        let envelope = MessageEnvelope::new(msg.clone());
        sender.try_send(envelope).unwrap();

        let received = mailbox.try_recv().unwrap();
        assert_eq!(received.payload.data, "test");
    }

    #[tokio::test]
    async fn test_unbounded_try_recv_empty() {
        let (mut mailbox, _sender) = UnboundedMailbox::<TestMessage>::new();

        match mailbox.try_recv() {
            Err(TryRecvError::Empty) => { /* expected */ }
            _ => panic!("Expected Empty error"),
        }
    }

    #[tokio::test]
    async fn test_unbounded_multiple_messages() {
        let (mut mailbox, sender) = UnboundedMailbox::<TestMessage>::new();

        // Send 1000 messages without blocking (unbounded)
        for i in 0..1000 {
            let msg = TestMessage {
                data: format!("msg_{i}"),
            };
            sender.send(MessageEnvelope::new(msg)).await.unwrap();
        }

        // Receive all messages
        for i in 0..1000 {
            let received = mailbox.recv().await.unwrap();
            assert_eq!(received.payload.data, format!("msg_{i}"));
        }
    }

    #[tokio::test]
    async fn test_unbounded_sender_clone() {
        let (mut mailbox, sender) = UnboundedMailbox::<TestMessage>::new();

        let sender2 = sender.clone();

        sender
            .send(MessageEnvelope::new(TestMessage {
                data: "from_sender1".to_string(),
            }))
            .await
            .unwrap();

        sender2
            .send(MessageEnvelope::new(TestMessage {
                data: "from_sender2".to_string(),
            }))
            .await
            .unwrap();

        let msg1 = mailbox.recv().await.unwrap();
        let msg2 = mailbox.recv().await.unwrap();

        assert_eq!(msg1.payload.data, "from_sender1");
        assert_eq!(msg2.payload.data, "from_sender2");
    }

    #[tokio::test]
    async fn test_unbounded_ttl_expiration() {
        use std::time::Duration;

        let (mut mailbox, sender) = UnboundedMailbox::<TestMessage>::new();

        // Send message with 1 second TTL
        let msg = TestMessage {
            data: "expired".to_string(),
        };
        let mut envelope = MessageEnvelope::new(msg);
        envelope.ttl = Some(1);

        sender.send(envelope).await.unwrap();

        // Sleep for 2 seconds to ensure TTL expires
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Send valid message (no TTL)
        sender
            .send(MessageEnvelope::new(TestMessage {
                data: "valid".to_string(),
            }))
            .await
            .unwrap();

        // First recv should skip expired message and return valid one
        let received = mailbox.recv().await.unwrap();
        assert_eq!(received.payload.data, "valid");

        // Check metrics - expired message should be counted as dropped
        let metrics = mailbox.metrics();
        assert_eq!(metrics.messages_dropped.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn test_unbounded_metrics() {
        let (mut mailbox, sender) = UnboundedMailbox::<TestMessage>::new();

        // Send 5 messages
        for i in 0..5 {
            sender
                .send(MessageEnvelope::new(TestMessage {
                    data: format!("msg_{i}"),
                }))
                .await
                .unwrap();
        }

        // Check metrics after sending
        {
            let metrics = mailbox.metrics();
            assert_eq!(metrics.messages_sent.load(Ordering::Relaxed), 5);
            assert_eq!(metrics.messages_received.load(Ordering::Relaxed), 0);
        }

        // Receive 3 messages
        for _ in 0..3 {
            mailbox.recv().await.unwrap();
        }

        // Check metrics after receiving
        let metrics = mailbox.metrics();
        assert_eq!(metrics.messages_received.load(Ordering::Relaxed), 3);
    }

    #[tokio::test]
    async fn test_unbounded_closed_mailbox() {
        let (mut mailbox, sender) = UnboundedMailbox::<TestMessage>::new();

        // Drop sender to close mailbox
        drop(sender);

        // recv should return None when closed and empty
        let result = mailbox.recv().await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_unbounded_closed_sender() {
        let (mailbox, sender) = UnboundedMailbox::<TestMessage>::new();

        // Drop receiver to close channel
        drop(mailbox);

        // Send should fail with Closed error
        let result = sender
            .send(MessageEnvelope::new(TestMessage {
                data: "test".to_string(),
            }))
            .await;

        assert!(matches!(result, Err(MailboxError::Closed)));
    }

    #[tokio::test]
    async fn test_unbounded_envelope_builder() {
        let (mut mailbox, sender) = UnboundedMailbox::<TestMessage>::new();

        let msg = TestMessage {
            data: "test".to_string(),
        };

        let envelope = MessageEnvelope::new(msg).with_ttl(3600);

        sender.send(envelope).await.unwrap();

        let received = mailbox.recv().await.unwrap();
        assert_eq!(received.ttl, Some(3600));
    }

    #[tokio::test]
    async fn test_unbounded_capacity_reporting() {
        let (mailbox, _sender) = UnboundedMailbox::<TestMessage>::new();

        // Unbounded mailbox always reports Unbounded capacity
        assert_eq!(mailbox.capacity(), MailboxCapacity::Unbounded);
    }

    #[tokio::test]
    async fn test_unbounded_high_volume() {
        let (mut mailbox, sender) = UnboundedMailbox::<TestMessage>::new();

        // Send 10,000 messages to test unbounded behavior
        for i in 0..10_000 {
            sender
                .send(MessageEnvelope::new(TestMessage {
                    data: format!("msg_{i}"),
                }))
                .await
                .unwrap();
        }

        // Verify all messages received
        for i in 0..10_000 {
            let received = mailbox.recv().await.unwrap();
            assert_eq!(received.payload.data, format!("msg_{i}"));
        }
    }
}
