//! Inter-component messaging abstractions for WASM components.
//!
//! These types define the message envelope, delivery guarantees, and message type
//! classifications needed for Block 5 (Inter-Component Communication). They provide
//! the foundation for actor-based message passing via airssys-rt MessageBroker
//! integration, following YAGNI principles (§6.1).
//!
//! # Design Rationale
//!
//! - **MessageEnvelope**: Uses ComponentId for routing, multicodec for serialization
//!   format (ADR-WASM-001), `chrono::DateTime<Utc>` per §3.2 for timestamps.
//!   Contains all metadata needed for three messaging patterns.
//! - **MessageType**: Covers fire-and-forget, request-response, and pub-sub patterns.
//! - **DeliveryGuarantee**: Three semantics levels (at-most-once, at-least-once,
//!   exactly-once future). ExactlyOnce marked for future implementation.
//!
//! Routing is handled by airssys-rt MessageBroker per ADR-WASM-009.
//! All types are Clone + Serialize/Deserialize for message passing and persistence.
//! No internal dependencies beyond core (zero circular deps).
//!
//! # Architecture
//!
//! Messages flow through airssys-rt MessageBroker with push-based delivery:
//! 1. Sender calls send_message() or send_request() host function
//! 2. Host validates capabilities and creates MessageEnvelope
//! 3. MessageBroker routes message (~211ns routing time)
//! 4. Receiver's handle_message() export is invoked (push delivery)
//!
//! Performance target: <300ns overhead per message (ADR-WASM-009)
//!
//! # References
//!
//! - ADR-WASM-009: Component Communication Model (messaging architecture)
//! - ADR-WASM-001: Multicodec Compatibility Strategy (serialization)
//! - KNOWLEDGE-WASM-005: Messaging Architecture (implementation patterns)
//! - airssys-rt: MessageBroker and routing performance (RT-TASK-008)

// Layer 2: External crates
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// Layer 3: Internal (core only)
use crate::core::component::ComponentId;

/// Message envelope for actor-based messaging between WASM components.
///
/// This struct wraps messages sent via the airssys-rt MessageBroker, providing
/// routing information (from/to ComponentIds), message type classification,
/// optional topic-based routing for pub-sub, correlation IDs for request-response,
/// and multicodec-encoded payload. Used in Block 5 for inter-component communication.
///
/// # Message Flow
///
/// 1. Sender component invokes `send_message()` host function with payload
/// 2. Host runtime validates MessageSend capability and creates MessageEnvelope
/// 3. MessageBroker routes envelope to recipient (~211ns routing time)
/// 4. Receiver's `handle_message()` export is invoked with envelope (push delivery)
///
/// # Serialization Format
///
/// Payload uses multicodec self-describing format (ADR-WASM-001):
/// - `codec` field identifies format (0x55 = raw binary, 0x0200 = JSON, 0x51 = CBOR)
/// - Enables format negotiation between components
/// - Future-proof for new serialization formats
///
/// # Example
///
/// ```
/// use airssys_wasm::core::messaging::{MessageEnvelope, MessageType};
/// use airssys_wasm::core::component::ComponentId;
/// use chrono::Utc;
///
/// let envelope = MessageEnvelope::new(
///     MessageType::FireAndForget,
///     ComponentId::new("sender"),
///     ComponentId::new("receiver"),
///     vec![1, 2, 3, 4],
///     0x55, // Raw binary codec
///     "msg-001".to_string(),
/// );
///
/// assert_eq!(envelope.from.as_str(), "sender");
/// assert_eq!(envelope.to.as_str(), "receiver");
/// assert_eq!(envelope.codec, 0x55);
/// ```
///
/// # References
///
/// - KNOWLEDGE-WASM-005 §4: Message Flow Architecture
/// - ADR-WASM-009: Component Communication Model
/// - ADR-WASM-001: Multicodec Compatibility Strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEnvelope {
    /// Message pattern type (fire-and-forget, request, response, publish).
    pub message_type: MessageType,

    /// Source component ID.
    pub from: ComponentId,

    /// Destination component ID.
    pub to: ComponentId,

    /// Optional topic for pub-sub routing (e.g., "events.user.created").
    pub topic: Option<String>,

    /// Message payload (multicodec-encoded bytes).
    pub payload: Vec<u8>,

    /// Multicodec identifier for payload format (0x55=raw, 0x0200=JSON, 0x51=CBOR).
    pub codec: u64,

    /// Unique message identifier (for deduplication and tracking).
    pub message_id: String,

    /// Optional correlation ID for request-response pattern (links request to response).
    pub correlation_id: Option<String>,

    /// UTC timestamp when message was created (§3.2 compliance).
    pub timestamp: DateTime<Utc>,
}

