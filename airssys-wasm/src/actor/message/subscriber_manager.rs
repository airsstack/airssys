//! Subscription management for topic-based message routing.
//!
//! This module implements `SubscriberManager`, managing component subscriptions
//! to topics and providing efficient lookup for message delivery.
//!
//! # Architecture Context (ADR-WASM-009)
//!
//! Per ADR-WASM-009 Component Communication Model:
//! - **Subscription Management**: Tracks which components subscribe to which topics
//! - **Subscriber Lookup**: Fast resolution of subscribers for a given topic
//! - **Wildcard Support**: Evaluates topic filters with wildcards
//!
//! # Responsibilities
//!
//! - Subscribe components to topics (with wildcard patterns)
//! - Unsubscribe components from topics
//! - Query all subscribers matching a topic
//! - Thread-safe concurrent subscription management
//!
//! # Performance
//!
//! Target: O(n) subscriber lookup where n = number of subscriptions
//!
//! # Example
//!
//! ```rust,ignore
//! use airssys_wasm::actor::SubscriberManager;
//! use airssys_wasm::core::ComponentId;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), WasmError> {
//!     let manager = SubscriberManager::new();
//!     let component_id = ComponentId::new("my-component");
//!     
//!     // Subscribe to topics with wildcards
//!     let handle = manager.subscribe(
//!         component_id.clone(),
//!         vec!["events.user.*".to_string()],
//!     ).await?;
//!     
//!     // Find subscribers for a topic
//!     let subscribers = manager.subscribers_for_topic("events.user.login").await;
//!     assert_eq!(subscribers.len(), 1);
//!     
//!     // Unsubscribe
//!     manager.unsubscribe(&handle).await?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! # References
//!
//! - **ADR-WASM-009**: Component Communication Model (Subscription Management)
//! - **ADR-WASM-018**: Three-Layer Architecture (Layer Separation)
//! - **WASM-TASK-004 Phase 4 Task 4.2**: Pub-Sub Message Routing

// Layer 1: Standard library imports
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

// Layer 2: Third-party crate imports
use tokio::sync::RwLock;
use uuid::Uuid;

// Layer 3: Internal module imports
use super::TopicFilter;
use crate::core::{ComponentId, WasmError};

/// Manages component subscriptions to topics.
///
/// SubscriberManager maintains a thread-safe registry of component subscriptions,
/// allowing components to subscribe to topics (with wildcard patterns) and
/// providing fast lookup of all subscribers for a given topic.
///
/// # Architecture
///
/// ```text
/// ComponentActor A → subscribe("events.user.*")
///                       ↓
/// ComponentActor B → subscribe("events.#")
///                       ↓
///                SubscriberManager
///                       ↓
/// publish("events.user.login")
///                       ↓
/// subscribers_for_topic() → [A, B]
/// ```
///
/// # Thread Safety
///
/// Uses `Arc<RwLock<HashMap>>` for concurrent access:
/// - Multiple readers (subscribers_for_topic)
/// - Exclusive writer (subscribe/unsubscribe)
///
/// # Examples
///
/// ```rust,ignore
/// use airssys_wasm::actor::SubscriberManager;
/// use airssys_wasm::core::ComponentId;
///
/// let manager = SubscriberManager::new();
/// let component_a = ComponentId::new("component-a");
/// let component_b = ComponentId::new("component-b");
///
/// // Subscribe to different patterns
/// manager.subscribe(component_a, vec!["events.*".into()]).await?;
/// manager.subscribe(component_b, vec!["events.user.*".into()]).await?;
///
/// // Find all subscribers for a topic
/// let subscribers = manager.subscribers_for_topic("events.user.login").await;
/// assert_eq!(subscribers.len(), 2);
/// ```
pub struct SubscriberManager {
    /// Subscriptions indexed by component ID
    subscriptions: Arc<RwLock<HashMap<ComponentId, Vec<Subscription>>>>,
}

/// Individual subscription record.
///
/// Subscription represents a component's subscription to one or more topics,
/// storing the topic patterns and a compiled filter for efficient matching.
#[derive(Debug, Clone)]
struct Subscription {
    /// Topic patterns subscribed to
    topics: Vec<String>,
    /// Compiled topic filter for matching
    filter: TopicFilter,
    /// Unique subscription identifier
    subscription_id: Uuid,
}

