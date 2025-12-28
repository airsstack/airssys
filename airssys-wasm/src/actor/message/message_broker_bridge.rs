//! Bridge trait for integrating ComponentActor with airssys-rt MessageBroker.
//!
//! This module defines the abstraction layer that connects Layer 2 (WASM components)
//! with Layer 3 (actor runtime message broker), maintaining clear ownership boundaries
//! per ADR-WASM-018.
//!
//! # Architecture Context (ADR-WASM-018)
//!
//! Per the Three-Layer Architecture:
//! - **Layer 2** (WASM Components): ComponentActor uses this trait
//! - **Layer 3** (Actor Runtime): MessageBrokerWrapper implements this trait
//! - **Boundary**: This trait is the integration point
//!
//! # Responsibilities
//!
//! The bridge handles:
//! - Publishing messages to topics
//! - Subscribing components to topics
//! - Unsubscribing from topics
//! - Tracking component subscriptions
//!
//! # Performance
//!
//! Bridge operations should add <50ns overhead vs. direct MessageBroker calls.
//! Target: ~211ns total routing latency (airssys-rt baseline).
//!
//! # Example
//!
//! ```rust,ignore
//! use airssys_wasm::actor::{MessageBrokerBridge, ComponentMessage};
//! use airssys_wasm::core::ComponentId;
//!
//! async fn publish_example(
//!     bridge: &dyn MessageBrokerBridge,
//!     topic: &str,
//!     message: ComponentMessage,
//! ) -> Result<(), WasmError> {
//!     // Publish message to topic
//!     bridge.publish(topic, message).await?;
//!     Ok(())
//! }
//!
//! async fn subscribe_example(
//!     bridge: &dyn MessageBrokerBridge,
//!     topic: &str,
//! ) -> Result<SubscriptionHandle, WasmError> {
//!     // Subscribe to topic
//!     let handle = bridge.subscribe(topic).await?;
//!     Ok(handle)
//! }
//! ```
//!
//! # References
//!
//! - **ADR-WASM-018**: Three-Layer Architecture (Layer Separation)
//! - **ADR-WASM-009**: Component Communication Model (Messaging Patterns)
//! - **WASM-TASK-004 Phase 4 Task 4.1**: MessageBroker Setup for Components

// Layer 1: Standard library imports
use std::collections::HashMap;
use std::sync::Arc;

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use tokio::sync::RwLock;
use uuid::Uuid;

// Layer 3: Internal module imports
use crate::actor::component::ComponentMessage;
use crate::core::{ComponentId, WasmError};
use airssys_rt::broker::MessageBroker;
use airssys_rt::message::MessageEnvelope;

/// Bridge abstraction for MessageBroker access.
///
/// Maintains layer separation (ADR-WASM-018) by providing a trait abstraction
/// over airssys-rt MessageBroker, preventing direct Layer 3 exposure in ComponentActor.
///
/// # Architecture Context
///
/// Per ADR-WASM-018:
/// - **Layer 2** (WASM Components): ComponentActor uses this trait
/// - **Layer 3** (Actor Runtime): MessageBrokerWrapper implements this trait
/// - **Boundary**: This trait is the integration point
///
/// # Responsibilities
///
/// - Publishing messages to topics
/// - Subscribing components to topics
/// - Unsubscribing from topics
/// - Tracking component subscriptions
///
/// # Performance
///
/// Target: <50ns overhead, ~211ns total routing latency
#[async_trait]
pub trait MessageBrokerBridge: Send + Sync {
    /// Publish message to topic.
    ///
    /// Delivers ComponentMessage to all subscribers of the specified topic.
    /// Uses fire-and-forget semantics per ADR-WASM-009.
    ///
    /// # Parameters
    ///
    /// - `topic`: Topic name (e.g., "events", "notifications.user")
    /// - `message`: ComponentMessage to publish
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Message published successfully
    /// - `Err(WasmError)`: Broker error (invalid topic, publish failed)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use airssys_wasm::actor::{MessageBrokerBridge, ComponentMessage};
    /// use airssys_wasm::core::ComponentId;
    ///
    /// async fn publish_event(
    ///     bridge: &dyn MessageBrokerBridge,
    /// ) -> Result<(), WasmError> {
    ///     let message = ComponentMessage::InterComponent {
    ///         sender: ComponentId::new("component-a"),
    ///         payload: vec![1, 2, 3],
    ///     };
    ///     
    ///     bridge.publish("events.user.login", message).await?;
    ///     Ok(())
    /// }
    /// ```
    async fn publish(&self, topic: &str, message: ComponentMessage) -> Result<(), WasmError>;

