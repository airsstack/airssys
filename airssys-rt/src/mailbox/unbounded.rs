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
use std::sync::Arc;

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use chrono::Utc;
use tokio::sync::mpsc;

// Layer 3: Internal module imports
use super::metrics::{AtomicMetrics, MetricsRecorder};
use super::traits::{MailboxCapacity, MailboxError, MailboxReceiver, MailboxSender, TryRecvError};
use crate::message::{Message, MessageEnvelope};

/// Unbounded mailbox with unlimited capacity.
///
/// UnboundedMailbox uses tokio's unbounded mpsc channel for async message passing
/// without capacity limits. Messages are never dropped or blocked, but this can
/// lead to unbounded memory growth if not managed carefully.
///
/// # Type Parameters
///
/// * `M` - The message type implementing [`Message`]
/// * `R` - The metrics recorder implementing [`MetricsRecorder`] (default: [`AtomicMetrics`])
///
/// # Memory Safety
///
/// While the mailbox itself is unbounded, system memory is finite. Monitor
/// mailbox metrics to detect potential memory issues.
///
/// # Example
///
/// ```ignore
/// use airssys_rt::mailbox::{UnboundedMailbox, AtomicMetrics};
/// use airssys_rt::message::{Message, MessageEnvelope};
///
/// #[derive(Debug, Clone)]
/// struct MyMessage;
/// impl Message for MyMessage {
///     const MESSAGE_TYPE: &'static str = "my_message";
/// }
///
/// // Create unbounded mailbox with default metrics
/// let (mailbox, sender) = UnboundedMailbox::<MyMessage, AtomicMetrics>::new();
/// ```
pub struct UnboundedMailbox<M: Message, R: MetricsRecorder> {
    receiver: mpsc::UnboundedReceiver<MessageEnvelope<M>>,
    pub metrics: Arc<R>,
}

/// Sender for unbounded mailbox.
///
/// The sender can send messages without ever blocking or failing due to
/// capacity limits. Clone the sender to share it across multiple tasks.
#[derive(Clone)]
pub struct UnboundedMailboxSender<M: Message, R: MetricsRecorder> {
    sender: mpsc::UnboundedSender<MessageEnvelope<M>>,
    pub metrics: Arc<R>,
}

impl<M: Message, R: MetricsRecorder> UnboundedMailbox<M, R> {
    /// Create a new unbounded mailbox with custom metrics recorder.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use airssys_rt::mailbox::{UnboundedMailbox, AtomicMetrics};
    /// use airssys_rt::message::Message;
    ///
    /// #[derive(Debug, Clone)]
    /// struct MyMsg;
    /// impl Message for MyMsg {
    ///     const MESSAGE_TYPE: &'static str = "my_msg";
    /// }
    ///
    /// let metrics = AtomicMetrics::new();
    /// let (mailbox, sender) = UnboundedMailbox::with_metrics(metrics);
    /// ```
    pub fn with_metrics(metrics: R) -> (Self, UnboundedMailboxSender<M, R>) {
        let (sender, receiver) = mpsc::unbounded_channel();
        let metrics = Arc::new(metrics);

        let mailbox = Self {
            receiver,
            metrics: Arc::clone(&metrics),
        };

        let sender = UnboundedMailboxSender { sender, metrics };

        (mailbox, sender)
    }
}

// Convenience constructor for AtomicMetrics (common case)
impl<M: Message> UnboundedMailbox<M, AtomicMetrics> {
    /// Create a new unbounded mailbox with AtomicMetrics.
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
    /// let (mailbox, sender) = UnboundedMailbox::new();
    /// ```
    pub fn new() -> (Self, UnboundedMailboxSender<M, AtomicMetrics>) {
        Self::with_metrics(AtomicMetrics::new())
    }
}

