// Layer 1: Standard library imports
use std::sync::Arc;

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use chrono::Utc;
use tokio::sync::mpsc;

// Layer 3: Internal module imports
use super::backpressure::BackpressureStrategy;
use super::metrics::{AtomicMetrics, MetricsRecorder};
use super::traits::{MailboxCapacity, MailboxError, MailboxReceiver, MailboxSender, TryRecvError};
use crate::message::{Message, MessageEnvelope};

/// Bounded mailbox with configurable capacity and backpressure handling.
///
/// BoundedMailbox uses tokio mpsc channels for async message passing
/// with a fixed maximum capacity. When the mailbox is full, the configured
/// backpressure strategy determines how new messages are handled.
///
/// # Type Parameters
///
/// * `M` - The message type implementing [`Message`]
/// * `R` - The metrics recorder implementing [`MetricsRecorder`] (default: [`AtomicMetrics`])
///
/// # Example
///
/// ```ignore
/// use airssys_rt::mailbox::{BoundedMailbox, BackpressureStrategy, AtomicMetrics};
/// use airssys_rt::message::{Message, MessageEnvelope};
///
/// #[derive(Debug, Clone)]
/// struct MyMessage;
/// impl Message for MyMessage {
///     const MESSAGE_TYPE: &'static str = "my_message";
/// }
///
/// // Create bounded mailbox with capacity 100 and default metrics
/// let (mailbox, sender) = BoundedMailbox::<MyMessage, AtomicMetrics>::new(100);
/// ```
pub struct BoundedMailbox<M: Message, R: MetricsRecorder> {
    receiver: mpsc::Receiver<MessageEnvelope<M>>,
    capacity: usize,
    pub metrics: Arc<R>,
}

/// Sender for bounded mailbox with backpressure support.
#[derive(Clone)]
pub struct BoundedMailboxSender<M: Message, R: MetricsRecorder> {
    sender: mpsc::Sender<MessageEnvelope<M>>,
    backpressure_strategy: Arc<BackpressureStrategy>,
    capacity: usize,
    pub metrics: Arc<R>,
}

impl<M: Message, R: MetricsRecorder> BoundedMailbox<M, R> {
    /// Create a new bounded mailbox with custom metrics recorder.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use airssys_rt::mailbox::{BoundedMailbox, AtomicMetrics};
    /// use airssys_rt::message::Message;
    ///
    /// #[derive(Debug, Clone)]
    /// struct MyMsg;
    /// impl Message for MyMsg {
    ///     const MESSAGE_TYPE: &'static str = "my_msg";
    /// }
    ///
    /// let metrics = AtomicMetrics::new();
    /// let (mailbox, sender) = BoundedMailbox::with_metrics(100, metrics);
    /// ```
    pub fn with_metrics(capacity: usize, metrics: R) -> (Self, BoundedMailboxSender<M, R>) {
        Self::with_backpressure_and_metrics(capacity, BackpressureStrategy::Error, metrics)
    }

    /// Create a bounded mailbox with custom backpressure strategy and metrics recorder.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use airssys_rt::mailbox::{BoundedMailbox, BackpressureStrategy, AtomicMetrics};
    /// # use airssys_rt::message::Message;
    /// # #[derive(Debug, Clone)]
    /// # struct MyMsg;
    /// # impl Message for MyMsg { const MESSAGE_TYPE: &'static str = "my_msg"; }
    ///
    /// let metrics = AtomicMetrics::new();
    /// let (mailbox, sender) = BoundedMailbox::with_backpressure_and_metrics(
    ///     100,
    ///     BackpressureStrategy::DropOldest,
    ///     metrics
    /// );
    /// ```
    pub fn with_backpressure_and_metrics(
        capacity: usize,
        strategy: BackpressureStrategy,
        metrics: R,
    ) -> (Self, BoundedMailboxSender<M, R>) {
        let (sender, receiver) = mpsc::channel(capacity);
        let metrics = Arc::new(metrics);

        let mailbox = Self {
            receiver,
            capacity,
            metrics: Arc::clone(&metrics),
        };

        let sender = BoundedMailboxSender {
            sender,
            backpressure_strategy: Arc::new(strategy),
            capacity,
            metrics,
        };

        (mailbox, sender)
    }
}

// Convenience constructors for AtomicMetrics (common case)
impl<M: Message> BoundedMailbox<M, AtomicMetrics> {
    /// Create a new bounded mailbox with default backpressure strategy and AtomicMetrics.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use airssys_rt::mailbox::BoundedMailbox;
    /// use airssys_rt::message::Message;
    ///
    /// #[derive(Debug, Clone)]
    /// struct MyMsg;
    /// impl Message for MyMsg {
    ///     const MESSAGE_TYPE: &'static str = "my_msg";
    /// }
    ///
    /// let (mailbox, sender) = BoundedMailbox::new(100);
    /// ```
    pub fn new(capacity: usize) -> (Self, BoundedMailboxSender<M, AtomicMetrics>) {
        Self::with_metrics(capacity, AtomicMetrics::new())
    }

