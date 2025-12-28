//! Unit tests for message routing components.
//!
//! Tests TopicFilter, MessagePublisher, and SubscriberManager functionality
//! including wildcard pattern matching, multi-topic publishing, and subscription
//! management.
//!
//! # Test Coverage
//!
//! - TopicFilter wildcard matching (single-level `*` and multi-level `#`)
//! - MessagePublisher publish operations (single, multi, correlation)
//! - SubscriberManager subscription lifecycle (subscribe, unsubscribe, lookup)
//! - Edge cases and error conditions
//!
//! # References
//!
//! - **WASM-TASK-004 Phase 4 Task 4.2**: Pub-Sub Message Routing

#![allow(clippy::unwrap_used)] // Test code: unwrap is acceptable

use airssys_rt::broker::InMemoryMessageBroker;
use airssys_wasm::actor::{MessageBrokerWrapper, MessagePublisher, SubscriberManager, TopicFilter};
use airssys_wasm::core::ComponentId;
use std::sync::Arc;
use uuid::Uuid;

// ============================================================================
// TopicFilter Tests (Wildcard Matching)
// ============================================================================

#[tokio::test]
async fn test_topic_filter_single_wildcard() {
    let filter = TopicFilter::from_patterns(vec!["events.user.*"]);

    // Should match single-level wildcard
    assert!(filter.matches("events.user.login"));
    assert!(filter.matches("events.user.logout"));
    assert!(filter.matches("events.user.register"));

    // Should NOT match different prefix
    assert!(!filter.matches("events.system.restart"));

    // Should NOT match too many levels
    assert!(!filter.matches("events.user.login.success"));
}

#[tokio::test]
async fn test_topic_filter_multi_wildcard() {
    let filter = TopicFilter::from_patterns(vec!["events.#"]);

    // Should match zero or more segments
    assert!(filter.matches("events"));
    assert!(filter.matches("events.user"));
    assert!(filter.matches("events.user.login"));
    assert!(filter.matches("events.user.login.success"));

    // Should NOT match different prefix
    assert!(!filter.matches("system.restart"));
}

#[tokio::test]
async fn test_topic_filter_exact_match() {
    let filter = TopicFilter::from_patterns(vec!["events.user.login"]);

    // Should match exactly
    assert!(filter.matches("events.user.login"));

    // Should NOT match different topics
    assert!(!filter.matches("events.user.logout"));
    assert!(!filter.matches("events.user"));
    assert!(!filter.matches("events.user.login.success"));
}

#[tokio::test]
async fn test_topic_filter_multiple_patterns() {
    let filter = TopicFilter::from_patterns(vec!["events.user.*", "system.#", "metrics.cpu"]);

    // Should match any pattern
    assert!(filter.matches("events.user.login"));
    assert!(filter.matches("system.restart"));
    assert!(filter.matches("system.restart.initiated"));
    assert!(filter.matches("metrics.cpu"));

    // Should NOT match none
    assert!(!filter.matches("events.system.error"));
    assert!(!filter.matches("metrics.memory"));
}

#[tokio::test]
async fn test_topic_filter_empty_patterns() {
    let filter = TopicFilter::from_patterns(vec![]);

    // Should match nothing
    assert!(!filter.matches("events.user.login"));
    assert!(!filter.matches("system.restart"));
}

// ============================================================================
// MessagePublisher Tests
// ============================================================================