impl MessageEnvelope {
    /// Create a new message envelope with required fields.
    ///
    /// Sets timestamp to current UTC time. Optional fields (topic, correlation_id)
    /// default to None. Use builder methods to set them.
    ///
    /// # Parameters
    ///
    /// - `message_type`: Classification (fire-and-forget, request, response, publish)
    /// - `from`: Source component ID
    /// - `to`: Destination component ID
    /// - `payload`: Multicodec-encoded message data
    /// - `codec`: Multicodec identifier for payload format
    /// - `message_id`: Unique identifier for deduplication
    ///
    /// # Example
    ///
    /// ```
    /// use airssys_wasm::core::messaging::{MessageEnvelope, MessageType};
    /// use airssys_wasm::core::component::ComponentId;
    ///
    /// let msg = MessageEnvelope::new(
    ///     MessageType::FireAndForget,
    ///     ComponentId::new("alice"),
    ///     ComponentId::new("bob"),
    ///     b"hello".to_vec(),
    ///     0x55,
    ///     "msg-123".to_string(),
    /// );
    ///
    /// assert!(msg.topic.is_none());
    /// assert!(msg.correlation_id.is_none());
    /// ```
    pub fn new(
        message_type: MessageType,
        from: ComponentId,
        to: ComponentId,
        payload: Vec<u8>,
        codec: u64,
        message_id: String,
    ) -> Self {
        Self {
            message_type,
            from,
            to,
            topic: None,
            payload,
            codec,
            message_id,
            correlation_id: None,
            timestamp: Utc::now(),
        }
    }

    /// Set topic for pub-sub routing (builder pattern).
    ///
    /// Topics follow hierarchical naming (e.g., "events.user.created").
    /// Used with MessageType::Publish for topic-based delivery.
    ///
    /// # Example
    ///
    /// ```
    /// use airssys_wasm::core::messaging::{MessageEnvelope, MessageType};
    /// use airssys_wasm::core::component::ComponentId;
    ///
    /// let msg = MessageEnvelope::new(
    ///     MessageType::Publish,
    ///     ComponentId::new("publisher"),
    ///     ComponentId::new("broker"),
    ///     vec![],
    ///     0x0200,
    ///     "msg-001".to_string(),
    /// ).with_topic("events.system.startup");
    ///
    /// assert_eq!(msg.topic, Some("events.system.startup".to_string()));
    /// ```
    pub fn with_topic(mut self, topic: impl Into<String>) -> Self {
        self.topic = Some(topic.into());
        self
    }