    /// Create a bounded mailbox with custom backpressure strategy and AtomicMetrics.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use airssys_rt::mailbox::{BoundedMailbox, BackpressureStrategy};
    /// # use airssys_rt::message::Message;
    /// # #[derive(Debug, Clone)]
    /// # struct MyMsg;
    /// # impl Message for MyMsg { const MESSAGE_TYPE: &'static str = "my_msg"; }
    ///
    /// let (mailbox, sender) = BoundedMailbox::with_backpressure(
    ///     100,
    ///     BackpressureStrategy::DropOldest
    /// );
    /// ```
    pub fn with_backpressure(
        capacity: usize,
        strategy: BackpressureStrategy,
    ) -> (Self, BoundedMailboxSender<M, AtomicMetrics>) {
        Self::with_backpressure_and_metrics(capacity, strategy, AtomicMetrics::new())
    }
}

#[async_trait]
impl<M: Message, R: MetricsRecorder> MailboxReceiver<M> for BoundedMailbox<M, R> {
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
                        return self.recv().await; // Try next message
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
                // Check TTL
                if let Some(ttl) = envelope.ttl {
                    let elapsed = Utc::now()
                        .signed_duration_since(envelope.timestamp)
                        .num_seconds() as u64;

                    if elapsed > ttl {
                        self.metrics.record_dropped();
                        return self.try_recv(); // Try next
                    }
                }

