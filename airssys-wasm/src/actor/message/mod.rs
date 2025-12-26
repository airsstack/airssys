//! Message routing and pub/sub system for inter-component communication.
//!
//! This module provides message routing, topic filtering, subscription
//! management, and integration with airssys-rt message broker.
//!
//! # Architecture
//!
//! The message routing system supports:
//! - Direct component-to-component routing
//! - Publish/subscribe patterns
//! - Topic-based filtering
//! - Integration with airssys-rt MessageBroker
//! - Request-response patterns with correlation tracking
//!
//! # Module Organization
//!
//! - `message_router` - Basic message routing
//! - `unified_router` - Advanced routing with statistics
//! - `message_broker_bridge` - Bridge to MessageBroker
//! - `message_publisher` - Publishing interface
//! - `message_filter` - Topic filtering logic
//! - `subscriber_manager` - Subscription management
//! - `actor_system_subscriber` - Actor system integration
//! - `correlation_tracker` - Request-response correlation tracking
//! - `request_response` - Request/response message types
//! - `timeout_handler` - Timeout enforcement for pending requests

// Module declarations
pub mod actor_system_subscriber;
pub mod correlation_tracker;
pub mod message_broker_bridge;
pub mod message_filter;
pub mod message_publisher;
pub mod message_router;
pub mod request_response;
pub mod subscriber_manager;
pub mod timeout_handler;
pub mod unified_router;

// Public re-exports
#[doc(inline)]
pub use actor_system_subscriber::ActorSystemSubscriber;
#[doc(inline)]
pub use correlation_tracker::CorrelationTracker;
#[doc(inline)]
pub use message_broker_bridge::{MessageBrokerBridge, MessageBrokerWrapper, SubscriptionHandle};
#[doc(inline)]
pub use message_filter::TopicFilter;
#[doc(inline)]
pub use message_publisher::MessagePublisher;
#[doc(inline)]
pub use message_router::MessageRouter;
#[doc(inline)]
pub use request_response::RequestMessage;
#[doc(inline)]
pub use subscriber_manager::{SubHandle, SubscriberManager};
#[doc(inline)]
pub use timeout_handler::TimeoutHandler;
#[doc(inline)]
pub use unified_router::{RoutingStats, UnifiedRouter};

// Re-export shared messaging types from core/messaging for compatibility
// (Task 1.1: Moved from actor/message/ to core/messaging/)
#[doc(inline)]
pub use crate::core::messaging::{CorrelationId, PendingRequest, RequestError, ResponseMessage};
// ADR-WASM-022: Moved from runtime/ to prevent circular dependency
pub mod messaging_subscription;
#[doc(inline)]
pub use messaging_subscription::{MessagingSubscriptionService, SubscriptionStatus};
