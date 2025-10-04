// Layer 1: Standard library imports
use std::fmt::Debug;

// Layer 2: Third-party crate imports
use chrono::{DateTime, Utc}; // §3.2 MANDATORY
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Layer 3: Internal module imports
use super::traits::{Message, MessagePriority};
use crate::util::ids::ActorAddress;

/// Generic message envelope with zero-cost abstraction
///
/// # Type Safety
/// The envelope is generic over the message type M, ensuring compile-time
/// type safety without runtime dispatch or type erasure.
///
/// # Stack Allocation
/// `MessageEnvelope<M>` is stack-allocated when M is stack-allocated,
/// avoiding heap overhead for message passing.
///
/// # Example
/// ```rust
/// use airssys_rt::message::{Message, MessageEnvelope, MessagePriority};
/// use airssys_rt::util::ActorAddress;
///
/// #[derive(Debug, Clone)]
/// struct MyMessage {
///     content: String,
/// }
///
/// impl Message for MyMessage {
///     const MESSAGE_TYPE: &'static str = "my_message";
/// }
///
/// let msg = MyMessage { content: "Hello".to_string() };
/// let sender = ActorAddress::named("sender");
///
/// let envelope = MessageEnvelope::new(msg)
///     .with_sender(sender)
///     .with_ttl(60);
///
/// assert_eq!(envelope.message_type(), "my_message");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEnvelope<M: Message> {
    /// The actual message payload
    pub payload: M,

    /// Optional sender address for reply capability
    pub sender: Option<ActorAddress>,

    /// Optional recipient for reply-to pattern
    pub reply_to: Option<ActorAddress>,

    /// Message creation timestamp (§3.2 chrono `DateTime<Utc>`)
    pub timestamp: DateTime<Utc>,

    /// Optional correlation ID for request/response tracking
    pub correlation_id: Option<Uuid>,

    /// Message priority (extracted from payload)
    pub priority: MessagePriority,

    /// Optional time-to-live in seconds
    pub ttl: Option<u64>,
}

impl<M: Message> MessageEnvelope<M> {
    /// Create a new message envelope with minimal information
    ///
    /// # Example
    /// ```rust
    /// use airssys_rt::message::{Message, MessageEnvelope};
    ///
    /// #[derive(Debug, Clone)]
    /// struct TestMsg;
    /// impl Message for TestMsg {
    ///     const MESSAGE_TYPE: &'static str = "test";
    /// }
    ///
    /// let envelope = MessageEnvelope::new(TestMsg);
    /// assert_eq!(envelope.message_type(), "test");
    /// ```
    pub fn new(payload: M) -> Self {
        let priority = payload.priority();
        Self {
            payload,
            sender: None,
            reply_to: None,
            timestamp: Utc::now(), // §3.2 chrono standard
            correlation_id: None,
            priority,
            ttl: None,
        }
    }

    /// Builder method: Set sender address
    ///
    /// # Example
    /// ```rust
    /// use airssys_rt::message::{Message, MessageEnvelope};
    /// use airssys_rt::util::ActorAddress;
    ///
    /// #[derive(Debug, Clone)]
    /// struct TestMsg;
    /// impl Message for TestMsg {
    ///     const MESSAGE_TYPE: &'static str = "test";
    /// }
    ///
    /// let sender = ActorAddress::named("sender");
    /// let envelope = MessageEnvelope::new(TestMsg)
    ///     .with_sender(sender.clone());
    ///
    /// assert_eq!(envelope.sender, Some(sender));
    /// ```
    pub fn with_sender(mut self, sender: ActorAddress) -> Self {
        self.sender = Some(sender);
        self
    }

    /// Builder method: Set reply-to address
    pub fn with_reply_to(mut self, reply_to: ActorAddress) -> Self {
        self.reply_to = Some(reply_to);
        self
    }

    /// Builder method: Set correlation ID
    pub fn with_correlation_id(mut self, id: Uuid) -> Self {
        self.correlation_id = Some(id);
        self
    }

    /// Builder method: Set time-to-live in seconds
    pub fn with_ttl(mut self, ttl_seconds: u64) -> Self {
        self.ttl = Some(ttl_seconds);
        self
    }

