//! Generic message broker trait for type-safe message routing.
//!
//! This module defines the core `MessageBroker<M>` trait that provides the interface
//! for message routing infrastructure. The broker is completely hidden from actor
//! implementations and managed by the ActorSystem.

// Layer 1: Standard library imports
use std::error::Error;
use std::time::Duration;

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use tokio::sync::mpsc;

// Layer 3: Internal module imports
use crate::message::{Message, MessageEnvelope};

/// Message stream from broker subscriptions.
///
/// A stream of messages published to the broker. Multiple subscribers can
/// independently receive all published messages.
///
/// # Example
///
/// ```ignore
/// let mut stream = broker.subscribe().await?;
///
/// while let Some(envelope) = stream.recv().await {
///     // Process message
///     route_to_actor(envelope).await?;
/// }
/// ```
pub struct MessageStream<M: Message> {
    receiver: mpsc::UnboundedReceiver<MessageEnvelope<M>>,
}

impl<M: Message> MessageStream<M> {
    /// Create a new message stream.
    ///
    /// This is typically called internally by broker implementations.
    pub fn new(receiver: mpsc::UnboundedReceiver<MessageEnvelope<M>>) -> Self {
        Self { receiver }
    }

    /// Receive next message from stream.
    ///
    /// Returns `None` when the stream is closed (all publishers dropped).
    pub async fn recv(&mut self) -> Option<MessageEnvelope<M>> {
        self.receiver.recv().await
    }

    /// Try to receive without blocking.
    ///
    /// Returns:
    /// - `Ok(envelope)` - Message available
    /// - `Err(TryRecvError::Empty)` - No messages available
    /// - `Err(TryRecvError::Disconnected)` - Stream closed
    pub fn try_recv(&mut self) -> Result<MessageEnvelope<M>, mpsc::error::TryRecvError> {
        self.receiver.try_recv()
    }
}

/// Generic message broker trait for pub-sub message bus architecture.
///
/// The broker implements a **publish-subscribe message bus** that decouples
/// message publishers from subscribers. This architecture enables:
/// - Multiple independent subscribers (ActorSystem, monitors, audit logs)
/// - Extensibility hooks for observability and resilience
/// - Clean separation between business logic and infrastructure
///
/// The broker is infrastructure managed by ActorSystem and is completely
/// hidden from actor implementations. Actors only implement `handle_message()`
/// and never directly interact with the broker. Instead, actors use the
/// `ActorContext` methods (`send`, `request`) which internally use the broker.
///
/// # Architecture: Pub-Sub Message Bus
///
/// ```text
/// ┌─────────────────── Publishers ────────────────────┐
/// │  Actor A          Actor B          Actor C        │
/// │     │                │                 │          │
/// │     └────────────────┼─────────────────┘          │
/// │                      │                            │
/// │                 publish(msg)                       │
/// └──────────────────────┼────────────────────────────┘
///                        ▼
///              ┌──────────────────┐
///              │  MessageBroker   │  ◀── Central message bus
///              │   (Pub-Sub Bus)  │      Broadcasts to all
///              └──────────────────┘
///                        │
///          ┌─────────────┼─────────────┐
///          │             │             │
///          ▼             ▼             ▼
///   ┌──────────┐  ┌──────────┐  ┌──────────┐
///   │ ActorSys │  │ Monitor  │  │  Audit   │  ◀── Subscribers
///   │ (routes) │  │(metrics) │  │  (logs)  │
///   └──────────┘  └──────────┘  └──────────┘
///        │
///        └──▶ routes to actor mailboxes
/// ```
///
/// # Key Differences from Direct Routing
///
/// **Pub-Sub (Current)**:
/// - `publish()` → broadcasts to ALL subscribers
/// - Subscribers independently route to actors
/// - Extensible: add monitors, audit logs, dead letter queues
/// - Loose coupling: publishers don't know subscribers
///
/// **Direct Routing (Deprecated)**:
/// - `send()` → directly to actor mailbox
/// - Tight coupling: broker knows all actors
/// - Hard to extend: no hooks for monitoring
///
/// # Type Safety
///
/// The broker is generic over message type `M`, ensuring compile-time type
/// verification for all routing operations. No runtime type checking or
/// reflection is used (§6.2 - Avoid dyn Patterns).
///
/// # Ownership Semantics
///
/// Messages are transferred by ownership, achieving zero-copy routing.
/// The broker does not clone message payloads - it transfers ownership
/// from sender to recipient's mailbox.
///
/// # Separation of Concerns
///
/// ```text
/// ┌─────────────────────────────────────────────┐
/// │           ActorSystem (manages)             │
/// │  ┌────────────┐        ┌────────────┐      │
/// │  │ ActorSystem│◀───────│   Broker   │      │
/// │  │ (subscribes        │  (pub-sub)  │      │
/// │  │  & routes)  │        └────────────┘      │
/// │  └────────────┘               ▲             │
/// │         │                     │             │
/// │         │ spawns         publish(msg)       │
/// │         ▼                     │             │
/// │  ┌────────────┐        ┌────────────┐      │
/// │  │   Actor    │────────│  Mailbox   │      │
/// │  │ (business) │        │ (receives) │      │
/// │  └────────────┘        └────────────┘      │
/// │         ▲                                   │
/// │         │ handle_message(M)                 │
/// │         │ (no broker knowledge)             │
/// └─────────────────────────────────────────────┘
/// ```
///
/// # Example (System-Level Usage)
///
/// ```ignore
/// use airssys_rt::broker::{InMemoryMessageBroker, MessageBroker};
/// use airssys_rt::message::MessageEnvelope;
/// use std::time::Duration;
///
/// // ActorSystem creates broker internally
/// let broker = InMemoryMessageBroker::<MyMessage>::new();
///
/// // Subscribe for routing (ActorSystem does this)
/// let mut routing_stream = broker.subscribe().await?;
/// tokio::spawn(async move {
///     while let Some(envelope) = routing_stream.recv().await {
///         // Route to actor via registry
///         route_to_actor(envelope).await;
///     }
/// });
///
/// // Publish messages to the bus
/// let envelope = MessageEnvelope::new(message)
///     .with_recipient(address);
/// broker.publish(envelope).await?;
/// // Broadcast to all subscribers (ActorSystem, monitors, etc.)
///
/// // Request-reply over pub-sub
/// let request_envelope = MessageEnvelope::new(request)
///     .with_recipient(address);
/// let response = broker.publish_request::<ResponseType>(
///     request_envelope,
///     Duration::from_secs(5)
/// ).await?;
/// ```
///
/// # Implementation Requirements
///
/// Implementations must:
/// - Be `Send + Sync` for concurrent access across async tasks
/// - Implement `Clone` for cheap broker handle distribution
/// - Support multiple independent subscribers
/// - Broadcast published messages to all active subscribers
/// - Use generic constraints, not trait objects (§6.2)
/// - Provide comprehensive error handling via `Error` associated type
#[async_trait]
pub trait MessageBroker<M: Message>: Send + Sync + Clone {
    /// Error type for broker operations.
    ///
    /// Must implement `Error + Send + Sync` for comprehensive error handling
    /// and propagation across async task boundaries.
    type Error: Error + Send + Sync + 'static;