#[tokio::test]
async fn test_message_publisher_publish_single() {
    let broker = InMemoryMessageBroker::new();
    let wrapper = Arc::new(MessageBrokerWrapper::new(broker));
    let component_id = ComponentId::new("test-publisher");
    let publisher = MessagePublisher::new(component_id.clone(), wrapper);

    // Publish to single topic
    let result = publisher.publish("test-topic", vec![1, 2, 3]).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_message_publisher_publish_multi() {
    let broker = InMemoryMessageBroker::new();
    let wrapper = Arc::new(MessageBrokerWrapper::new(broker));
    let component_id = ComponentId::new("test-publisher");
    let publisher = MessagePublisher::new(component_id, wrapper);

    // Publish to multiple topics
    let topics = ["topic-1", "topic-2", "topic-3"];
    let result = publisher.publish_multi(&topics, vec![1, 2, 3]).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_message_publisher_publish_with_correlation() {
    let broker = InMemoryMessageBroker::new();
    let wrapper = Arc::new(MessageBrokerWrapper::new(broker));
    let component_id = ComponentId::new("test-publisher");
    let publisher = MessagePublisher::new(component_id, wrapper);

    // Publish with correlation ID
    let correlation_id = Uuid::new_v4();
    let result = publisher
        .publish_with_correlation("request-topic", vec![1, 2, 3], correlation_id)
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_message_publisher_empty_payload() {
    let broker = InMemoryMessageBroker::new();
    let wrapper = Arc::new(MessageBrokerWrapper::new(broker));
    let component_id = ComponentId::new("test-publisher");
    let publisher = MessagePublisher::new(component_id, wrapper);

    // Should handle empty payload
    let result = publisher.publish("test-topic", vec![]).await;
    assert!(result.is_ok());
}

// ============================================================================
// SubscriberManager Tests
// ============================================================================

#[tokio::test]
async fn test_subscriber_manager_subscribe() {
    let manager = SubscriberManager::new();
    let component_id = ComponentId::new("test-component");

    // Subscribe to topic
    let handle = manager
        .subscribe(component_id.clone(), vec!["test-topic".into()])
        .await;

    assert!(handle.is_ok());
    let handle = handle.unwrap();
    assert_eq!(handle.component_id(), &component_id);
}

#[tokio::test]
async fn test_subscriber_manager_subscribers_for_topic() {
    let manager = SubscriberManager::new();
    let component_id = ComponentId::new("test-component");

    // Subscribe
    manager
        .subscribe(component_id.clone(), vec!["events.user.*".into()])
        .await
        .unwrap();

    // Find subscribers
    let subscribers = manager.subscribers_for_topic("events.user.login").await;
    assert_eq!(subscribers.len(), 1);
    assert_eq!(subscribers[0], component_id);
}

#[tokio::test]
async fn test_subscriber_manager_multiple_subscribers() {
    let manager = SubscriberManager::new();
    let component_a = ComponentId::new("component-a");
    let component_b = ComponentId::new("component-b");

    // Both subscribe to overlapping patterns
    manager
        .subscribe(component_a.clone(), vec!["events.#".into()])
        .await
        .unwrap();
    manager
        .subscribe(component_b.clone(), vec!["events.user.*".into()])
        .await
        .unwrap();

    // Both should receive (multi-wildcard and single-wildcard both match)
    let subscribers = manager.subscribers_for_topic("events.user.login").await;
    assert_eq!(subscribers.len(), 2);
}

#[tokio::test]
async fn test_subscriber_manager_unsubscribe() {
    let manager = SubscriberManager::new();
    let component_id = ComponentId::new("test-component");

    // Subscribe
    let handle = manager
        .subscribe(component_id.clone(), vec!["test-topic".into()])
        .await
        .unwrap();

    // Verify subscription
    let subscribers = manager.subscribers_for_topic("test-topic").await;
    assert_eq!(subscribers.len(), 1);

    // Unsubscribe
    let result = manager.unsubscribe(&handle).await;
    assert!(result.is_ok());

    // Verify unsubscribed
    let subscribers = manager.subscribers_for_topic("test-topic").await;
    assert_eq!(subscribers.len(), 0);
}

#[tokio::test]
async fn test_subscriber_manager_no_match() {
    let manager = SubscriberManager::new();
    let component_id = ComponentId::new("test-component");

    // Subscribe to specific pattern
    manager
        .subscribe(component_id, vec!["events.user.*".into()])
        .await
        .unwrap();

    // Query different topic
    let subscribers = manager.subscribers_for_topic("system.restart").await;
    assert_eq!(subscribers.len(), 0);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[tokio::test]
async fn test_topic_filter_edge_case_empty_topic() {
    let filter = TopicFilter::from_patterns(vec!["events"]);

    // Empty topic should not match
    assert!(!filter.matches(""));
}

#[tokio::test]
async fn test_topic_filter_edge_case_root_wildcard() {
    let filter = TopicFilter::from_patterns(vec!["#"]);

    // Should match everything
    assert!(filter.matches("events"));
    assert!(filter.matches("events.user"));
    assert!(filter.matches("system.restart"));
}

#[tokio::test]
async fn test_subscriber_manager_edge_case_duplicate_subscriptions() {
    let manager = SubscriberManager::new();
    let component_id = ComponentId::new("test-component");

    // Subscribe twice to same topic
    manager
        .subscribe(component_id.clone(), vec!["test-topic".into()])
        .await
        .unwrap();
    manager
        .subscribe(component_id.clone(), vec!["test-topic".into()])
        .await
        .unwrap();

    // Should find component (potentially twice, depending on implementation)
    let subscribers = manager.subscribers_for_topic("test-topic").await;
    assert!(!subscribers.is_empty());
}

#[tokio::test]
async fn test_subscriber_manager_edge_case_unsubscribe_nonexistent() {
    let manager = SubscriberManager::new();
    let component_id = ComponentId::new("test-component");

    // Create handle for non-existent subscription
    let handle = airssys_wasm::actor::SubHandle::new(
        component_id,
        vec!["test-topic".into()],
        Uuid::new_v4(),
    );

    // Unsubscribe should fail
    let result = manager.unsubscribe(&handle).await;
    assert!(result.is_err());
}