    /// Check if message has expired based on TTL
    ///
    /// # Example
    /// ```rust
    /// use airssys_rt::message::{Message, MessageEnvelope};
    ///
    /// #[derive(Debug, Clone)]
    /// struct TestMsg;
    /// impl Message for TestMsg {
    ///     const MESSAGE_TYPE: &'static str = "test";
    /// }
    ///
    /// let envelope = MessageEnvelope::new(TestMsg).with_ttl(60);
    /// assert!(!envelope.is_expired()); // Fresh message
    /// ```
    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            let age = Utc::now()
                .signed_duration_since(self.timestamp)
                .num_seconds() as u64;
            age > ttl
        } else {
            false
        }
    }

    /// Get message type from payload's const
    pub fn message_type(&self) -> &'static str {
        M::MESSAGE_TYPE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    struct TestMessage {
        content: String,
    }

    impl Message for TestMessage {
        const MESSAGE_TYPE: &'static str = "test_message";
    }

    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    struct HighPriorityMessage {
        data: u64,
    }

    impl Message for HighPriorityMessage {
        const MESSAGE_TYPE: &'static str = "high_priority";

        fn priority(&self) -> MessagePriority {
            MessagePriority::High
        }
    }

    #[test]
    fn test_envelope_creation() {
        let msg = TestMessage {
            content: "test".to_string(),
        };
        let envelope = MessageEnvelope::new(msg);

        assert_eq!(envelope.message_type(), "test_message");
        assert_eq!(envelope.priority, MessagePriority::Normal);
        assert!(envelope.sender.is_none());
        assert!(envelope.reply_to.is_none());
        assert!(envelope.correlation_id.is_none());
        assert!(envelope.ttl.is_none());
    }

    #[test]
    fn test_envelope_with_priority() {
        let msg = HighPriorityMessage { data: 42 };
        let envelope = MessageEnvelope::new(msg);

        assert_eq!(envelope.priority, MessagePriority::High);
    }

    #[test]
    fn test_builder_pattern_sender() {
        let msg = TestMessage {
            content: "test".to_string(),
        };
        let sender = ActorAddress::named("sender");

        let envelope = MessageEnvelope::new(msg).with_sender(sender.clone());

        assert_eq!(envelope.sender, Some(sender));
    }

    #[test]
    fn test_builder_pattern_reply_to() {
        let msg = TestMessage {
            content: "test".to_string(),
        };
        let reply_to = ActorAddress::anonymous();

        let envelope = MessageEnvelope::new(msg).with_reply_to(reply_to.clone());

        assert_eq!(envelope.reply_to, Some(reply_to));
    }

    #[test]
    fn test_builder_pattern_correlation_id() {
        let msg = TestMessage {
            content: "test".to_string(),
        };
        let correlation_id = Uuid::new_v4();

        let envelope = MessageEnvelope::new(msg).with_correlation_id(correlation_id);

        assert_eq!(envelope.correlation_id, Some(correlation_id));
    }

    #[test]
    fn test_builder_pattern_ttl() {
        let msg = TestMessage {
            content: "test".to_string(),
        };

        let envelope = MessageEnvelope::new(msg).with_ttl(60);

        assert_eq!(envelope.ttl, Some(60));
    }

    #[test]
    fn test_builder_pattern_chaining() {
        let msg = TestMessage {
            content: "test".to_string(),
        };
        let sender = ActorAddress::named("sender");
        let reply_to = ActorAddress::named("reply_to");
        let correlation_id = Uuid::new_v4();

        let envelope = MessageEnvelope::new(msg)
            .with_sender(sender.clone())
            .with_reply_to(reply_to.clone())
            .with_correlation_id(correlation_id)
            .with_ttl(120);

        assert_eq!(envelope.sender, Some(sender));
        assert_eq!(envelope.reply_to, Some(reply_to));
        assert_eq!(envelope.correlation_id, Some(correlation_id));
        assert_eq!(envelope.ttl, Some(120));
    }

    #[test]
    fn test_ttl_not_expired() {
        let msg = TestMessage {
            content: "test".to_string(),
        };
        let envelope = MessageEnvelope::new(msg).with_ttl(10);

        assert!(!envelope.is_expired());
    }

    #[test]
    fn test_ttl_no_expiration_when_none() {
        let msg = TestMessage {
            content: "test".to_string(),
        };
        let mut envelope = MessageEnvelope::new(msg);

        // Manually set old timestamp
        envelope.timestamp = Utc::now() - chrono::Duration::seconds(100);

        assert!(!envelope.is_expired()); // No TTL set, never expires
    }

    #[test]
    fn test_ttl_expired() {
        let msg = TestMessage {
            content: "test".to_string(),
        };
        let mut envelope = MessageEnvelope::new(msg).with_ttl(1);

        // Manually set timestamp to past
        envelope.timestamp = Utc::now() - chrono::Duration::seconds(5);

        assert!(envelope.is_expired());
    }

    #[test]
    fn test_message_type_accessor() {
        let msg = TestMessage {
            content: "test".to_string(),
        };
        let envelope = MessageEnvelope::new(msg);

        assert_eq!(envelope.message_type(), TestMessage::MESSAGE_TYPE);
    }
}
