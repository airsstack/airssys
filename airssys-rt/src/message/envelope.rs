// Layer 1: Standard library imports
use std::fmt::Debug;

// Layer 2: Third-party crate imports
use chrono::{DateTime, Utc}; // §3.2 MANDATORY
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Layer 3: Internal module imports
use super::traits::{Message, MessagePriority};
use crate::util::ids::ActorAddress;

/// Message envelope with routing metadata and zero-cost abstraction.
///
/// Wraps user messages with metadata needed for routing, tracking, and delivery.
/// The envelope provides request/reply capabilities, expiration tracking, and
/// priority-based routing without sacrificing type safety or performance.
///
/// # Purpose
///
/// The envelope serves multiple purposes in the messaging system:
/// - **Routing**: Sender and reply-to addresses for message routing
/// - **Tracking**: Correlation IDs for request/response correlation
/// - **Expiration**: Time-to-live (TTL) for message expiration
/// - **Priority**: Priority-based message processing
/// - **Timestamps**: Message creation time for ordering and diagnostics
///
/// # Type Safety
///
/// The envelope is generic over the message type M, ensuring compile-time
/// type safety without runtime dispatch or type erasure. The message type
/// is preserved throughout the entire messaging pipeline.
///
/// # Zero-Cost Abstraction
///
/// **Stack Allocation**: `MessageEnvelope<M>` is stack-allocated when M is
/// stack-allocated, avoiding heap overhead for message passing.
///
/// **No Virtual Dispatch**: Generic constraints ensure compile-time resolution,
/// no dynamic dispatch overhead.
///
/// # Envelope Metadata
///
/// The envelope includes the following metadata fields:
///
/// - **payload**: The actual message (generic type M)
/// - **sender**: Optional sender address for replies
/// - **reply_to**: Optional recipient for reply messages
/// - **timestamp**: Message creation time (UTC, §3.2)
/// - **correlation_id**: Optional UUID for request/response tracking
/// - **priority**: Message priority (from payload.priority())
/// - **ttl**: Optional time-to-live in seconds
///
/// # Builder Pattern
///
/// The envelope uses a fluent builder API for ergonomic construction:
///
/// ```rust
/// use airssys_rt::message::{Message, MessageEnvelope};
/// use airssys_rt::util::ActorAddress;
/// use uuid::Uuid;
///
/// #[derive(Debug, Clone)]
/// struct MyMessage { data: String }
///
/// impl Message for MyMessage {
///     const MESSAGE_TYPE: &'static str = "my_message";
/// }
///
/// let msg = MyMessage { data: "hello".to_string() };
/// let sender = ActorAddress::named("sender");
/// let reply_to = ActorAddress::named("recipient");
///
/// let envelope = MessageEnvelope::new(msg)
///     .with_sender(sender)
///     .with_reply_to(reply_to)
///     .with_correlation_id(Uuid::new_v4())
///     .with_ttl(60); // 60 seconds TTL
///
/// assert_eq!(envelope.message_type(), "my_message");
/// assert!(!envelope.is_expired());
/// ```
///
/// # Request/Reply Pattern
///
/// The envelope supports request/reply correlation via sender and correlation_id:
///
/// ```rust,ignore
/// // Requester side
/// let request = RequestMessage { query: "status" };
/// let correlation_id = Uuid::new_v4();
///
/// let envelope = MessageEnvelope::new(request)
///     .with_sender(my_address.clone())
///     .with_correlation_id(correlation_id);
///
/// broker.publish(envelope).await?;
///
/// // Responder side
/// async fn handle_request(envelope: MessageEnvelope<RequestMessage>) {
///     let response = ResponseMessage { status: "ok" };
///     
///     let reply = MessageEnvelope::new(response)
///         .with_sender(my_address.clone())
///         .with_reply_to(envelope.sender.unwrap())
///         .with_correlation_id(envelope.correlation_id.unwrap());
///     
///     broker.publish(reply).await?;
/// }
/// ```
///
/// # Message Expiration
///
/// Messages can have a time-to-live (TTL) for automatic expiration:
///
/// ```rust
/// use airssys_rt::message::{Message, MessageEnvelope};
///
/// #[derive(Debug, Clone)]
/// struct CachedData { value: u64 }
///
/// impl Message for CachedData {
///     const MESSAGE_TYPE: &'static str = "cached_data";
/// }
///
/// let envelope = MessageEnvelope::new(CachedData { value: 42 })
///     .with_ttl(300); // Expires in 5 minutes
///
/// // Check expiration before processing
/// if envelope.is_expired() {
///     // Discard expired message
///     return;
/// }
/// ```
///
/// # Performance Characteristics
///
/// Based on RT-TASK-008 baseline measurements (Oct 16, 2025):
///
/// - **Envelope creation**: ~737ns (included in message creation)
/// - **Expiration check**: <10ns (simple timestamp comparison)
/// - **Serialization overhead**: Minimal (serde derive macros)
///
/// Source: `BENCHMARKING.md` §6.2
///
/// # See Also
///
/// - [`Message`](super::Message) - Message trait that envelopes wrap
/// - [`MessageBroker`](crate::broker::MessageBroker) - Broker that routes envelopes
/// - [`ActorContext::send()`](crate::actor::ActorContext::send) - Sending enveloped messages
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
    /// Create a new message envelope with minimal metadata.
    ///
    /// Creates an envelope with:
    /// - The provided message payload
    /// - Current timestamp (UTC, §3.2)
    /// - Priority extracted from payload
    /// - No sender, reply_to, correlation_id, or TTL (all None)
    ///
    /// Use builder methods to add optional metadata.
    ///
    /// # Performance
    ///
    /// Envelope creation is ~737ns including message construction
    /// (from BENCHMARKING.md §6.2).
    ///
    /// # Examples
    ///
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
    /// assert!(envelope.sender.is_none());
    /// assert!(envelope.reply_to.is_none());
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

    /// Builder method: Set sender address for reply capability.
    ///
    /// The sender address allows recipients to send replies back to the
    /// original sender. This is essential for request/reply patterns.
    ///
    /// # Examples
    ///
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

    /// Builder method: Set reply-to address for response routing.
    ///
    /// The reply-to address specifies where responses should be sent,
    /// which may differ from the original sender (e.g., for delegation).
    pub fn with_reply_to(mut self, reply_to: ActorAddress) -> Self {
        self.reply_to = Some(reply_to);
        self
    }

    /// Builder method: Set correlation ID for request/response tracking.
    ///
    /// Correlation IDs link requests with their responses, enabling:
    /// - Request/response matching
    /// - Distributed tracing
    /// - Debugging message flows
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::message::{Message, MessageEnvelope};
    /// use uuid::Uuid;
    ///
    /// #[derive(Debug, Clone)]
    /// struct TestMsg;
    /// impl Message for TestMsg {
    ///     const MESSAGE_TYPE: &'static str = "test";
    /// }
    ///
    /// let id = Uuid::new_v4();
    /// let envelope = MessageEnvelope::new(TestMsg)
    ///     .with_correlation_id(id);
    ///
    /// assert_eq!(envelope.correlation_id, Some(id));
    /// ```
    pub fn with_correlation_id(mut self, id: Uuid) -> Self {
        self.correlation_id = Some(id);
        self
    }

    /// Builder method: Set time-to-live in seconds.
    ///
    /// Messages with TTL expire after the specified duration and can be
    /// filtered out using `is_expired()`. Useful for:
    /// - Cache invalidation messages
    /// - Time-sensitive notifications
    /// - Preventing stale message processing
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::message::{Message, MessageEnvelope};
    ///
    /// #[derive(Debug, Clone)]
    /// struct TestMsg;
    /// impl Message for TestMsg {
    ///     const MESSAGE_TYPE: &'static str = "test";
    /// }
    ///
    /// let envelope = MessageEnvelope::new(TestMsg)
    ///     .with_ttl(60); // 60 seconds
    ///
    /// assert_eq!(envelope.ttl, Some(60));
    /// assert!(!envelope.is_expired()); // Fresh message
    /// ```
    pub fn with_ttl(mut self, ttl_seconds: u64) -> Self {
        self.ttl = Some(ttl_seconds);
        self
    }

    /// Check if message has expired based on TTL.
    ///
    /// Compares the message's age (since timestamp) against its TTL.
    /// Returns `false` if no TTL is set (messages don't expire by default).
    ///
    /// # Performance
    ///
    /// Expiration check is <10ns (simple timestamp arithmetic).
    ///
    /// # Examples
    ///
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
    ///
    /// // No TTL = never expires
    /// let no_ttl = MessageEnvelope::new(TestMsg);
    /// assert!(!no_ttl.is_expired());
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

    /// Get message type from payload's const MESSAGE_TYPE.
    ///
    /// Returns the compile-time message type identifier without runtime
    /// reflection or type checking overhead.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::message::{Message, MessageEnvelope};
    ///
    /// #[derive(Debug, Clone)]
    /// struct TestMsg;
    /// impl Message for TestMsg {
    ///     const MESSAGE_TYPE: &'static str = "test_message";
    /// }
    ///
    /// let envelope = MessageEnvelope::new(TestMsg);
    /// assert_eq!(envelope.message_type(), "test_message");
    /// ```
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
