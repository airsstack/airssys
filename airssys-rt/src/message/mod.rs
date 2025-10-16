//! Message system with zero-cost abstractions.
//!
//! This module provides the message passing infrastructure for actor communication,
//! implementing type-safe, high-performance message routing with optional pub/sub.
//!
//! # Components
//!
//! - [`Message`] - Core trait that all messages must implement
//! - [`MessageEnvelope`] - Message wrapper with routing metadata
//! - [`MessagePriority`] - Priority levels for message ordering (High, Normal, Low)
//!
//! # Design Philosophy
//!
//! - **Type safety**: Compile-time message type verification
//! - **Zero-cost abstractions**: No heap allocations for simple messages
//! - **Performance**: Stack allocation, no vtables, minimal overhead
//! - **Flexibility**: Optional metadata (priority, expiration, correlation)
//!
//! # Performance Characteristics
//!
//! Based on RT-TASK-008 baseline measurements (Oct 16, 2025):
//!
//! - **Message creation**: ~737ns (with envelope and metadata)
//! - **Message throughput**: ~4.7M messages/sec (4.7x target of 1M/sec)
//! - **Direct processing**: ~31.5ns/message (actor handle_message)
//! - **Broker routing**: ~212ns/message (with pub/sub)
//! - **Mailbox operations**: ~182ns/message (send to bounded mailbox)
//! - **Expiration check**: <10ns (chrono DateTime comparison)
//!
//! Source: `BENCHMARKING.md` §6.2
//!
//! # Message Patterns
//!
//! ## 1. Fire-and-Forget
//!
//! ```rust,ignore
//! use airssys_rt::prelude::*;
//!
//! #[derive(Debug, Clone)]
//! struct LogMessage(String);
//!
//! impl Message for LogMessage {
//!     const MESSAGE_TYPE: &'static str = "log";
//! }
//!
//! // Send and don't wait for response
//! ctx.send(target_address, LogMessage("event happened".to_string()))?;
//! ```
//!
//! ## 2. Request-Reply
//!
//! ```rust,ignore
//! use airssys_rt::prelude::*;
//! use tokio::sync::oneshot;
//!
//! #[derive(Debug)]
//! struct QueryMessage {
//!     data: String,
//!     reply: oneshot::Sender<String>,
//! }
//!
//! impl Message for QueryMessage {
//!     const MESSAGE_TYPE: &'static str = "query";
//! }
//!
//! // Send with timeout and wait for response
//! let result = ctx.request(
//!     target_address,
//!     QueryMessage { data: "get_value".to_string(), reply },
//!     Duration::from_millis(100)
//! ).await?;
//! ```
//!
//! ## 3. Priority Messages
//!
//! ```rust,ignore
//! use airssys_rt::prelude::*;
//!
//! #[derive(Debug, Clone)]
//! struct CriticalAlert(String);
//!
//! impl Message for CriticalAlert {
//!     const MESSAGE_TYPE: &'static str = "alert";
//!     
//!     // Override default priority
//!     fn priority(&self) -> MessagePriority {
//!         MessagePriority::High
//!     }
//! }
//!
//! // High priority messages processed first
//! ctx.send(target_address, CriticalAlert("system failure".to_string()))?;
//! ```
//!
//! ## 4. Expiring Messages
//!
//! ```rust,ignore
//! use airssys_rt::prelude::*;
//! use chrono::{Duration, Utc};
//!
//! #[derive(Debug, Clone)]
//! struct TimeoutMessage(String);
//!
//! impl Message for TimeoutMessage {
//!     const MESSAGE_TYPE: &'static str = "timeout";
//!     
//!     // Messages expire if not processed within window
//!     fn expires_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
//!         Some(Utc::now() + Duration::milliseconds(500))
//!     }
//! }
//! ```
//!
//! # Serialization Support
//!
//! Messages can optionally derive serde traits for network transport:
//!
//! ```rust,ignore
//! use serde::{Deserialize, Serialize};
//! use airssys_rt::prelude::*;
//!
//! #[derive(Debug, Clone, Serialize, Deserialize)]
//! struct NetworkMessage {
//!     payload: String,
//! }
//!
//! impl Message for NetworkMessage {
//!     const MESSAGE_TYPE: &'static str = "network";
//! }
//! ```
//!
//! # Module Organization (§4.3)
//!
//! This mod.rs file contains ONLY module declarations and re-exports.
//! Implementation code is in individual module files:
//!
//! - `traits.rs` - Message trait and MessagePriority enum
//! - `envelope.rs` - MessageEnvelope implementation
//!
//! # See Also
//!
//! - [`actor`](crate::actor) - Actor system that consumes messages
//! - [`broker`](crate::broker) - Message broker for pub/sub patterns
//! - [`mailbox`](crate::mailbox) - Mailbox implementations for message queuing
//!
//! Provides core message traits and types for type-safe message passing
//! in the actor system. Built on compile-time type identification and
//! generic constraints for maximum performance.

pub mod envelope;
pub mod traits;

pub use envelope::MessageEnvelope;
pub use traits::{Message, MessagePriority};