#[async_trait]
impl<M: Message, R: MetricsRecorder> MailboxReceiver<M> for UnboundedMailbox<M, R> {
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
                        self.metrics.record_dropped();
                        return self.recv().await; // Try next message (recursive)
                    }
                }

                // Update metrics
                self.metrics.record_received();
                self.metrics.update_last_message(Utc::now()); // §3.2

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
                        self.metrics.record_dropped();
                        return self.try_recv(); // Try next message (recursive)
                    }
                }

                // Update metrics
                self.metrics.record_received();
                self.metrics.update_last_message(Utc::now()); // §3.2

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
        self.metrics.in_flight() as usize
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[async_trait]
impl<M: Message, R: MetricsRecorder + Clone> MailboxSender<M> for UnboundedMailboxSender<M, R> {
    type Error = MailboxError;

    async fn send(&self, envelope: MessageEnvelope<M>) -> Result<(), Self::Error> {
        // Unbounded send never blocks or fails due to capacity
        self.sender
            .send(envelope)
            .map_err(|_| MailboxError::Closed)?;

        // Update metrics after sending
        self.metrics.record_sent();
        Ok(())
    }

    fn try_send(&self, envelope: MessageEnvelope<M>) -> Result<(), Self::Error> {
        // Unbounded send never fails due to capacity, only if closed
        self.sender
            .send(envelope)
            .map_err(|_| MailboxError::Closed)?;

        // Update metrics after sending
        self.metrics.record_sent();
        Ok(())
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
        let (mailbox, _sender): (UnboundedMailbox<TestMessage, _>, _) = UnboundedMailbox::new();
        assert_eq!(mailbox.capacity(), MailboxCapacity::Unbounded);
        assert_eq!(mailbox.len(), 0);
        assert!(mailbox.is_empty());
    }

    #[tokio::test]
    async fn test_unbounded_send_and_recv() {
        let (mut mailbox, sender) = UnboundedMailbox::new();
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
        let (mut mailbox, sender) = UnboundedMailbox::new();
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
        let (mut mailbox, _sender): (UnboundedMailbox<TestMessage, _>, _) = UnboundedMailbox::new();

        match mailbox.try_recv() {
            Err(TryRecvError::Empty) => { /* expected */ }
            _ => panic!("Expected Empty error"),
        }
    }

    #[tokio::test]
    async fn test_unbounded_multiple_messages() {
        let (mut mailbox, sender) = UnboundedMailbox::new();

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
        let (mut mailbox, sender) = UnboundedMailbox::new();

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
        use tokio::time::sleep;

        let (mut mailbox, sender) = UnboundedMailbox::new();

        // Send message with 1 second TTL
        let msg = TestMessage {
            data: "expired".to_string(),
        };
        let mut envelope = MessageEnvelope::new(msg);
        envelope.ttl = Some(1);

        sender.send(envelope).await.unwrap();

        // Sleep for 2 seconds to ensure TTL expires
        sleep(Duration::from_secs(2)).await;

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
        assert_eq!(mailbox.metrics.dropped_count(), 1);
    }

    #[tokio::test]
    async fn test_unbounded_metrics() {
        let (mut mailbox, sender) = UnboundedMailbox::new();

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
        assert_eq!(mailbox.metrics.sent_count(), 5);
        assert_eq!(mailbox.metrics.received_count(), 0);

        // Receive 3 messages
        for _ in 0..3 {
            mailbox.recv().await.unwrap();
        }

        // Check metrics after receiving
        assert_eq!(mailbox.metrics.received_count(), 3);
    }

    #[tokio::test]
    async fn test_unbounded_closed_mailbox() {
        let (mut mailbox, sender): (UnboundedMailbox<TestMessage, _>, _) = UnboundedMailbox::new();

        // Drop sender to close mailbox
        drop(sender);

        // recv should return None when closed and empty
        let result = mailbox.recv().await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_unbounded_closed_sender() {
        let (mailbox, sender) = UnboundedMailbox::new();

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
        let (mut mailbox, sender) = UnboundedMailbox::new();

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
        let (mailbox, _sender): (UnboundedMailbox<TestMessage, _>, _) = UnboundedMailbox::new();

        // Unbounded mailbox always reports Unbounded capacity
        assert_eq!(mailbox.capacity(), MailboxCapacity::Unbounded);
    }

    #[tokio::test]
    async fn test_unbounded_high_volume() {
        let (mut mailbox, sender) = UnboundedMailbox::new();

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
