//! Inter-component communication infrastructure.
//!
//! This module provides messaging infrastructure for communication
//! between WASM components, including:
//!
//! - MessageBroker integration
//! - Request-response patterns
//! - Topic-based pub-sub (Phase 2)
//! - Multicodec message encoding
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │              Messaging Module            │
//! │  ┌────────────────────────────────┐     │
//! │  │  • MessageBroker integration  │     │
//! │  │  • Request-response routing │     │
//! │  │  • Fire-and-forget messaging│     │
//! │  │  • Metrics and monitoring     │     │
//! │  └────────────────────────────────┘     │
//! └─────────────────────────────────────────┘
//!                         ↓ uses
//! ┌─────────────────────────────────────────┐
//! │    airssys-rt InMemoryMessageBroker │
//! └─────────────────────────────────────────┘
//! ```
//!
//! # Architecture (Planned for Phase 2+)
//!
//! Fire-and-forget and request-response patterns are currently
//! implemented as host functions in `runtime/async_host.rs`.
//! These stub files are kept for potential future helper functions
//! if needed.
//!
//! # Message Types
//!
//! Message types are defined in `src/core/messaging.rs` as the
//! `MessageType` enum:
//!
//! - `MessageType::FireAndForget` - One-way message, no response
//! - `MessageType::Request` - Request expecting response
//! - `MessageType::Response` - Response to a request
//!
//! Components use these message types directly.
//!

// Module declarations (§4.3 - declaration-only pattern)
pub mod codec;
pub mod messaging_service;
pub mod router;
pub mod topics;

// Public re-exports
pub use messaging_service::{
    MessageReceptionMetrics, MessageReceptionStats, MessagingService, MessagingStats,
};
pub use router::{ResponseRouter, ResponseRouterStats};

// Re-export message types from core for convenience
pub use crate::core::messaging::MessageType;