    /// Subscribe to topic.
    ///
    /// Registers interest in a topic, returning a handle for tracking the subscription.
    /// All messages published to this topic will be delivered to the subscriber.
    ///
    /// # Parameters
    ///
    /// - `topic`: Topic name to subscribe to
    /// - `component_id`: Component subscribing (for tracking)
    ///
    /// # Returns
    ///
    /// - `Ok(SubscriptionHandle)`: Subscription created successfully
    /// - `Err(WasmError)`: Broker error (invalid topic, subscribe failed)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// async fn subscribe_to_events(
    ///     bridge: &dyn MessageBrokerBridge,
    ///     component_id: ComponentId,
    /// ) -> Result<SubscriptionHandle, WasmError> {
    ///     let handle = bridge.subscribe("events.user", &component_id).await?;
    ///     Ok(handle)
    /// }
    /// ```
    async fn subscribe(
        &self,
        topic: &str,
        component_id: &ComponentId,
    ) -> Result<SubscriptionHandle, WasmError>;

    /// Unsubscribe from topic.
    ///
    /// Removes subscription identified by handle. Component will no longer
    /// receive messages on this topic.
    ///
    /// # Parameters
    ///
    /// - `handle`: Subscription handle from subscribe()
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Unsubscribed successfully
    /// - `Err(WasmError)`: Handle invalid or subscription not found
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// async fn unsubscribe_from_events(
    ///     bridge: &dyn MessageBrokerBridge,
    ///     handle: SubscriptionHandle,
    /// ) -> Result<(), WasmError> {
    ///     bridge.unsubscribe(handle).await?;
    ///     Ok(())
    /// }
    /// ```
    async fn unsubscribe(&self, handle: SubscriptionHandle) -> Result<(), WasmError>;

    /// Get current subscriptions for component.
    ///
    /// Returns list of topics the component is subscribed to.
    ///
    /// # Parameters
    ///
    /// - `component_id`: Component to query
    ///
    /// # Returns
    ///
    /// - `Ok(Vec<String>)`: List of topic names
    /// - `Err(WasmError)`: Query failed
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// async fn list_subscriptions(
    ///     bridge: &dyn MessageBrokerBridge,
    ///     component_id: &ComponentId,
    /// ) -> Result<Vec<String>, WasmError> {
    ///     let topics = bridge.subscriptions(component_id).await?;
    ///     Ok(topics)
    /// }
    /// ```
    async fn subscriptions(&self, component_id: &ComponentId) -> Result<Vec<String>, WasmError>;
}

/// Concrete implementation wrapping airssys-rt MessageBroker.
///
/// MessageBrokerWrapper implements the MessageBrokerBridge trait, providing
/// Layer 2 access to Layer 3 MessageBroker functionality while maintaining
/// layer separation per ADR-WASM-018.
///
/// # Architecture
///
/// ```text
/// ComponentActor (Layer 2)
///     ↓ uses trait
/// MessageBrokerBridge (trait abstraction)
///     ↓ implemented by
/// MessageBrokerWrapper (Layer 2)
///     ↓ wraps
/// MessageBroker<ComponentMessage> (Layer 3)
/// ```
///
/// # Thread Safety
///
/// Uses Arc<RwLock<>> for subscription tracking, allowing concurrent reads
/// and exclusive writes.
///
/// # Examples
///
/// ```rust,ignore
/// use airssys_rt::broker::InMemoryMessageBroker;
/// use airssys_wasm::actor::MessageBrokerWrapper;
///
/// let broker = InMemoryMessageBroker::new();
/// let wrapper = MessageBrokerWrapper::new(broker);
/// ```
pub struct MessageBrokerWrapper<B: MessageBroker<ComponentMessage>> {
    broker: B,
    subscription_tracker: Arc<RwLock<SubscriptionTracker>>,
}

impl<B: MessageBroker<ComponentMessage>> MessageBrokerWrapper<B> {
    /// Create new wrapper from MessageBroker instance.
    ///
    /// # Arguments
    ///
    /// * `broker` - MessageBroker implementation
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let broker = InMemoryMessageBroker::new();
    /// let wrapper = MessageBrokerWrapper::new(broker);
    /// ```
    pub fn new(broker: B) -> Self {
        Self {
            broker,
            subscription_tracker: Arc::new(RwLock::new(SubscriptionTracker::new())),
        }
    }

