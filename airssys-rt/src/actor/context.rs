//! Actor context with metadata and state management.
//!
//! Provides zero-cost actor context with generic constraints for type safety.

// Layer 1: Standard library imports
use std::marker::PhantomData;

// Layer 2: Third-party crate imports
use chrono::{DateTime, Utc}; // §3.2 MANDATORY

// Layer 3: Internal module imports
use crate::message::Message;
use crate::util::{ActorAddress, ActorId};

/// Actor context with metadata for zero-cost abstractions.
///
/// Generic over message type M for compile-time type safety.
///
/// # Examples
///
/// ```rust
/// use airssys_rt::{ActorContext, Message};
/// use airssys_rt::util::ActorAddress;
///
/// #[derive(Debug, Clone)]
/// struct MyMessage;
///
/// impl Message for MyMessage {
///     const MESSAGE_TYPE: &'static str = "my_message";
/// }
///
/// let address = ActorAddress::named("my-actor");
/// let context = ActorContext::<MyMessage>::new(address);
/// assert_eq!(context.address().name(), Some("my-actor"));
/// ```
pub struct ActorContext<M: Message> {
    address: ActorAddress,
    id: ActorId,
    created_at: DateTime<Utc>,
    last_message_at: Option<DateTime<Utc>>,
    message_count: u64,
    _marker: PhantomData<M>,
}

impl<M: Message> ActorContext<M> {
    /// Create a new actor context.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::{ActorContext, Message};
    /// use airssys_rt::util::ActorAddress;
    ///
    /// #[derive(Debug, Clone)]
    /// struct TestMessage;
    ///
    /// impl Message for TestMessage {
    ///     const MESSAGE_TYPE: &'static str = "test";
    /// }
    ///
    /// let address = ActorAddress::anonymous();
    /// let context = ActorContext::<TestMessage>::new(address);
    /// ```
    pub fn new(address: ActorAddress) -> Self {
        Self {
            id: *address.id(),
            address,
            created_at: Utc::now(), // §3.2
            last_message_at: None,
            message_count: 0,
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestMessage;

    impl Message for TestMessage {
        const MESSAGE_TYPE: &'static str = "test";
    }

    #[test]
    fn test_context_creation() {
        let address = ActorAddress::anonymous();
        let context = ActorContext::<TestMessage>::new(address);

        assert_eq!(context.message_count(), 0);
        assert!(context.last_message_at().is_none());
    }

    #[test]
    fn test_context_address_accessor() {
        let address = ActorAddress::named("test-actor");
        let context = ActorContext::<TestMessage>::new(address.clone());

        assert_eq!(context.address(), &address);
        assert_eq!(context.address().name(), Some("test-actor"));
    }

    #[test]
    fn test_context_id_accessor() {
        let address = ActorAddress::anonymous();
        let id = *address.id();
        let context = ActorContext::<TestMessage>::new(address);

        assert_eq!(context.id(), &id);
    }

    #[test]
    fn test_record_message() {
        let address = ActorAddress::anonymous();
        let mut context = ActorContext::<TestMessage>::new(address);

        assert_eq!(context.message_count(), 0);
        assert!(context.last_message_at().is_none());

        context.record_message();

        assert_eq!(context.message_count(), 1);
        assert!(context.last_message_at().is_some());
    }

    #[test]
    fn test_multiple_message_records() {
        let address = ActorAddress::anonymous();
        let mut context = ActorContext::<TestMessage>::new(address);

        for i in 1..=10 {
            context.record_message();
            assert_eq!(context.message_count(), i);
        }
    }

    #[test]
    fn test_created_at_timestamp() {
        let address = ActorAddress::anonymous();
        let before = Utc::now();
        let context = ActorContext::<TestMessage>::new(address);
        let after = Utc::now();

        let created = context.created_at();
        assert!(created >= before && created <= after);
    }
}
