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

// Layer 3: Internal module imports
use crate::message::{Message, MessageEnvelope};

/// Generic message broker trait for type-safe message routing.
///
/// The broker is infrastructure managed by ActorSystem and is completely
/// hidden from actor implementations. Actors only implement `handle_message()`
/// and never directly interact with the broker. Instead, actors use the
/// `ActorContext` methods (`send`, `request`) which internally use the broker.
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
/// │  │ ActorSystem│───────▶│   Broker   │      │
/// │  └────────────┘        └────────────┘      │
/// │         │                     │             │
/// │         │ spawns              │ routes      │
/// │         ▼                     ▼             │
/// │  ┌────────────┐        ┌────────────┐      │
/// │  │   Actor    │        │  Mailbox   │      │
/// │  │ (business) │◀───────│ (receives) │      │
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
/// // System registers actors with their mailbox senders
/// broker.register_actor(address, mailbox_sender)?;
///
/// // System routes fire-and-forget messages
/// let envelope = MessageEnvelope::new(message)
///     .with_recipient(address);
/// broker.send(envelope).await?;
///
/// // System handles request-reply patterns
/// let request_envelope = MessageEnvelope::new(request)
///     .with_recipient(address);
/// let response = broker.request::<ResponseType>(
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
/// - Use generic constraints, not trait objects (§6.2)
/// - Provide comprehensive error handling via `Error` associated type
#[async_trait]
pub trait MessageBroker<M: Message>: Send + Sync + Clone {
    /// Error type for broker operations.
    ///
    /// Must implement `Error + Send + Sync` for comprehensive error handling
    /// and propagation across async task boundaries.
    type Error: Error + Send + Sync + 'static;

    /// Send a message to an actor by address (fire-and-forget).
    ///
    /// Transfers ownership of the message envelope to the target actor's
    /// mailbox. Returns error if actor not found or mailbox closed.
    ///
    /// This is a non-blocking operation that completes when the message is
    /// enqueued in the target mailbox, not when it's processed.
    ///
    /// # Arguments
    ///
    /// * `envelope` - The message envelope containing the message and routing metadata
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Target actor not found in registry
    /// - Target actor's mailbox is closed (actor stopped)
    /// - Mailbox is full and backpressure rejects the message
    ///
    /// # Example
    ///
    /// ```ignore
    /// let envelope = MessageEnvelope::new(message)
    ///     .with_sender(sender_address)
    ///     .with_recipient(target_address);
    ///
    /// broker.send(envelope).await?;
    /// // Message ownership transferred to target mailbox
    /// ```
    async fn send(&self, envelope: MessageEnvelope<M>) -> Result<(), Self::Error>;

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
    async fn request<R: Message + for<'de> serde::Deserialize<'de>>(
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