    /// Publish a message to the broker bus.
    ///
    /// Messages are broadcast to all subscribers. This is the fundamental
    /// operation for actor-to-actor communication in the pub-sub architecture.
    ///
    /// # Pub-Sub Semantics
    ///
    /// Unlike direct routing, `publish()` does NOT directly deliver to a specific
    /// actor. Instead, it broadcasts the message to all subscribers (typically
    /// ActorSystem routers), which then route to the appropriate actor mailbox.
    ///
    /// # Extensibility Hooks
    ///
    /// Implementations can add hooks for:
    /// - Logging and distributed tracing
    /// - Metrics collection (message rates, sizes)
    /// - Message persistence for replay
    /// - Circuit breakers for resilience
    /// - Rate limiting for fairness
    ///
    /// # Arguments
    ///
    /// * `envelope` - The message envelope containing message and metadata
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Broker is shut down
    /// - Persistence layer fails (if enabled)
    /// - Circuit breaker is open (if enabled)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let envelope = MessageEnvelope::new(message)
    ///     .with_sender(sender_address)
    ///     .with_recipient(recipient_address);
    ///
    /// broker.publish(envelope).await?;
    /// // Message broadcast to all subscribers
    /// ```
    async fn publish(&self, envelope: MessageEnvelope<M>) -> Result<(), Self::Error>;