    /// Set correlation ID for request-response pattern (builder pattern).
    ///
    /// Links a response message back to its originating request. Typically
    /// set to the original request's message_id.
    ///
    /// # Example
    ///
    /// ```
    /// use airssys_wasm::core::messaging::{MessageEnvelope, MessageType};
    /// use airssys_wasm::core::component::ComponentId;
    ///
    /// let request = MessageEnvelope::new(
    ///     MessageType::Request,
    ///     ComponentId::new("client"),
    ///     ComponentId::new("server"),
    ///     b"get_data".to_vec(),
    ///     0x55,
    ///     "req-001".to_string(),
    /// ).with_correlation_id("session-123");
    ///
    /// assert_eq!(request.correlation_id, Some("session-123".to_string()));
    /// ```
    pub fn with_correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = Some(correlation_id.into());
        self
    }

    /// Check if this is a fire-and-forget message (no response expected).
    ///
    /// # Example
    ///
    /// ```
    /// use airssys_wasm::core::messaging::{MessageEnvelope, MessageType};
    /// use airssys_wasm::core::component::ComponentId;
    ///
    /// let msg = MessageEnvelope::new(
    ///     MessageType::FireAndForget,
    ///     ComponentId::new("a"),
    ///     ComponentId::new("b"),
    ///     vec![],
    ///     0x55,
    ///     "msg-1".to_string(),
    /// );
    ///
    /// assert!(msg.is_fire_and_forget());
    /// assert!(!msg.is_request());
    /// ```
    pub fn is_fire_and_forget(&self) -> bool {
        matches!(self.message_type, MessageType::FireAndForget)
    }

    /// Check if this is a request message (expects response).
    ///
    /// # Example
    ///
    /// ```
    /// use airssys_wasm::core::messaging::{MessageEnvelope, MessageType};
    /// use airssys_wasm::core::component::ComponentId;
    ///
    /// let msg = MessageEnvelope::new(
    ///     MessageType::Request,
    ///     ComponentId::new("a"),
    ///     ComponentId::new("b"),
    ///     vec![],
    ///     0x55,
    ///     "req-1".to_string(),
    /// );
    ///
    /// assert!(msg.is_request());
    /// assert!(!msg.is_response());
    /// ```
    pub fn is_request(&self) -> bool {
        matches!(self.message_type, MessageType::Request)
    }

    /// Check if this is a response message.
    ///
    /// # Example
    ///
    /// ```
    /// use airssys_wasm::core::messaging::{MessageEnvelope, MessageType};
    /// use airssys_wasm::core::component::ComponentId;
    ///
    /// let msg = MessageEnvelope::new(
    ///     MessageType::Response,
    ///     ComponentId::new("server"),
    ///     ComponentId::new("client"),
    ///     vec![1, 2, 3],
    ///     0x0200,
    ///     "resp-1".to_string(),
    /// );
    ///
    /// assert!(msg.is_response());
    /// assert!(!msg.is_publish());
    /// ```
    pub fn is_response(&self) -> bool {
        matches!(self.message_type, MessageType::Response)
    }

    /// Check if this is a publish message (pub-sub pattern).
    ///
    /// # Example
    ///
    /// ```
    /// use airssys_wasm::core::messaging::{MessageEnvelope, MessageType};
    /// use airssys_wasm::core::component::ComponentId;
    ///
    /// let msg = MessageEnvelope::new(
    ///     MessageType::Publish,
    ///     ComponentId::new("pub"),
    ///     ComponentId::new("broker"),
    ///     vec![],
    ///     0x55,
    ///     "pub-1".to_string(),
    /// );
    ///
    /// assert!(msg.is_publish());
    /// ```
    pub fn is_publish(&self) -> bool {
        matches!(self.message_type, MessageType::Publish)
    }

    /// Create a reply message to this request.
    ///
    /// Swaps `from` and `to` fields, sets `correlation_id` to original `message_id`,
    /// and marks as Response type. Used for request-response pattern.
    ///
    /// # Parameters
    ///
    /// - `payload`: Response data (multicodec-encoded)
    /// - `codec`: Multicodec identifier for response format
    /// - `message_id`: Unique ID for the response message
    ///
    /// # Example
    ///
    /// ```
    /// use airssys_wasm::core::messaging::{MessageEnvelope, MessageType};
    /// use airssys_wasm::core::component::ComponentId;
    ///
    /// let request = MessageEnvelope::new(
    ///     MessageType::Request,
    ///     ComponentId::new("client"),
    ///     ComponentId::new("server"),
    ///     b"get_data".to_vec(),
    ///     0x55,
    ///     "req-001".to_string(),
    /// );
    ///
    /// let response = request.reply_to(b"data".to_vec(), 0x55, "resp-001".to_string());
    ///
    /// assert_eq!(response.from.as_str(), "server");
    /// assert_eq!(response.to.as_str(), "client");
    /// assert_eq!(response.correlation_id, Some("req-001".to_string()));
    /// assert!(response.is_response());
    /// ```
    pub fn reply_to(&self, payload: Vec<u8>, codec: u64, message_id: String) -> Self {
        Self {
            message_type: MessageType::Response,
            from: self.to.clone(),
            to: self.from.clone(),
            topic: None,
            payload,
            codec,
            message_id,
            correlation_id: Some(self.message_id.clone()),
            timestamp: Utc::now(),
        }
    }
}