impl SubscriberManager {
    /// Create new SubscriberManager.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let manager = SubscriberManager::new();
    /// ```
    pub fn new() -> Self {
        Self {
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Subscribe component to topics.
    ///
    /// Creates a subscription for the specified component to receive messages
    /// published to topics matching the provided patterns. Patterns support
    /// MQTT-style wildcards (`*` and `#`).
    ///
    /// # Parameters
    ///
    /// * `component_id` - Component subscribing
    /// * `topics` - Topic patterns (e.g., `["events.user.*", "system.#"]`)
    ///
    /// # Returns
    ///
    /// - `Ok(SubHandle)`: Subscription created successfully
    /// - `Err(WasmError)`: Invalid topic pattern or subscription failed
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let handle = manager.subscribe(
    ///     component_id,
    ///     vec!["events.user.*".into(), "system.alerts".into()],
    /// ).await?;
    /// ```
    ///
    /// # Thread Safety
    ///
    /// Acquires exclusive write lock on subscriptions HashMap.
    pub async fn subscribe(
        &self,
        component_id: ComponentId,
        topics: Vec<String>,
    ) -> Result<SubHandle, WasmError> {
        // Compile topic filter
        let filter = TopicFilter::from_patterns(topics.iter().map(|s| s.as_str()).collect());

        let subscription_id = Uuid::new_v4();

        let subscription = Subscription {
            topics: topics.clone(),
            filter,
            subscription_id,
        };

        // Add to subscriptions
        let mut subs = self.subscriptions.write().await;
        subs.entry(component_id.clone())
            .or_default()
            .push(subscription);

        Ok(SubHandle::new(component_id, topics, subscription_id))
    }

    /// Get all subscribers matching topic.
    ///
    /// Returns list of component IDs subscribed to topics that match the
    /// provided topic name (considering wildcard patterns).
    ///
    /// # Parameters
    ///
    /// * `topic` - Topic name to match (e.g., "events.user.login")
    ///
    /// # Returns
    ///
    /// Vector of ComponentIds with matching subscriptions
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let subscribers = manager.subscribers_for_topic("events.user.login").await;
    /// for subscriber in subscribers {
    ///     println!("Deliver to: {}", subscriber.as_str());
    /// }
    /// ```
    ///
    /// # Performance
    ///
    /// Time complexity: O(n) where n = total number of subscriptions
    /// Each subscription's filter is evaluated against the topic.
    ///
    /// # Thread Safety
    ///
    /// Acquires shared read lock on subscriptions HashMap.
    pub async fn subscribers_for_topic(&self, topic: &str) -> Vec<ComponentId> {
        let subs = self.subscriptions.read().await;

        // Use HashSet to prevent duplicates if component has overlapping subscriptions
        let mut seen = HashSet::new();
        let mut subscribers = Vec::new();

        for (component_id, component_subs) in subs.iter() {
            for sub in component_subs {
                if sub.filter.matches(topic) && seen.insert(component_id.clone()) {
                    // Component not yet added and filter matches
                    subscribers.push(component_id.clone());
                    break; // Component already added to HashSet, skip remaining subs
                }
            }
        }

        subscribers
    }

    /// Unsubscribe component from topics.
    ///
    /// Removes the subscription identified by the handle. The component will
    /// no longer receive messages on the subscribed topics.
    ///
    /// # Parameters
    ///
    /// * `handle` - Subscription handle from subscribe()
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Unsubscribed successfully
    /// - `Err(WasmError)`: Subscription not found
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let handle = manager.subscribe(component_id, topics).await?;
    /// // ... later ...
    /// manager.unsubscribe(&handle).await?;
    /// ```
    ///
    /// # Thread Safety
    ///
    /// Acquires exclusive write lock on subscriptions HashMap.
    pub async fn unsubscribe(&self, handle: &SubHandle) -> Result<(), WasmError> {
        let mut subs = self.subscriptions.write().await;

        if let Some(component_subs) = subs.get_mut(&handle.component_id) {
            // Remove subscription matching the subscription_id
            let initial_len = component_subs.len();
            component_subs.retain(|sub| sub.subscription_id != handle.subscription_id);

            if component_subs.len() < initial_len {
                // Subscription was removed
                if component_subs.is_empty() {
                    // No subscriptions left, remove component entry
                    subs.remove(&handle.component_id);
                }
                Ok(())
            } else {
                Err(WasmError::internal(format!(
                    "Subscription not found: {:?}",
                    handle.subscription_id
                )))
            }
        } else {
            Err(WasmError::internal(format!(
                "No subscriptions found for component: {}",
                handle.component_id.as_str()
            )))
        }
    }

    /// Get all subscriptions for component.
    ///
    /// Returns list of topic patterns the component is subscribed to.
    ///
    /// # Parameters
    ///
    /// * `component_id` - Component to query
    ///
    /// # Returns
    ///
    /// Vector of topic pattern strings
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let topics = manager.get_subscriptions(&component_id).await;
    /// for topic in topics {
    ///     println!("Subscribed to: {}", topic);
    /// }
    /// ```
    pub async fn get_subscriptions(&self, component_id: &ComponentId) -> Vec<String> {
        let subs = self.subscriptions.read().await;

        subs.get(component_id)
            .map(|component_subs| {
                component_subs
                    .iter()
                    .flat_map(|sub| sub.topics.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get total number of subscriptions across all components.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let count = manager.subscription_count().await;
    /// println!("Total subscriptions: {}", count);
    /// ```
    pub async fn subscription_count(&self) -> usize {
        let subs = self.subscriptions.read().await;
        subs.values().map(|v| v.len()).sum()
    }
}

impl Default for SubscriberManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle representing a subscription.
///
/// SubHandle uniquely identifies a subscription for tracking and
/// unsubscribe operations.
///
/// # Examples
///
/// ```rust,ignore
/// let handle = manager.subscribe(component_id, topics).await?;
/// // Use handle to unsubscribe later
/// manager.unsubscribe(&handle).await?;
/// ```
#[derive(Debug, Clone)]
pub struct SubHandle {
    /// Component that owns this subscription
    component_id: ComponentId,
    /// Topic patterns subscribed to
    topics: Vec<String>,
    /// Unique subscription identifier
    subscription_id: Uuid,
}

impl SubHandle {
    /// Create new subscription handle.
    ///
    /// # Arguments
    ///
    /// * `component_id` - Component subscribing
    /// * `topics` - Topic patterns
    /// * `subscription_id` - Unique identifier
    pub fn new(component_id: ComponentId, topics: Vec<String>, subscription_id: Uuid) -> Self {
        Self {
            component_id,
            topics,
            subscription_id,
        }
    }

    /// Get component ID.
    pub fn component_id(&self) -> &ComponentId {
        &self.component_id
    }

    /// Get subscribed topics.
    pub fn topics(&self) -> &[String] {
        &self.topics
    }

    /// Get subscription ID.
    pub fn subscription_id(&self) -> &Uuid {
        &self.subscription_id
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)] // Test code: unwrap is acceptable
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_subscribe_single_topic() {
        let manager = SubscriberManager::new();
        let component_id = ComponentId::new("test-component");

        let handle = manager
            .subscribe(component_id.clone(), vec!["test-topic".into()])
            .await
            .unwrap();

        assert_eq!(handle.component_id(), &component_id);
        assert_eq!(handle.topics().len(), 1);
    }

    #[tokio::test]
    async fn test_subscribe_multiple_topics() {
        let manager = SubscriberManager::new();
        let component_id = ComponentId::new("test-component");

        let topics = vec!["topic-1".into(), "topic-2".into(), "topic-3".into()];
        let handle = manager
            .subscribe(component_id.clone(), topics)
            .await
            .unwrap();

        assert_eq!(handle.topics().len(), 3);
    }

    #[tokio::test]
    async fn test_subscribers_for_topic_exact_match() {
        let manager = SubscriberManager::new();
        let component_id = ComponentId::new("test-component");

        manager
            .subscribe(component_id.clone(), vec!["events.user.login".into()])
            .await
            .unwrap();

        let subscribers = manager.subscribers_for_topic("events.user.login").await;
        assert_eq!(subscribers.len(), 1);
        assert_eq!(subscribers[0], component_id);
    }

    #[tokio::test]
    async fn test_subscribers_for_topic_wildcard_match() {
        let manager = SubscriberManager::new();
        let component_id = ComponentId::new("test-component");

        manager
            .subscribe(component_id.clone(), vec!["events.user.*".into()])
            .await
            .unwrap();

        let subscribers = manager.subscribers_for_topic("events.user.login").await;
        assert_eq!(subscribers.len(), 1);
        assert_eq!(subscribers[0], component_id);
    }

    #[tokio::test]
    async fn test_subscribers_for_topic_multi_wildcard() {
        let manager = SubscriberManager::new();
        let component_id = ComponentId::new("test-component");

        manager
            .subscribe(component_id.clone(), vec!["events.#".into()])
            .await
            .unwrap();

        let subscribers = manager
            .subscribers_for_topic("events.user.login.success")
            .await;
        assert_eq!(subscribers.len(), 1);
    }

    #[tokio::test]
    async fn test_subscribers_for_topic_multiple_subscribers() {
        let manager = SubscriberManager::new();
        let component_a = ComponentId::new("component-a");
        let component_b = ComponentId::new("component-b");

        manager
            .subscribe(component_a.clone(), vec!["events.#".into()])
            .await
            .unwrap();
        manager
            .subscribe(component_b.clone(), vec!["events.user.*".into()])
            .await
            .unwrap();

        let subscribers = manager.subscribers_for_topic("events.user.login").await;
        assert_eq!(subscribers.len(), 2);
    }

    #[tokio::test]
    async fn test_subscribers_for_topic_no_match() {
        let manager = SubscriberManager::new();
        let component_id = ComponentId::new("test-component");

        manager
            .subscribe(component_id, vec!["events.user.*".into()])
            .await
            .unwrap();

        let subscribers = manager.subscribers_for_topic("system.restart").await;
        assert_eq!(subscribers.len(), 0);
    }

    #[tokio::test]
    async fn test_unsubscribe() {
        let manager = SubscriberManager::new();
        let component_id = ComponentId::new("test-component");

        let handle = manager
            .subscribe(component_id.clone(), vec!["test-topic".into()])
            .await
            .unwrap();

        let subscribers = manager.subscribers_for_topic("test-topic").await;
        assert_eq!(subscribers.len(), 1);

        manager.unsubscribe(&handle).await.unwrap();

        let subscribers = manager.subscribers_for_topic("test-topic").await;
        assert_eq!(subscribers.len(), 0);
    }

    #[tokio::test]
    async fn test_unsubscribe_nonexistent() {
        let manager = SubscriberManager::new();
        let component_id = ComponentId::new("test-component");

        let handle = SubHandle::new(component_id, vec!["test-topic".into()], Uuid::new_v4());

        let result = manager.unsubscribe(&handle).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_subscriptions() {
        let manager = SubscriberManager::new();
        let component_id = ComponentId::new("test-component");

        let topics = vec!["topic-1".into(), "topic-2".into()];
        manager
            .subscribe(component_id.clone(), topics.clone())
            .await
            .unwrap();

        let retrieved = manager.get_subscriptions(&component_id).await;
        assert_eq!(retrieved.len(), 2);
        assert!(retrieved.contains(&"topic-1".to_string()));
        assert!(retrieved.contains(&"topic-2".to_string()));
    }

    #[tokio::test]
    async fn test_get_subscriptions_empty() {
        let manager = SubscriberManager::new();
        let component_id = ComponentId::new("test-component");

        let retrieved = manager.get_subscriptions(&component_id).await;
        assert_eq!(retrieved.len(), 0);
    }

    #[tokio::test]
    async fn test_subscription_count() {
        let manager = SubscriberManager::new();
        let component_a = ComponentId::new("component-a");
        let component_b = ComponentId::new("component-b");

        manager
            .subscribe(component_a, vec!["topic-1".into()])
            .await
            .unwrap();
        manager
            .subscribe(component_b, vec!["topic-2".into()])
            .await
            .unwrap();

        let count = manager.subscription_count().await;
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_multiple_subscriptions_same_component() {
        let manager = SubscriberManager::new();
        let component_id = ComponentId::new("test-component");

        manager
            .subscribe(component_id.clone(), vec!["topic-1".into()])
            .await
            .unwrap();
        manager
            .subscribe(component_id.clone(), vec!["topic-2".into()])
            .await
            .unwrap();

        let count = manager.subscription_count().await;
        assert_eq!(count, 2);

        let topics = manager.get_subscriptions(&component_id).await;
        assert_eq!(topics.len(), 2);
    }
}
