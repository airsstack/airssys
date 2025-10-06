//! Actor context with metadata and state management.
//!
//! Provides zero-cost actor context with generic constraints for type safety.

// Layer 1: Standard library imports
use std::marker::PhantomData;

// Layer 2: Third-party crate imports
use chrono::{DateTime, Utc}; // §3.2 MANDATORY

// Layer 3: Internal module imports
use crate::broker::MessageBroker;
use crate::message::{Message, MessageEnvelope};
use crate::util::{ActorAddress, ActorId};

/// Actor context with metadata for zero-cost abstractions.
///
/// Generic over message type M and broker type B for compile-time type safety.
/// Broker is injected via dependency injection (ADR-006).
///
/// # Type Parameters
///
/// * `M` - Message type
/// * `B` - Broker implementation (injected by ActorSystem)
///
/// # Examples
///
/// ```rust,ignore
/// use airssys_rt::{ActorContext, Message};
/// use airssys_rt::util::ActorAddress;
/// use airssys_rt::broker::InMemoryMessageBroker;
///
/// #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// struct MyMessage;
///
/// impl Message for MyMessage {
///     const MESSAGE_TYPE: &'static str = "my_message";
/// }
///
/// let address = ActorAddress::named("my-actor");
/// let broker = InMemoryMessageBroker::new();
/// let context = ActorContext::new(address, broker);
/// ```
pub struct ActorContext<M: Message, B: MessageBroker<M>> {
    address: ActorAddress,
    id: ActorId,
    created_at: DateTime<Utc>,
    last_message_at: Option<DateTime<Utc>>,
    message_count: u64,
    broker: B, // Dependency injection (ADR-006)
    _marker: PhantomData<M>,
}

impl<M: Message, B: MessageBroker<M>> ActorContext<M, B> {
    /// Create a new actor context.
    ///
    /// # Arguments
    ///
    /// * `address` - Actor's address
    /// * `broker` - Message broker for sending messages (injected)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use airssys_rt::{ActorContext, Message};
    /// use airssys_rt::util::ActorAddress;
    /// use airssys_rt::broker::InMemoryMessageBroker;
    ///
    /// #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    /// struct TestMessage;
    ///
    /// impl Message for TestMessage {
    ///     const MESSAGE_TYPE: &'static str = "test";
    /// }
    ///
    /// let address = ActorAddress::anonymous();
    /// let broker = InMemoryMessageBroker::new();
    /// let context = ActorContext::new(address, broker);
    /// ```
    pub fn new(address: ActorAddress, broker: B) -> Self {
        Self {
            id: *address.id(),
            address,
            created_at: Utc::now(), // §3.2
            last_message_at: None,
            message_count: 0,
            broker,
            _marker: PhantomData,
        }
    }

    /// Get the actor's address.
    pub fn address(&self) -> &ActorAddress {
        &self.address
    }

    /// Get the actor's ID.
    pub fn id(&self) -> &ActorId {
        &self.id
    }

    /// Get the actor's creation timestamp.
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Get the timestamp of the last processed message.
    pub fn last_message_at(&self) -> Option<DateTime<Utc>> {
        self.last_message_at
    }

    /// Get the total number of messages processed.
    pub fn message_count(&self) -> u64 {
        self.message_count
    }

    /// Record that a message was processed.
    ///
    /// Updates last_message_at and increments message_count.
    pub fn record_message(&mut self) {
        self.last_message_at = Some(Utc::now());
        self.message_count += 1;
    }

    /// Send a message to another actor.
    ///
    /// Publishes the message to the broker, which broadcasts it to all subscribers.
    /// The ActorSystem router will route it to the target actor.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to send
    /// * `recipient` - Target actor address
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// context.send(message, target_address).await?;
    /// ```
    pub async fn send(&self, message: M, recipient: ActorAddress) -> Result<(), String>
    where
        M: serde::Serialize,
    {
        let mut envelope = MessageEnvelope::new(message);
        envelope.reply_to = Some(recipient);

        self.broker
            .publish(envelope)
            .await
            .map_err(|e| e.to_string())
    }

    /// Send a request and wait for a reply.
    ///
    /// # Arguments
    ///
    /// * `request` - The request message
    /// * `recipient` - Target actor address
    /// * `timeout` - Maximum time to wait for response
    ///
    /// # Returns
    ///
    /// The response message, or None if no response received
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let response = context
    ///     .request(request, target_address, Duration::from_secs(5))
    ///     .await?;
    /// ```
    pub async fn request(
        &self,
        request: M,
        recipient: ActorAddress,
        timeout: std::time::Duration,
    ) -> Result<Option<MessageEnvelope<M>>, String>
    where
        M: serde::Serialize + for<'de> serde::Deserialize<'de>,
    {
        let mut envelope = MessageEnvelope::new(request);
        envelope.reply_to = Some(recipient);

        self.broker
            .publish_request(envelope, timeout)
            .await
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::broker::in_memory::InMemoryMessageBroker;
    use crate::message::MessagePriority;

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    struct TestMessage;

    impl Message for TestMessage {
        const MESSAGE_TYPE: &'static str = "test";

        fn priority(&self) -> MessagePriority {
            MessagePriority::Normal
        }
    }

    fn create_test_context() -> ActorContext<TestMessage, InMemoryMessageBroker<TestMessage>> {
        let address = ActorAddress::anonymous();
        let broker = InMemoryMessageBroker::new();
        ActorContext::new(address, broker)
    }

    #[test]
    fn test_context_creation() {
        let context = create_test_context();

        assert_eq!(context.message_count(), 0);
        assert!(context.last_message_at().is_none());
    }

    #[test]
    fn test_context_address_accessor() {
        let address = ActorAddress::named("test-actor");
        let broker = InMemoryMessageBroker::<TestMessage>::new();
        let context = ActorContext::new(address.clone(), broker);

        assert_eq!(context.address(), &address);
        assert_eq!(context.address().name(), Some("test-actor"));
    }

    #[test]
    fn test_context_id_accessor() {
        let address = ActorAddress::anonymous();
        let id = *address.id();
        let broker = InMemoryMessageBroker::<TestMessage>::new();
        let context = ActorContext::new(address, broker);

        assert_eq!(context.id(), &id);
    }

    #[test]
    fn test_record_message() {
        let mut context = create_test_context();

        assert_eq!(context.message_count(), 0);
        assert!(context.last_message_at().is_none());

        context.record_message();

        assert_eq!(context.message_count(), 1);
        assert!(context.last_message_at().is_some());
    }

    #[test]
    fn test_multiple_message_records() {
        let mut context = create_test_context();

        for i in 1..=10 {
            context.record_message();
            assert_eq!(context.message_count(), i);
        }
    }

    #[test]
    fn test_created_at_timestamp() {
        let address = ActorAddress::anonymous();
        let before = Utc::now();
        let broker = InMemoryMessageBroker::<TestMessage>::new();
        let context = ActorContext::new(address, broker);
        let after = Utc::now();

        let created = context.created_at();
        assert!(created >= before && created <= after);
    }
}