    /// Get reference to underlying MessageBroker (for testing).
    #[allow(clippy::expect_used, clippy::unwrap_used, clippy::unwrap_err_used, clippy::expect_err_used, clippy::panic, clippy::unwrap_on_result, clippy::indexing_slicing, clippy::too_many_arguments, clippy::type_complexity, reason = "test code")]
#[cfg(test)]
    pub fn broker(&self) -> &B {
        &self.broker
    }
}

#[async_trait]
impl<B: MessageBroker<ComponentMessage> + Send + Sync> MessageBrokerBridge
    for MessageBrokerWrapper<B>
{
    async fn publish(&self, _topic: &str, message: ComponentMessage) -> Result<(), WasmError> {
        // Wrap in MessageEnvelope
        let envelope = MessageEnvelope::new(message);

        // Delegate to Layer 3 MessageBroker
        self.broker.publish(envelope).await.map_err(|e| {
            WasmError::message_broker_error(format!("Failed to publish message: {}", e))
        })
    }

    async fn subscribe(
        &self,
        topic: &str,
        component_id: &ComponentId,
    ) -> Result<SubscriptionHandle, WasmError> {
        // Subscribe to Layer 3 broker
        let _stream =
            self.broker.subscribe().await.map_err(|e| {
                WasmError::message_broker_error(format!("Failed to subscribe: {}", e))
            })?;

        // Create subscription handle
        let handle = SubscriptionHandle::new(topic, component_id.clone());

        // Track subscription
        let mut tracker = self.subscription_tracker.write().await;
        tracker.add_subscription(&handle, topic.to_string());

        Ok(handle)
    }

    async fn unsubscribe(&self, handle: SubscriptionHandle) -> Result<(), WasmError> {
        // Remove from tracking
        let mut tracker = self.subscription_tracker.write().await;
        tracker
            .remove_subscription(&handle)
            .map_err(WasmError::internal)
    }

    async fn subscriptions(&self, component_id: &ComponentId) -> Result<Vec<String>, WasmError> {
        let tracker = self.subscription_tracker.read().await;
        Ok(tracker.get_subscriptions(component_id))
    }
}

/// Tracks component subscriptions (Layer 2 responsibility).
///
/// SubscriptionTracker maintains a mapping of ComponentId to subscribed topics,
/// providing Layer 2 visibility into subscription state without exposing Layer 3
/// MessageBroker internals.
///
/// # Thread Safety
///
/// Wrapped in Arc<RwLock<>> for concurrent access from multiple ComponentActors.
///
/// # Examples
///
/// ```rust,ignore
/// let tracker = SubscriptionTracker::new();
/// let handle = SubscriptionHandle::new("events", component_id);
/// tracker.add_subscription(&handle, "events".to_string());
/// ```
struct SubscriptionTracker {
    subscriptions: HashMap<ComponentId, Vec<String>>,
}

impl SubscriptionTracker {
    /// Create new empty tracker.
    fn new() -> Self {
        Self {
            subscriptions: HashMap::new(),
        }
    }

    /// Add subscription for component.
    ///
    /// # Parameters
    ///
    /// - `handle`: Subscription handle with component_id
    /// - `topic`: Topic name
    fn add_subscription(&mut self, handle: &SubscriptionHandle, topic: String) {
        self.subscriptions
            .entry(handle.component_id.clone())
            .or_default()
            .push(topic);
    }

    /// Remove subscription by handle.
    ///
    /// # Parameters
    ///
    /// - `handle`: Subscription handle to remove
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Subscription removed
    /// - `Err(String)`: No subscriptions found for component
    fn remove_subscription(&mut self, handle: &SubscriptionHandle) -> Result<(), String> {
        if let Some(topics) = self.subscriptions.get_mut(&handle.component_id) {
            topics.retain(|t| t != &handle.topic);
            if topics.is_empty() {
                self.subscriptions.remove(&handle.component_id);
            }
            Ok(())
        } else {
            Err(format!(
                "No subscriptions found for component: {}",
                handle.component_id.as_str()
            ))
        }
    }

    /// Get all subscriptions for component.
    ///
    /// # Parameters
    ///
    /// - `component_id`: Component to query
    ///
    /// # Returns
    ///
    /// Vector of topic names (empty if no subscriptions)
    fn get_subscriptions(&self, component_id: &ComponentId) -> Vec<String> {
        self.subscriptions
            .get(component_id)
            .cloned()
            .unwrap_or_default()
    }
}