/// Message pattern type classification.
///
/// Defines the four messaging patterns supported by the component communication
/// model. Each pattern has different semantics for delivery and response expectations.
///
/// # Patterns
///
/// - **FireAndForget**: One-way message, no response expected (~280ns end-to-end)
/// - **Request**: Expects a Response message back (~560ns round-trip)
/// - **Response**: Reply to a Request (linked via correlation_id)
/// - **Publish**: Topic-based broadcast to multiple subscribers
///
/// # Performance Characteristics
///
/// Based on RT-TASK-008 benchmarks with airssys-rt MessageBroker:
/// - Fire-and-forget: ~280ns end-to-end (no waiting)
/// - Request-response: ~560ns round-trip (includes response wait)
/// - MessageBroker routing: ~211ns per message
/// - Target overhead: <300ns per message (ADR-WASM-009)
///
/// # Example
///
/// ```
/// use airssys_wasm::core::messaging::MessageType;
///
/// let msg_type = MessageType::Request;
/// assert!(msg_type.expects_response());
/// assert!(!msg_type.is_one_way());
/// assert_eq!(msg_type.description(), "Request message expecting response");
/// ```
///
/// # References
///
/// - KNOWLEDGE-WASM-005 §4: Messaging Patterns
/// - ADR-WASM-009: Component Communication Model
/// - RT-TASK-008: MessageBroker Performance Benchmarks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageType {
    /// One-way message with no response expected.
    FireAndForget,

    /// Request message expecting a Response.
    Request,

    /// Response to a Request (linked via correlation_id).
    Response,

    /// Topic-based message for pub-sub pattern.
    Publish,
}

impl MessageType {
    /// Check if this message type expects a response.
    ///
    /// Only Request messages expect responses.
    ///
    /// # Example
    ///
    /// ```
    /// use airssys_wasm::core::messaging::MessageType;
    ///
    /// assert!(MessageType::Request.expects_response());
    /// assert!(!MessageType::FireAndForget.expects_response());
    /// assert!(!MessageType::Response.expects_response());
    /// assert!(!MessageType::Publish.expects_response());
    /// ```
    pub fn expects_response(&self) -> bool {
        matches!(self, Self::Request)
    }

    /// Check if this is a one-way message (no response handling needed).
    ///
    /// Both FireAndForget and Publish are one-way.
    ///
    /// # Example
    ///
    /// ```
    /// use airssys_wasm::core::messaging::MessageType;
    ///
    /// assert!(MessageType::FireAndForget.is_one_way());
    /// assert!(MessageType::Publish.is_one_way());
    /// assert!(!MessageType::Request.is_one_way());
    /// assert!(!MessageType::Response.is_one_way());
    /// ```
    pub fn is_one_way(&self) -> bool {
        matches!(self, Self::FireAndForget | Self::Publish)
    }