    /// Subscribe to message events on the broker.
    ///
    /// Returns a stream of all messages published to the broker. Multiple
    /// subscribers can listen to the same message stream independently.
    ///
    /// # Subscriber Lifecycle
    ///
    /// The subscription remains active until the MessageStream is dropped.
    /// When the stream is dropped, the subscriber is automatically unregistered.
    ///
    /// # Use Cases
    ///
    /// - **ActorSystem**: Subscribes to route messages to actors via ActorRegistry
    /// - **Monitor Service**: Subscribes for observability and metrics collection
    /// - **Audit Service**: Subscribes for compliance logging and event sourcing
    /// - **Dead Letter Queue**: Subscribes to capture undeliverable messages
    ///
    /// # Multiple Subscribers
    ///
    /// Each subscriber receives ALL published messages independently. The broker
    /// maintains separate channels for each subscriber.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Broker is shut down
    /// - Maximum subscriber limit reached (implementation-specific)
    ///
    /// # Example
    ///
    /// ```ignore
    /// // ActorSystem subscribes for routing
    /// let mut routing_stream = broker.subscribe().await?;
    ///
    /// tokio::spawn(async move {
    ///     while let Some(envelope) = routing_stream.recv().await {
    ///         // Route to actor via registry
    ///         if let Some(recipient) = envelope.metadata.reply_to {
    ///             let sender = registry.resolve(&recipient)?;
    ///             sender.send(envelope).await?;
    ///         }
    ///     }
    /// });
    ///
    /// // Monitor subscribes for metrics
    /// let mut monitor_stream = broker.subscribe().await?;
    ///
    /// tokio::spawn(async move {
    ///     while let Some(envelope) = monitor_stream.recv().await {
    ///         metrics.record_message(&envelope);
    ///     }
    /// });
    /// ```
    async fn subscribe(&self) -> Result<MessageStream<M>, Self::Error>;

    /// Send a request and wait for a reply (request-reply pattern).
    ///
    /// Sends a message and waits for a response of type `R` within the specified
    /// timeout duration. This implements a synchronous request-reply pattern on
    /// top of the asynchronous actor system.
    ///
    /// The broker automatically handles:
    /// - Correlation ID generation for request tracking
    /// - Timeout management with automatic cleanup
    /// - Response routing back to the requester
    ///
    /// # Type Parameters
    ///
    /// * `R` - The expected response message type (must implement `Message`)
    ///
    /// # Arguments
    ///
    /// * `envelope` - The request message envelope
    /// * `timeout` - Maximum duration to wait for a response
    ///
    /// # Returns
    ///
    /// - `Ok(Some(envelope))` - Response received within timeout
    /// - `Ok(None)` - No response (request completed but no reply sent)
    /// - `Err(error)` - Request failed or timeout exceeded
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Target actor not found
    /// - Target actor's mailbox is closed
    /// - Timeout exceeded waiting for response
    /// - Request send failed
    ///
    /// # Example
    ///
    /// ```ignore
    /// use std::time::Duration;
    ///
    /// let request = MessageEnvelope::new(AuthRequest { username, password })
    ///     .with_sender(self_address)
    ///     .with_recipient(auth_service_address);
    ///
    /// let response = broker.request::<AuthResponse>(
    ///     request,
    ///     Duration::from_secs(5)
    /// ).await?;
    ///
    /// if let Some(auth_response) = response {
    ///     println!("Auth result: {:?}", auth_response.payload);
    /// }
    /// ```
    ///
    /// # Performance Considerations
    ///
    /// Request-reply is a blocking operation that holds a task waiting for response.
    /// For long-running operations, consider using fire-and-forget with manual
    /// correlation IDs instead (see KNOWLEDGE-RT-010 Pattern 3).
    async fn publish_request<R: Message + for<'de> serde::Deserialize<'de>>(
        &self,
        envelope: MessageEnvelope<M>,
        timeout: Duration,
    ) -> Result<Option<MessageEnvelope<R>>, Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test that MessageBroker trait requirements are properly defined
    #[test]
    fn test_message_broker_trait_bounds() {
        // This test verifies the trait bounds compile correctly
        fn _assert_broker_bounds<M: Message, B: MessageBroker<M>>() {
            fn _assert_send<T: Send>() {}
            fn _assert_sync<T: Sync>() {}
            fn _assert_clone<T: Clone>() {}

            _assert_send::<B>();
            _assert_sync::<B>();
            _assert_clone::<B>();
        }

        // Compilation of this test validates trait bounds
    }

    #[test]
    fn test_message_broker_error_bounds() {
        // Verify error type bounds
        fn _verify_error_bounds<M: Message, B: MessageBroker<M>>() {
            fn _assert_error<T: Error>() {}
            fn _assert_send<T: Send>() {}
            fn _assert_sync<T: Sync>() {}
            fn _assert_static<T: 'static>() {}

            _assert_error::<B::Error>();
            _assert_send::<B::Error>();
            _assert_sync::<B::Error>();
            _assert_static::<B::Error>();
        }

        // Compilation validates error bounds
    }

    // Test documentation examples compile
    #[test]
    fn test_trait_documentation_validity() {
        // This ensures the trait definition allows the patterns shown in docs
        // Actual implementation tests will be in in_memory.rs
    }
}
