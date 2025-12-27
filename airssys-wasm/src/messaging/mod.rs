//! Inter-component communication infrastructure.
//!
//! This module provides messaging infrastructure for communication
//! between WASM components, including:
//!
//! - MessageBroker integration
//! - Request-response patterns
//! - Fire-and-forget messaging
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

// Module declarations (§4.3 - declaration-only pattern)
pub mod messaging_service;
pub mod router;
pub mod fire_and_forget;
pub mod request_response;
pub mod codec;
pub mod topics; // Phase 2

// Public re-exports
pub use messaging_service::{MessagingService, MessagingStats};
pub use router::{ResponseRouter, ResponseRouterStats};
pub use fire_and_forget::FireAndForget;
pub use request_response::{RequestResponse, RequestError};
pub use codec::MulticodecCodec;
