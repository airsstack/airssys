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

/// Actor context with metadata and state management.
///
/// Provides zero-cost actor context with generic constraints for compile-time type safety.
/// Each actor receives a context that provides access to metadata, messaging capabilities,
/// and runtime statistics.
///
/// # Purpose
///
/// The context serves as the actor's interface to the actor system, providing:
/// - **Identity**: Unique actor ID and address (name or anonymous)
/// - **Metadata**: Creation time, message count, last message timestamp
/// - **Messaging**: Send messages to other actors via the message broker
/// - **Statistics**: Runtime metrics for monitoring and debugging
///
/// # Type Parameters
///
/// * `M` - Message type this actor can handle (from `Actor::Message`)
/// * `B` - Message broker implementation (injected by ActorSystem via ADR-006)
///
/// # Generic Constraints
///
/// Uses generic constraints instead of trait objects (§6.2) for zero-cost abstractions:
/// - No dynamic dispatch overhead
/// - Compile-time type checking
/// - Inlined method calls
/// - No heap allocations for context operations
///
/// # Context Metadata
///
/// Actors can access the following metadata through their context:
///
/// - **address()**: Actor's full address (ID + optional name)
/// - **id()**: Unique actor identifier (UUID)
/// - **created_at()**: Actor spawn timestamp (UTC)
/// - **message_count()**: Total messages processed
/// - **last_message_at()**: Timestamp of last processed message
///
/// # Messaging Through Context
///
/// The context provides two messaging patterns:
///
/// ## Fire-and-Forget
/// Send a message without waiting for response:
/// ```rust,ignore
/// context.send(message, recipient_address).await?;
/// ```
///
/// ## Request/Reply
/// Send a message and wait for response with timeout:
/// ```rust,ignore
/// let response = context
///     .request(message, recipient_address, Duration::from_secs(5))
///     .await?;
/// ```
///
/// # Examples
///
/// Access actor metadata within message handler:
///
/// ```rust,ignore
/// use airssys_rt::prelude::*;
/// use async_trait::async_trait;
///
/// struct StatefulActor {
///     state: u64,
/// }
///
/// #[async_trait]
/// impl Actor for StatefulActor {
///     type Message = MyMessage;
///     type Error = std::io::Error;
///     
///     async fn handle_message<B: MessageBroker<Self::Message>>(
///         &mut self,
///         msg: Self::Message,
///         ctx: &mut ActorContext<Self::Message, B>,
///     ) -> Result<(), Self::Error> {
///         // Access context metadata
///         println!("Actor {} processing message #{}",
///             ctx.address(), ctx.message_count());
///         
///         // Update internal state
///         self.state += 1;
///         
///         // Record message processing
///         ctx.record_message();
///         
///         Ok(())
///     }
/// }
/// ```
///
/// Send message to another actor:
///
/// ```rust,ignore
/// async fn handle_message<B: MessageBroker<Self::Message>>(
///     &mut self,
///     msg: Self::Message,
///     ctx: &mut ActorContext<Self::Message, B>,
/// ) -> Result<(), Self::Error> {
///     // Forward message to another actor
///     let target = ActorAddress::named("worker");
///     ctx.send(WorkMessage { task: "process" }, target).await?;
///     Ok(())
/// }
/// ```
///
/// # Thread Safety
///
/// While the context is `Send + Sync`, actor message handlers are guaranteed
/// to execute sequentially (single-threaded per actor). This means:
/// - No need for internal locking in actors
/// - Safe to mutate actor state directly
/// - Context metadata updates are atomic per message
///
/// # See Also
///
/// - [`Actor`](crate::actor::Actor) - Actor trait that receives context
/// - [`MessageBroker`](crate::broker::MessageBroker) - Broker injected into context
/// - [`ActorAddress`](crate::util::ActorAddress) - Actor addressing and identity
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
    ///
    /// Returns the full actor address including ID and optional name.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Named actor
    /// println!("Actor name: {:?}", ctx.address().name());
    ///
    /// // Anonymous actor
    /// println!("Actor ID: {}", ctx.address().id());
    /// ```
    pub fn address(&self) -> &ActorAddress {
        &self.address
    }

    /// Get the actor's unique ID.
    ///
    /// Returns the UUID that uniquely identifies this actor in the system.
    pub fn id(&self) -> &ActorId {
        &self.id
    }

    /// Get the actor's creation timestamp.
    ///
    /// Returns when this actor was spawned (UTC timezone per §3.2).
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Get the timestamp of the last processed message.
    ///
    /// Returns `None` if the actor hasn't processed any messages yet.
    /// Updated automatically by `record_message()`.
    pub fn last_message_at(&self) -> Option<DateTime<Utc>> {
        self.last_message_at
    }

    /// Get the total number of messages processed.
    ///
    /// This counter increments with each call to `record_message()`.
    /// Useful for monitoring actor throughput and activity.
    pub fn message_count(&self) -> u64 {
        self.message_count
    }

    /// Record that a message was processed.
    ///
    /// Updates `last_message_at` to current time and increments `message_count`.
    /// Typically called by the actor system after successful message handling.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// async fn handle_message<B: MessageBroker<Self::Message>>(
    ///     &mut self,
    ///     msg: Self::Message,
    ///     ctx: &mut ActorContext<Self::Message, B>,
    /// ) -> Result<(), Self::Error> {
    ///     // Process message
    ///     self.do_work(&msg)?;
    ///     
    ///     // Record processing
    ///     ctx.record_message();
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn record_message(&mut self) {
        self.last_message_at = Some(Utc::now());
        self.message_count += 1;
    }

    /// Send a message to another actor (fire-and-forget pattern).
    ///
    /// Publishes the message to the broker, which broadcasts it to all subscribers.
    /// The ActorSystem router will route it to the target actor. This is an async
    /// operation that completes once the message is published to the broker.
    ///
    /// **Does not wait for the recipient to process the message.**
    ///
    /// # Arguments
    ///
    /// * `message` - The message to send (must be serializable)
    /// * `recipient` - Target actor's address
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message published to broker successfully
    /// * `Err(String)` - Failed to publish (broker error, serialization failure)
    ///
    /// # Performance
    ///
    /// - Broker routing: ~212ns per message (from BENCHMARKING.md §6.2)
    /// - Throughput: 4.7M messages/second sustained
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Send work to a worker actor
    /// let worker_addr = ActorAddress::named("worker-1");
    /// ctx.send(WorkMessage { task: "process_data" }, worker_addr).await?;
    ///
    /// // Broadcast to multiple actors (send to each)
    /// for worker in worker_addresses {
    ///     ctx.send(message.clone(), worker).await?;
    /// }
    /// ```
    ///
    /// # See Also
    ///
    /// - [`request()`](#method.request) - For request/reply pattern with timeout
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

    /// Send a request and wait for a reply (request/reply pattern).
    ///
    /// Sends a message and waits for a response from the recipient, with a timeout.
    /// This is useful for query-style interactions where you need a result.
    ///
    /// **Blocks until response received or timeout expires.**
    ///
    /// # Arguments
    ///
    /// * `request` - The request message
    /// * `recipient` - Target actor's address
    /// * `timeout` - Maximum time to wait for response
    ///
    /// # Returns
    ///
    /// * `Ok(Some(envelope))` - Response received within timeout
    /// * `Ok(None)` - Timeout expired, no response
    /// * `Err(String)` - Failed to send request
    ///
    /// # Performance
    ///
    /// Full roundtrip latency: ~737ns (from BENCHMARKING.md §6.2)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use std::time::Duration;
    /// use tokio::sync::oneshot;
    ///
    /// // Query pattern with oneshot channel
    /// let (tx, rx) = oneshot::channel();
    /// let query = QueryMessage::GetStatus(tx);
    ///
    /// ctx.send(query, service_addr).await?;
    ///
    /// // Wait for response with timeout
    /// match tokio::time::timeout(Duration::from_secs(5), rx).await {
    ///     Ok(Ok(status)) => println!("Status: {:?}", status),
    ///     Ok(Err(_)) => println!("Sender dropped"),
    ///     Err(_) => println!("Timeout"),
    /// }
    /// ```
    ///
    /// Alternative using context request method:
    ///
    /// ```rust,ignore
    /// let response = ctx
    ///     .request(QueryMessage::GetMetrics, service_addr, Duration::from_secs(5))
    ///     .await?;
    ///
    /// match response {
    ///     Some(envelope) => println!("Metrics: {:?}", envelope.message),
    ///     None => println!("No response within timeout"),
    /// }
    /// ```
    ///
    /// # See Also
    ///
    /// - [`send()`](#method.send) - For fire-and-forget pattern (no response expected)
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
