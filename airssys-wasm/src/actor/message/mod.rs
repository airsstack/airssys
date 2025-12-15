//! Message routing and pub/sub system for inter-component communication.
//!
//! This module provides message routing, topic filtering, subscription
//! management, and integration with the airssys-rt message broker.
//!
//! # Architecture
//!
//! The message routing system supports:
//! - Direct component-to-component routing
//! - Publish/subscribe patterns
//! - Topic-based filtering
//! - Integration with airssys-rt MessageBroker
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

// Module declarations
pub mod message_router;
pub mod unified_router;
pub mod message_broker_bridge;
pub mod message_publisher;
pub mod message_filter;
pub mod subscriber_manager;
pub mod actor_system_subscriber;

// Public re-exports
#[doc(inline)]
pub use message_router::MessageRouter;
#[doc(inline)]
pub use unified_router::{RoutingStats, UnifiedRouter};
#[doc(inline)]
pub use message_broker_bridge::{MessageBrokerBridge, MessageBrokerWrapper, SubscriptionHandle};
#[doc(inline)]
pub use message_publisher::MessagePublisher;
#[doc(inline)]
pub use message_filter::TopicFilter;
#[doc(inline)]
pub use subscriber_manager::{SubHandle, SubscriberManager};
#[doc(inline)]
pub use actor_system_subscriber::ActorSystemSubscriber;
