//! Mailbox system for actor message queuing.
//!
//! This module provides the mailbox infrastructure for actors, including:
//! - Generic `MailboxReceiver<M>` and `MailboxSender<M>` traits
//! - Bounded mailboxes with configurable capacity
//! - Unbounded mailboxes for high-throughput scenarios
//! - Backpressure strategies for flow control
//!
//! # Example
//!
//! ```ignore
//! // Full example will be available after Phase 2 (BoundedMailbox implementation)
//! use airssys_rt::mailbox::{BoundedMailbox, BackpressureStrategy};
//! use airssys_rt::message::{Message, MessageEnvelope};
//!
//! #[derive(Debug, Clone)]
//! struct MyMessage { data: String }
//!
//! impl Message for MyMessage {
//!     const MESSAGE_TYPE: &'static str = "my_message";
//! }
//!
//! # async fn example() {
//! // Create bounded mailbox with capacity 100
//! let (mut receiver, sender) = BoundedMailbox::<MyMessage>::new(100);
//!
//! // Send a message
//! let msg = MyMessage { data: "Hello".to_string() };
//! sender.send(MessageEnvelope::new(msg)).await.unwrap();
//!
//! // Receive the message
//! let envelope = receiver.recv().await.unwrap();
//! println!("Received: {:?}", envelope.payload.data);
//! # }
//! ```

// §4.3 MANDATORY: Module declarations only (no implementation code)
pub mod backpressure;
pub mod traits;

// Re-exports for convenience
pub use backpressure::BackpressureStrategy;
pub use traits::{
    MailboxCapacity, MailboxError, MailboxMetrics, MailboxReceiver, MailboxSender, TryRecvError,
};