    /// Get human-readable description of this message type.
    ///
    /// # Example
    ///
    /// ```
    /// use airssys_wasm::core::messaging::MessageType;
    ///
    /// assert_eq!(
    ///     MessageType::FireAndForget.description(),
    ///     "Fire-and-forget message (no response)"
    /// );
    /// ```
    pub fn description(&self) -> &'static str {
        match self {
            Self::FireAndForget => "Fire-and-forget message (no response)",
            Self::Request => "Request message expecting response",
            Self::Response => "Response to a request",
            Self::Publish => "Pub-sub topic broadcast",
        }
    }
}

/// Delivery guarantee semantics for message passing.
///
/// Defines three levels of delivery guarantees with different trade-offs between
/// reliability, latency, and implementation complexity. Components declare their
/// required guarantee level via capability metadata.
///
/// # Guarantee Levels
///
/// - **AtMostOnce**: Message delivered 0 or 1 times (may lose, fast, ~280ns)
/// - **AtLeastOnce**: Message delivered 1+ times (may duplicate, slower, ~560ns)
/// - **ExactlyOnce**: Message delivered exactly once (no loss/dup, future phase)
///
/// # Implementation Status (YAGNI §6.1)
///
/// - Phase 1 (Current): AtMostOnce, AtLeastOnce implemented
/// - Phase 2+ (Future): ExactlyOnce requires distributed coordination (Raft/Paxos)
///
/// ExactlyOnce is feature-gated until distributed consensus integration is ready.
/// Most applications work with AtLeastOnce + idempotent message handlers.
///
/// # Example
///
/// ```
/// use airssys_wasm::core::messaging::DeliveryGuarantee;
///
/// let guarantee = DeliveryGuarantee::AtLeastOnce;
/// assert!(!guarantee.may_lose_messages());
/// assert!(guarantee.may_duplicate());
/// assert!(!guarantee.is_exactly_once());
/// assert_eq!(guarantee.expected_latency_ns(), 560);
/// ```
///
/// # References
///
/// - KNOWLEDGE-WASM-005 §8: Delivery Guarantees
/// - ADR-WASM-009: Component Communication Model (Phase 1 scope)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeliveryGuarantee {
    /// At-most-once delivery: message may be lost (fast, ~280ns).
    AtMostOnce,

    /// At-least-once delivery: message may be duplicated (reliable, ~560ns).
    AtLeastOnce,
}

impl DeliveryGuarantee {
    /// Check if messages may be lost with this guarantee.
    ///
    /// Only AtMostOnce allows message loss.
    ///
    /// # Example
    ///
    /// ```
    /// use airssys_wasm::core::messaging::DeliveryGuarantee;
    ///
    /// assert!(DeliveryGuarantee::AtMostOnce.may_lose_messages());
    /// assert!(!DeliveryGuarantee::AtLeastOnce.may_lose_messages());
    /// ```
    pub fn may_lose_messages(&self) -> bool {
        matches!(self, Self::AtMostOnce)
    }

    /// Check if messages may be duplicated with this guarantee.
    ///
    /// AtLeastOnce allows duplicates (requires idempotent handlers).
    ///
    /// # Example
    ///
    /// ```
    /// use airssys_wasm::core::messaging::DeliveryGuarantee;
    ///
    /// assert!(!DeliveryGuarantee::AtMostOnce.may_duplicate());
    /// assert!(DeliveryGuarantee::AtLeastOnce.may_duplicate());
    /// ```
    pub fn may_duplicate(&self) -> bool {
        matches!(self, Self::AtLeastOnce)
    }

    /// Check if this is exactly-once delivery (future phase).
    ///
    /// Currently always returns false. ExactlyOnce delivery is deferred
    /// per YAGNI principles until distributed consensus requirements are validated.
    ///
    /// # Example
    ///
    /// ```
    /// use airssys_wasm::core::messaging::DeliveryGuarantee;
    ///
    /// assert!(!DeliveryGuarantee::AtMostOnce.is_exactly_once());
    /// assert!(!DeliveryGuarantee::AtLeastOnce.is_exactly_once());
    /// ```
    pub fn is_exactly_once(&self) -> bool {
        false
    }

