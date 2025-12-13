//! Backpressure strategies for handling full mailboxes.
//!
//! This module provides backpressure handling strategies for bounded mailboxes,
//! allowing fine-grained control over message delivery behavior when mailboxes
//! reach capacity.

// Layer 1: Standard library imports
use std::fmt;

// Layer 2: Third-party crate imports
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

// Layer 3: Internal module imports
use super::traits::MailboxError;
use crate::message::{Message, MessageEnvelope, MessagePriority};

/// Backpressure strategies for handling full mailboxes.
///
/// When a bounded mailbox reaches capacity, the backpressure strategy
/// determines how the system handles additional incoming messages.
///
/// # Strategy Selection
///
/// Different strategies are appropriate for different scenarios:
/// - **Block**: Critical messages that must be delivered (may cause sender delays)
/// - **Drop**: Low priority messages where silent failure is acceptable
/// - **Error**: Request/response patterns where sender needs immediate feedback
///
/// # Example
///
/// ```ignore
/// use airssys_rt::mailbox::{BackpressureStrategy, BoundedMailbox};
/// use airssys_rt::message::MessagePriority;
///
/// // Select strategy based on message priority
/// let strategy = BackpressureStrategy::for_priority(MessagePriority::Critical);
/// assert_eq!(strategy, BackpressureStrategy::Block);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BackpressureStrategy {
    /// Block sender until space becomes available (async wait).
    ///
    /// Use for critical messages that must be delivered. May cause
    /// sender delays if receiver is slow.
    Block,

    /// Drop the incoming message when mailbox is full.
    ///
    /// Use for low-priority messages or scenarios where silent message
    /// dropping is acceptable (e.g., best-effort delivery, logging, metrics).
    Drop,

    /// Return an error to the sender immediately.
    ///
    /// Use for request/response patterns where the sender needs to know
    /// immediately if delivery failed (e.g., API calls, synchronous operations).
    #[default]
    Error,
}

impl fmt::Display for BackpressureStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Block => write!(f, "Block"),
            Self::Drop => write!(f, "Drop"),
            Self::Error => write!(f, "Error"),
        }
    }
}

impl BackpressureStrategy {
    /// Apply the backpressure strategy to send a message.
    ///
    /// This method handles the actual message delivery according to the
    /// configured strategy when the mailbox is full.
    ///
    /// # Errors
    ///
    /// Returns `MailboxError::Full` if strategy is Error and mailbox is full.
    /// Returns `MailboxError::Closed` if the receiver is closed.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use airssys_rt::mailbox::BackpressureStrategy;
    /// use airssys_rt::message::MessageEnvelope;
    ///
    /// let strategy = BackpressureStrategy::Block;
    /// strategy.apply(&sender, envelope).await?;
    /// ```
    pub async fn apply<M: Message>(
        &self,
        sender: &mpsc::Sender<MessageEnvelope<M>>,
        envelope: MessageEnvelope<M>,
    ) -> Result<(), MailboxError> {
        match self {
            Self::Block => {
                // Wait for space (async blocking)
                sender
                    .send(envelope)
                    .await
                    .map_err(|_| MailboxError::Closed)?;
                Ok(())
            }

            Self::Drop => {
                // Drop incoming message if mailbox is full
                match sender.try_send(envelope) {
                    Ok(()) => Ok(()),
                    Err(mpsc::error::TrySendError::Full(_)) => {
                        // Silently drop the incoming message
                        Ok(())
                    }
                    Err(mpsc::error::TrySendError::Closed(_)) => Err(MailboxError::Closed),
                }
            }

            Self::Error => {
                // Return error immediately if full
                sender.try_send(envelope).map_err(|e| match e {
                    mpsc::error::TrySendError::Full(_) => MailboxError::Full {
                        capacity: sender.max_capacity(),
                    },
                    mpsc::error::TrySendError::Closed(_) => MailboxError::Closed,
                })
            }
        }
    }

    /// Select appropriate backpressure strategy based on message priority.
    ///
    /// # Strategy Mapping
    ///
    /// - Critical → Block (must be delivered)
    /// - High → Block (important messages)
    /// - Normal → Error (sender should handle failure)
    /// - Low → Drop (can be safely discarded)
    ///
    /// # Example
    ///
    /// ```
    /// use airssys_rt::mailbox::BackpressureStrategy;
    /// use airssys_rt::message::MessagePriority;
    ///
    /// let strategy = BackpressureStrategy::for_priority(MessagePriority::Critical);
    /// assert_eq!(strategy, BackpressureStrategy::Block);
    ///
    /// let strategy = BackpressureStrategy::for_priority(MessagePriority::Low);
    /// assert_eq!(strategy, BackpressureStrategy::Drop);
    /// ```
    pub fn for_priority(priority: MessagePriority) -> Self {
        match priority {
            MessagePriority::Critical => Self::Block,
            MessagePriority::High => Self::Block,
            MessagePriority::Normal => Self::Error,
            MessagePriority::Low => Self::Drop,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)] // Tests are allowed to use unwrap for simplicity
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestMsg {
        content: String,
    }

    impl Message for TestMsg {
        const MESSAGE_TYPE: &'static str = "test_msg";
    }

    #[test]
    fn test_backpressure_strategy_default() {
        assert_eq!(BackpressureStrategy::default(), BackpressureStrategy::Error);
    }

    #[test]
    fn test_backpressure_strategy_display() {
        assert_eq!(BackpressureStrategy::Block.to_string(), "Block");
        assert_eq!(BackpressureStrategy::Drop.to_string(), "Drop");
        assert_eq!(BackpressureStrategy::Error.to_string(), "Error");
    }