                self.metrics.record_received();
                self.metrics.update_last_message(Utc::now());
                Ok(envelope)
            }
            Err(mpsc::error::TryRecvError::Empty) => Err(TryRecvError::Empty),
            Err(mpsc::error::TryRecvError::Disconnected) => Err(TryRecvError::Closed),
        }
    }

    fn capacity(&self) -> MailboxCapacity {
        MailboxCapacity::Bounded(self.capacity)
    }

    fn len(&self) -> usize {
        // Use MetricsRecorder's in_flight() method
        self.metrics.in_flight() as usize
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[async_trait]
impl<M: Message, R: MetricsRecorder + Clone> MailboxSender<M> for BoundedMailboxSender<M, R> {
    type Error = MailboxError;

    async fn send(&self, envelope: MessageEnvelope<M>) -> Result<(), Self::Error> {
        // Apply backpressure strategy
        self.backpressure_strategy
            .apply(&self.sender, envelope)
            .await?;

        self.metrics.record_sent();
        Ok(())
    }

    fn try_send(&self, envelope: MessageEnvelope<M>) -> Result<(), Self::Error> {
        self.sender.try_send(envelope).map_err(|e| match e {
            mpsc::error::TrySendError::Full(_) => MailboxError::Full {
                capacity: self.capacity,
            },
            mpsc::error::TrySendError::Closed(_) => MailboxError::Closed,
        })?;

        self.metrics.record_sent();
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)] // Tests are allowed to use unwrap for simplicity
mod tests {
    use super::*;
    use crate::message::MessagePriority;

    #[derive(Debug, Clone)]
    struct TestMessage {
        content: String,
    }

    impl Message for TestMessage {
        const MESSAGE_TYPE: &'static str = "test_message";
    }

    #[tokio::test]
    async fn test_bounded_mailbox_creation() {
        let (mailbox, _sender): (BoundedMailbox<TestMessage, _>, _) = BoundedMailbox::new(10);
        assert_eq!(mailbox.capacity(), MailboxCapacity::Bounded(10));
        assert_eq!(mailbox.len(), 0);
        assert!(mailbox.is_empty());
    }

    #[tokio::test]
    async fn test_send_receive() {
        let (mut mailbox, sender) = BoundedMailbox::new(10);

        let msg = TestMessage {
            content: "test".to_string(),
        };
        let envelope = MessageEnvelope::new(msg);

        sender.send(envelope).await.unwrap();

        let received = mailbox.recv().await.unwrap();
        assert_eq!(received.payload.content, "test");
    }

    #[tokio::test]
    async fn test_bounded_capacity_enforcement() {
        let (mut _mailbox, sender) = BoundedMailbox::new(2);

        // Fill the mailbox
        sender
            .try_send(MessageEnvelope::new(TestMessage {
                content: "1".to_string(),
            }))
            .unwrap();
        sender
            .try_send(MessageEnvelope::new(TestMessage {
                content: "2".to_string(),
            }))
            .unwrap();

        // Third message should fail
        let result = sender.try_send(MessageEnvelope::new(TestMessage {
            content: "3".to_string(),
        }));
        assert!(matches!(result, Err(MailboxError::Full { .. })));
    }

    #[tokio::test]
    async fn test_try_recv_empty() {
        let (mut mailbox, _sender): (BoundedMailbox<TestMessage, _>, _) = BoundedMailbox::new(10);
        let result = mailbox.try_recv();
        assert!(matches!(result, Err(TryRecvError::Empty)));
    }

    #[tokio::test]
    async fn test_metrics_tracking() {
        let (mut mailbox, sender) = BoundedMailbox::new(10);

        let envelope = MessageEnvelope::new(TestMessage {
            content: "test".to_string(),
        });

        sender.send(envelope).await.unwrap();
        assert_eq!(mailbox.metrics.sent_count(), 1);

        let _received = mailbox.recv().await.unwrap();
        assert_eq!(mailbox.metrics.received_count(), 1);
    }

    #[tokio::test]
    async fn test_multiple_senders() {
        let (mut mailbox, sender) = BoundedMailbox::new(10);

        let sender2 = sender.clone();

        sender
            .send(MessageEnvelope::new(TestMessage {
                content: "sender1".to_string(),
            }))
            .await
            .unwrap();
        sender2
            .send(MessageEnvelope::new(TestMessage {
                content: "sender2".to_string(),
            }))
            .await
            .unwrap();

        let msg1 = mailbox.recv().await.unwrap();
        let msg2 = mailbox.recv().await.unwrap();

        assert!(msg1.payload.content == "sender1" || msg1.payload.content == "sender2");
        assert!(msg2.payload.content == "sender1" || msg2.payload.content == "sender2");
    }

    #[tokio::test]
    async fn test_closed_mailbox() {
        let (_mailbox, sender) = BoundedMailbox::new(10);

        // Drop the receiver
        drop(_mailbox);

        // Sending should fail
        let result = sender
            .send(MessageEnvelope::new(TestMessage {
                content: "test".to_string(),
            }))
            .await;
        assert!(matches!(result, Err(MailboxError::Closed)));
    }

    #[tokio::test]
    async fn test_with_backpressure_strategy() {
        let (mailbox, _sender): (BoundedMailbox<TestMessage, _>, _) =
            BoundedMailbox::with_backpressure(10, BackpressureStrategy::Drop);
        assert_eq!(mailbox.capacity(), MailboxCapacity::Bounded(10));
    }

    #[tokio::test]
    async fn test_len_approximation() {
        let (mut mailbox, sender) = BoundedMailbox::new(10);

        sender
            .send(MessageEnvelope::new(TestMessage {
                content: "1".to_string(),
            }))
            .await
            .unwrap();
        sender
            .send(MessageEnvelope::new(TestMessage {
                content: "2".to_string(),
            }))
            .await
            .unwrap();

        assert_eq!(mailbox.len(), 2);
        assert!(!mailbox.is_empty());

        let _msg = mailbox.recv().await.unwrap();
        assert_eq!(mailbox.len(), 1);
    }

    #[tokio::test]
    async fn test_ttl_expiration() {
        use std::time::Duration;
        use tokio::time::sleep;

        let (mut mailbox, sender) = BoundedMailbox::new(10);

        let msg = TestMessage {
            content: "expired".to_string(),
        };
        // Create message with 1 second TTL
        let mut envelope = MessageEnvelope::new(msg);
        envelope.ttl = Some(1);

        sender.send(envelope).await.unwrap();

        // Sleep for 2 seconds to ensure TTL expires
        sleep(Duration::from_secs(2)).await;

        // Send a valid message
        let valid_msg = MessageEnvelope::new(TestMessage {
            content: "valid".to_string(),
        });
        sender.send(valid_msg).await.unwrap();

        // Should skip expired message and return valid one
        let received = mailbox.recv().await.unwrap();
        assert_eq!(received.payload.content, "valid");

        // Check that one message was dropped
        assert_eq!(mailbox.metrics.dropped_count(), 1);
    }

    #[tokio::test]
    async fn test_priority_message() {
        let (mut mailbox, sender) = BoundedMailbox::new(10);

        let msg = TestMessage {
            content: "high priority".to_string(),
        };
        let mut envelope = MessageEnvelope::new(msg);
        envelope.priority = MessagePriority::High;

        sender.send(envelope).await.unwrap();

        let received = mailbox.recv().await.unwrap();
        assert_eq!(received.priority, MessagePriority::High);
    }
}