    /// Get expected latency in nanoseconds for this guarantee.
    ///
    /// Based on RT-TASK-008 benchmarks:
    /// - AtMostOnce: 280ns (fire-and-forget)
    /// - AtLeastOnce: 560ns (with acknowledgment)
    ///
    /// # Example
    ///
    /// ```
    /// use airssys_wasm::core::messaging::DeliveryGuarantee;
    ///
    /// assert_eq!(DeliveryGuarantee::AtMostOnce.expected_latency_ns(), 280);
    /// assert_eq!(DeliveryGuarantee::AtLeastOnce.expected_latency_ns(), 560);
    /// ```
    pub fn expected_latency_ns(&self) -> u64 {
        match self {
            Self::AtMostOnce => 280,
            Self::AtLeastOnce => 560,
        }
    }

    /// Get human-readable description of this guarantee.
    ///
    /// # Example
    ///
    /// ```
    /// use airssys_wasm::core::messaging::DeliveryGuarantee;
    ///
    /// assert_eq!(
    ///     DeliveryGuarantee::AtMostOnce.description(),
    ///     "At-most-once: may lose messages (fast, ~280ns)"
    /// );
    /// ```
    pub fn description(&self) -> &'static str {
        match self {
            Self::AtMostOnce => "At-most-once: may lose messages (fast, ~280ns)",
            Self::AtLeastOnce => "At-least-once: may duplicate messages (reliable, ~560ns)",
        }
    }
}