    #[test]
    fn test_strategy_for_priority_critical() {
        let strategy = BackpressureStrategy::for_priority(MessagePriority::Critical);
        assert_eq!(strategy, BackpressureStrategy::Block);
    }

    #[test]
    fn test_strategy_for_priority_high() {
        let strategy = BackpressureStrategy::for_priority(MessagePriority::High);
        assert_eq!(strategy, BackpressureStrategy::Block);
    }

    #[test]
    fn test_strategy_for_priority_normal() {
        let strategy = BackpressureStrategy::for_priority(MessagePriority::Normal);
        assert_eq!(strategy, BackpressureStrategy::Error);
    }

    #[test]
    fn test_strategy_for_priority_low() {
        let strategy = BackpressureStrategy::for_priority(MessagePriority::Low);
        assert_eq!(strategy, BackpressureStrategy::Drop);
    }

    #[tokio::test]
    async fn test_apply_block_strategy() {
        let (sender, mut receiver) = mpsc::channel::<MessageEnvelope<TestMsg>>(2);
        let envelope = MessageEnvelope::new(TestMsg {
            content: "test".to_string(),
        });

        // Should succeed even when channel has capacity
        BackpressureStrategy::Block
            .apply(&sender, envelope)
            .await
            .unwrap();

        // Verify message was sent
        let received = receiver.recv().await.unwrap();
        assert_eq!(received.payload.content, "test");
    }

    #[tokio::test]
    async fn test_apply_block_strategy_waits() {
        use std::time::Duration;
        use tokio::time::sleep;

        let (sender, mut receiver) = mpsc::channel::<MessageEnvelope<TestMsg>>(1);

        // Fill the channel
        sender
            .try_send(MessageEnvelope::new(TestMsg {
                content: "first".to_string(),
            }))
            .unwrap();

        // Try to send another message (will block)
        let sender_clone = sender.clone();
        let handle = tokio::spawn(async move {
            BackpressureStrategy::Block
                .apply(
                    &sender_clone,
                    MessageEnvelope::new(TestMsg {
                        content: "second".to_string(),
                    }),
                )
                .await
        });

        // Give the task a moment to start
        sleep(Duration::from_millis(10)).await;

        // Receive first message to make space
        receiver.recv().await.unwrap();

        // The blocked send should now complete
        handle.await.unwrap().unwrap();

        // Verify second message was delivered
        let second = receiver.recv().await.unwrap();
        assert_eq!(second.payload.content, "second");
    }

    #[tokio::test]
    async fn test_apply_error_strategy_full() {
        let (sender, _receiver) = mpsc::channel::<MessageEnvelope<TestMsg>>(1);

        // Fill the channel
        sender
            .try_send(MessageEnvelope::new(TestMsg {
                content: "first".to_string(),
            }))
            .unwrap();

        // Next should fail with Error strategy
        let result = BackpressureStrategy::Error
            .apply(
                &sender,
                MessageEnvelope::new(TestMsg {
                    content: "second".to_string(),
                }),
            )
            .await;

        assert!(matches!(result, Err(MailboxError::Full { .. })));
    }

    #[tokio::test]
    async fn test_apply_error_strategy_success() {
        let (sender, mut receiver) = mpsc::channel::<MessageEnvelope<TestMsg>>(2);

        // Should succeed when channel has space
        BackpressureStrategy::Error
            .apply(
                &sender,
                MessageEnvelope::new(TestMsg {
                    content: "test".to_string(),
                }),
            )
            .await
            .unwrap();

        let received = receiver.recv().await.unwrap();
        assert_eq!(received.payload.content, "test");
    }

    #[tokio::test]
    async fn test_apply_drop_strategy() {
        let (sender, mut receiver) = mpsc::channel::<MessageEnvelope<TestMsg>>(1);

        // Fill the channel
        sender
            .try_send(MessageEnvelope::new(TestMsg {
                content: "first".to_string(),
            }))
            .unwrap();

        // Try to send with Drop - should silently drop incoming message
        BackpressureStrategy::Drop
            .apply(
                &sender,
                MessageEnvelope::new(TestMsg {
                    content: "second".to_string(),
                }),
            )
            .await
            .unwrap(); // Should return Ok

        // Only first message should be in channel
        let received = receiver.recv().await.unwrap();
        assert_eq!(received.payload.content, "first");

        // Channel should be empty now
        assert!(receiver.try_recv().is_err());
    }

    #[tokio::test]
    async fn test_apply_closed_channel() {
        let (sender, receiver) = mpsc::channel::<MessageEnvelope<TestMsg>>(1);

        // Drop receiver to close channel
        drop(receiver);

        // All strategies should return Closed error
        let result = BackpressureStrategy::Block
            .apply(
                &sender,
                MessageEnvelope::new(TestMsg {
                    content: "test".to_string(),
                }),
            )
            .await;
        assert!(matches!(result, Err(MailboxError::Closed)));

        let result = BackpressureStrategy::Error
            .apply(
                &sender,
                MessageEnvelope::new(TestMsg {
                    content: "test".to_string(),
                }),
            )
            .await;
        assert!(matches!(result, Err(MailboxError::Closed)));

        let result = BackpressureStrategy::Drop
            .apply(
                &sender,
                MessageEnvelope::new(TestMsg {
                    content: "test".to_string(),
                }),
            )
            .await;
        assert!(matches!(result, Err(MailboxError::Closed)));
    }

    #[test]
    fn test_strategy_equality() {
        assert_eq!(BackpressureStrategy::Block, BackpressureStrategy::Block);
        assert_ne!(BackpressureStrategy::Block, BackpressureStrategy::Error);
    }

    #[test]
    fn test_strategy_clone() {
        let strategy = BackpressureStrategy::Drop;
        let cloned = strategy;
        assert_eq!(strategy, cloned);
    }
}