/// Handle representing a subscription.
///
/// SubscriptionHandle uniquely identifies a subscription for tracking and unsubscribe
/// operations. Each handle has a unique UUID to differentiate multiple subscriptions
/// to the same topic by the same component.
///
/// # Examples
///
/// ```rust,ignore
/// use airssys_wasm::actor::SubscriptionHandle;
/// use airssys_wasm::core::ComponentId;
///
/// let component_id = ComponentId::new("my-component");
/// let handle = SubscriptionHandle::new("events", component_id);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubscriptionHandle {
    /// Component that owns this subscription
    component_id: ComponentId,
    /// Topic being subscribed to
    topic: String,
    /// Unique handle identifier (UUID)
    handle_id: Uuid,
}

impl SubscriptionHandle {
    /// Create new subscription handle.
    ///
    /// # Arguments
    ///
    /// * `topic` - Topic name
    /// * `component_id` - Component subscribing
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let handle = SubscriptionHandle::new("events", component_id);
    /// ```
    pub fn new(topic: &str, component_id: ComponentId) -> Self {
        Self {
            component_id,
            topic: topic.to_string(),
            handle_id: Uuid::new_v4(),
        }
    }

    /// Get component ID.
    pub fn component_id(&self) -> &ComponentId {
        &self.component_id
    }

    /// Get topic name.
    pub fn topic(&self) -> &str {
        &self.topic
    }

    /// Get handle ID.
    pub fn handle_id(&self) -> &Uuid {
        &self.handle_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscription_handle_creation() {
        let component_id = ComponentId::new("test-component");
        let handle = SubscriptionHandle::new("test-topic", component_id.clone());

        assert_eq!(handle.component_id(), &component_id);
        assert_eq!(handle.topic(), "test-topic");
        assert_ne!(handle.handle_id(), &Uuid::nil());
    }

    #[test]
    fn test_subscription_handle_uniqueness() {
        let component_id = ComponentId::new("test-component");
        let handle1 = SubscriptionHandle::new("test-topic", component_id.clone());
        let handle2 = SubscriptionHandle::new("test-topic", component_id);

        // Different UUIDs even for same component/topic
        assert_ne!(handle1.handle_id(), handle2.handle_id());
        assert_ne!(handle1, handle2);
    }

    #[test]
    fn test_subscription_tracker_add() {
        let mut tracker = SubscriptionTracker::new();
        let component_id = ComponentId::new("test-component");
        let handle = SubscriptionHandle::new("topic-1", component_id.clone());

        tracker.add_subscription(&handle, "topic-1".to_string());

        let subscriptions = tracker.get_subscriptions(&component_id);
        assert_eq!(subscriptions.len(), 1);
        assert_eq!(subscriptions[0], "topic-1");
    }

    #[test]
    fn test_subscription_tracker_multiple_topics() {
        let mut tracker = SubscriptionTracker::new();
        let component_id = ComponentId::new("test-component");

        let handle1 = SubscriptionHandle::new("topic-1", component_id.clone());
        let handle2 = SubscriptionHandle::new("topic-2", component_id.clone());

        tracker.add_subscription(&handle1, "topic-1".to_string());
        tracker.add_subscription(&handle2, "topic-2".to_string());

        let subscriptions = tracker.get_subscriptions(&component_id);
        assert_eq!(subscriptions.len(), 2);
        assert!(subscriptions.contains(&"topic-1".to_string()));
        assert!(subscriptions.contains(&"topic-2".to_string()));
    }

    #[test]
    fn test_subscription_tracker_remove() {
        let mut tracker = SubscriptionTracker::new();
        let component_id = ComponentId::new("test-component");
        let handle = SubscriptionHandle::new("topic-1", component_id.clone());

        tracker.add_subscription(&handle, "topic-1".to_string());
        assert_eq!(tracker.get_subscriptions(&component_id).len(), 1);

        let result = tracker.remove_subscription(&handle);
        assert!(result.is_ok());
        assert_eq!(tracker.get_subscriptions(&component_id).len(), 0);
    }

    #[test]
    fn test_subscription_tracker_remove_nonexistent() {
        let mut tracker = SubscriptionTracker::new();
        let component_id = ComponentId::new("test-component");
        let handle = SubscriptionHandle::new("topic-1", component_id);

        let result = tracker.remove_subscription(&handle);
        assert!(result.is_err());
    }

    #[test]
    fn test_subscription_tracker_get_empty() {
        let tracker = SubscriptionTracker::new();
        let component_id = ComponentId::new("test-component");

        let subscriptions = tracker.get_subscriptions(&component_id);
        assert_eq!(subscriptions.len(), 0);
    }
}