#[cfg(test)]
#[allow(clippy::panic)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_message_envelope_creation() {
        let envelope = MessageEnvelope::new(
            MessageType::FireAndForget,
            ComponentId::new("sender"),
            ComponentId::new("receiver"),
            vec![1, 2, 3],
            0x55,
            "msg-001".to_string(),
        );

        assert_eq!(envelope.message_type, MessageType::FireAndForget);
        assert_eq!(envelope.from.as_str(), "sender");
        assert_eq!(envelope.to.as_str(), "receiver");
        assert_eq!(envelope.payload, vec![1, 2, 3]);
        assert_eq!(envelope.codec, 0x55);
        assert_eq!(envelope.message_id, "msg-001");
        assert!(envelope.topic.is_none());
        assert!(envelope.correlation_id.is_none());
        assert!(envelope.timestamp <= Utc::now());
    }

    #[test]
    fn test_message_envelope_with_topic() {
        let envelope = MessageEnvelope::new(
            MessageType::Publish,
            ComponentId::new("pub"),
            ComponentId::new("broker"),
            vec![],
            0x0200,
            "msg-002".to_string(),
        )
        .with_topic("events.user.created");

        assert_eq!(envelope.topic, Some("events.user.created".to_string()));
    }

    #[test]
    fn test_message_envelope_with_correlation_id() {
        let envelope = MessageEnvelope::new(
            MessageType::Request,
            ComponentId::new("client"),
            ComponentId::new("server"),
            vec![],
            0x55,
            "req-001".to_string(),
        )
        .with_correlation_id("session-123");

        assert_eq!(envelope.correlation_id, Some("session-123".to_string()));
    }

    #[test]
    fn test_message_type_classification() {
        let fire_and_forget = MessageEnvelope::new(
            MessageType::FireAndForget,
            ComponentId::new("a"),
            ComponentId::new("b"),
            vec![],
            0x55,
            "msg-1".to_string(),
        );
        assert!(fire_and_forget.is_fire_and_forget());
        assert!(!fire_and_forget.is_request());

        let request = MessageEnvelope::new(
            MessageType::Request,
            ComponentId::new("c"),
            ComponentId::new("d"),
            vec![],
            0x55,
            "req-1".to_string(),
        );
        assert!(request.is_request());
        assert!(!request.is_response());

        let response = MessageEnvelope::new(
            MessageType::Response,
            ComponentId::new("e"),
            ComponentId::new("f"),
            vec![],
            0x55,
            "resp-1".to_string(),
        );
        assert!(response.is_response());
        assert!(!response.is_publish());

        let publish = MessageEnvelope::new(
            MessageType::Publish,
            ComponentId::new("g"),
            ComponentId::new("h"),
            vec![],
            0x55,
            "pub-1".to_string(),
        );
        assert!(publish.is_publish());
        assert!(!publish.is_fire_and_forget());
    }

    #[test]
    fn test_reply_to() {
        let request = MessageEnvelope::new(
            MessageType::Request,
            ComponentId::new("client"),
            ComponentId::new("server"),
            b"get_data".to_vec(),
            0x55,
            "req-001".to_string(),
        );

        let response = request.reply_to(b"data_response".to_vec(), 0x0200, "resp-001".to_string());

        assert_eq!(response.from.as_str(), "server");
        assert_eq!(response.to.as_str(), "client");
        assert_eq!(response.correlation_id, Some("req-001".to_string()));
        assert!(response.is_response());
        assert_eq!(response.payload, b"data_response".to_vec());
        assert_eq!(response.codec, 0x0200);
        assert_eq!(response.message_id, "resp-001");
    }

    #[test]
    fn test_message_envelope_serialization() {
        let envelope = MessageEnvelope::new(
            MessageType::Request,
            ComponentId::new("sender"),
            ComponentId::new("receiver"),
            vec![1, 2, 3],
            0x55,
            "msg-001".to_string(),
        );

        let json = serde_json::to_value(&envelope)
            .unwrap_or_else(|e| panic!("serialization should succeed: {e}"));
        assert_eq!(json["from"], "sender");
        assert_eq!(json["to"], "receiver");
        assert_eq!(json["message_id"], "msg-001");

        let deserialized: MessageEnvelope = serde_json::from_value(json)
            .unwrap_or_else(|e| panic!("deserialization should succeed: {e}"));
        assert_eq!(deserialized.from.as_str(), "sender");
        assert_eq!(deserialized.message_id, "msg-001");
    }

    #[test]
    fn test_message_type_expects_response() {
        assert!(MessageType::Request.expects_response());
        assert!(!MessageType::FireAndForget.expects_response());
        assert!(!MessageType::Response.expects_response());
        assert!(!MessageType::Publish.expects_response());
    }

    #[test]
    fn test_message_type_is_one_way() {
        assert!(MessageType::FireAndForget.is_one_way());
        assert!(MessageType::Publish.is_one_way());
        assert!(!MessageType::Request.is_one_way());
        assert!(!MessageType::Response.is_one_way());
    }

    #[test]
    fn test_message_type_description() {
        assert_eq!(
            MessageType::FireAndForget.description(),
            "Fire-and-forget message (no response)"
        );
        assert_eq!(
            MessageType::Request.description(),
            "Request message expecting response"
        );
    }

    #[test]
    fn test_delivery_guarantee_may_lose() {
        assert!(DeliveryGuarantee::AtMostOnce.may_lose_messages());
        assert!(!DeliveryGuarantee::AtLeastOnce.may_lose_messages());
    }

    #[test]
    fn test_delivery_guarantee_may_duplicate() {
        assert!(!DeliveryGuarantee::AtMostOnce.may_duplicate());
        assert!(DeliveryGuarantee::AtLeastOnce.may_duplicate());
    }

    #[test]
    fn test_delivery_guarantee_expected_latency() {
        assert_eq!(DeliveryGuarantee::AtMostOnce.expected_latency_ns(), 280);
        assert_eq!(DeliveryGuarantee::AtLeastOnce.expected_latency_ns(), 560);
    }
}
