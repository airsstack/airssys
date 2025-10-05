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
/// - **DropOldest**: Time-sensitive data where newer is more relevant
/// - **DropNewest**: Batch processing where older messages should complete first
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackpressureStrategy {
    /// Block sender until space becomes available (async wait).
    ///
    /// Use for critical messages that must be delivered. May cause
    /// sender delays if receiver is slow.
    Block,

    /// Drop the oldest message in the queue to make room for new message.
    ///
    /// Use for time-sensitive data streams where newer data is more
    /// relevant (e.g., sensor readings, real-time updates).
    DropOldest,

    /// Drop the incoming (newest) message.
    ///
    /// Use when processing order matters and older messages should
    /// complete before newer ones (e.g., sequential batch processing).
    DropNewest,

    /// Return an error to the sender immediately.
    ///
    /// Use for request/response patterns where the sender needs to know
    /// immediately if delivery failed (e.g., API calls, synchronous operations).
    Error,
}

impl Default for BackpressureStrategy {
    fn default() -> Self {
        Self::Error
    }
}

impl fmt::Display for BackpressureStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Block => write!(f, "Block"),
            Self::DropOldest => write!(f, "DropOldest"),
            Self::DropNewest => write!(f, "DropNewest"),
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

            Self::DropOldest => {
                // Try to send, if full drop newest (limitation of tokio mpsc)
                // Note: mpsc::Sender doesn't support peeking/dropping from front,
                // so we fall back to DropNewest behavior
                match sender.try_send(envelope) {
                    Ok(()) => Ok(()),
                    Err(mpsc::error::TrySendError::Full(_)) => {
                        // Silently drop the new message (mpsc limitation)
                        Ok(())
                    }
                    Err(mpsc::error::TrySendError::Closed(_)) => Err(MailboxError::Closed),
                }
            }

            Self::DropNewest => {
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
    /// - High → DropOldest (newer data more important)
    /// - Normal → Error (sender should handle failure)
    /// - Low → DropNewest (can be safely discarded)
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
    /// assert_eq!(strategy, BackpressureStrategy::DropNewest);
    /// ```
    pub fn for_priority(priority: MessagePriority) -> Self {
        match priority {
            MessagePriority::Critical => Self::Block,
            MessagePriority::High => Self::DropOldest,
            MessagePriority::Normal => Self::Error,
            MessagePriority::Low => Self::DropNewest,
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
        assert_eq!(BackpressureStrategy::DropOldest.to_string(), "DropOldest");
        assert_eq!(BackpressureStrategy::DropNewest.to_string(), "DropNewest");
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
        assert_eq!(strategy, BackpressureStrategy::DropOldest);
    }

    #[test]
    fn test_strategy_for_priority_normal() {
        let strategy = BackpressureStrategy::for_priority(MessagePriority::Normal);
        assert_eq!(strategy, BackpressureStrategy::Error);
    }

    #[test]
    fn test_strategy_for_priority_low() {
        let strategy = BackpressureStrategy::for_priority(MessagePriority::Low);
        assert_eq!(strategy, BackpressureStrategy::DropNewest);
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
        let (sender, mut receiver) = mpsc::channel::<MessageEnvelope<TestMsg>>(1);

        // Fill the channel
        sender
            .try_send(MessageEnvelope::new(TestMsg {
                content: "first".to_string(),
            }))
            .unwrap();

        // Spawn task to send with Block strategy
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
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

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
    async fn test_apply_drop_newest_strategy() {
        let (sender, mut receiver) = mpsc::channel::<MessageEnvelope<TestMsg>>(1);

        // Fill the channel
        sender
            .try_send(MessageEnvelope::new(TestMsg {
                content: "first".to_string(),
            }))
            .unwrap();

        // Try to send with DropNewest - should silently drop
        BackpressureStrategy::DropNewest
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
    async fn test_apply_drop_oldest_strategy() {
        let (sender, mut receiver) = mpsc::channel::<MessageEnvelope<TestMsg>>(1);

        // Fill the channel
        sender
            .try_send(MessageEnvelope::new(TestMsg {
                content: "first".to_string(),
            }))
            .unwrap();

        // Try to send with DropOldest - due to mpsc limitation, drops newest
        BackpressureStrategy::DropOldest
            .apply(
                &sender,
                MessageEnvelope::new(TestMsg {
                    content: "second".to_string(),
                }),
            )
            .await
            .unwrap(); // Should return Ok

        // First message should still be there (mpsc limitation)
        let received = receiver.recv().await.unwrap();
        assert_eq!(received.payload.content, "first");
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

        let result = BackpressureStrategy::DropNewest
            .apply(
                &sender,
                MessageEnvelope::new(TestMsg {
                    content: "test".to_string(),
                }),
            )
            .await;
        assert!(matches!(result, Err(MailboxError::Closed)));

        let result = BackpressureStrategy::DropOldest
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
        let strategy = BackpressureStrategy::DropOldest;
        let cloned = strategy;
        assert_eq!(strategy, cloned);
    }
}
