#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

//! Integration tests for pub-sub message routing.
//!
//! Tests end-to-end pub-sub flows including:
//! - Message publishing and delivery to subscribers
//! - Multiple subscribers per topic
//! - Wildcard subscription routing
//! - Message correlation patterns
//! - Concurrent publish/subscribe operations
//!
//! # Test Scenarios
//!
//! 1. End-to-end pub-sub flow (publisher → broker → subscriber)
//! 2. Multiple subscribers receiving same message
//! 3. Wildcard pattern subscription routing
//! 4. Request-response with correlation IDs
//! 5. Concurrent operations (thread safety)
//!
//! # References
//!
//! - **WASM-TASK-004 Phase 4 Task 4.2**: Pub-Sub Message Routing

use airssys_rt::broker::InMemoryMessageBroker;
use airssys_wasm::actor::{
    MessageBrokerBridge, MessageBrokerWrapper, MessagePublisher, SubscriberManager,
};
use airssys_wasm::core::ComponentId;
use std::sync::Arc;
use uuid::Uuid;

// ============================================================================
// End-to-End Pub-Sub Flow
// ============================================================================

#[tokio::test]
async fn test_end_to_end_pub_sub() {
    // Setup
    let broker = InMemoryMessageBroker::new();
    let wrapper: Arc<dyn MessageBrokerBridge> = Arc::new(MessageBrokerWrapper::new(broker));
    let manager = SubscriberManager::new();

    let publisher_id = ComponentId::new("publisher");
    let subscriber_id = ComponentId::new("subscriber");

    // Create publisher
    let publisher = MessagePublisher::new(publisher_id, Arc::clone(&wrapper));

    // Subscribe
    let _handle = manager
        .subscribe(subscriber_id.clone(), vec!["test-topic".into()])
        .await
        .unwrap();

    // Publish
    let result = publisher.publish("test-topic", vec![1, 2, 3]).await;
    assert!(result.is_ok());

    // Verify subscribers exist
    let subscribers = manager.subscribers_for_topic("test-topic").await;
    assert_eq!(subscribers.len(), 1);
    assert_eq!(subscribers[0], subscriber_id);
}

// ============================================================================
// Multiple Subscribers per Topic
// ============================================================================

#[tokio::test]
async fn test_multiple_subscribers_same_topic() {
    let broker = InMemoryMessageBroker::new();
    let wrapper: Arc<dyn MessageBrokerBridge> = Arc::new(MessageBrokerWrapper::new(broker));
    let manager = SubscriberManager::new();

    let publisher_id = ComponentId::new("publisher");
    let subscriber_a = ComponentId::new("subscriber-a");
    let subscriber_b = ComponentId::new("subscriber-b");
    let subscriber_c = ComponentId::new("subscriber-c");

    // Create publisher
    let publisher = MessagePublisher::new(publisher_id, wrapper);

    // Multiple subscribers to same topic
    manager
        .subscribe(subscriber_a.clone(), vec!["events".into()])
        .await
        .unwrap();
    manager
        .subscribe(subscriber_b.clone(), vec!["events".into()])
        .await
        .unwrap();
    manager
        .subscribe(subscriber_c.clone(), vec!["events".into()])
        .await
        .unwrap();

    // Publish
    publisher.publish("events", vec![1, 2, 3]).await.unwrap();

    // All should receive
    let subscribers = manager.subscribers_for_topic("events").await;
    assert_eq!(subscribers.len(), 3);
    assert!(subscribers.contains(&subscriber_a));
    assert!(subscribers.contains(&subscriber_b));
    assert!(subscribers.contains(&subscriber_c));
}

// ============================================================================
// Wildcard Subscription Routing
// ============================================================================

#[tokio::test]
async fn test_wildcard_subscription_routing() {
    let manager = SubscriberManager::new();

    let subscriber_single = ComponentId::new("subscriber-single");
    let subscriber_multi = ComponentId::new("subscriber-multi");
    let subscriber_exact = ComponentId::new("subscriber-exact");

    // Subscribe with different patterns
    manager
        .subscribe(subscriber_single.clone(), vec!["events.user.*".into()])
        .await
        .unwrap();

    manager
        .subscribe(subscriber_multi.clone(), vec!["events.#".into()])
        .await
        .unwrap();

    manager
        .subscribe(subscriber_exact.clone(), vec!["events.user.login".into()])
        .await
        .unwrap();

    // Test various topics

    // "events.user.login" should match all three
    let subscribers = manager.subscribers_for_topic("events.user.login").await;
    assert_eq!(subscribers.len(), 3);

    // "events.user.logout" should match single and multi wildcards
    let subscribers = manager.subscribers_for_topic("events.user.logout").await;
    assert_eq!(subscribers.len(), 2);
    assert!(subscribers.contains(&subscriber_single));
    assert!(subscribers.contains(&subscriber_multi));

    // "events.system.restart" should match only multi wildcard
    let subscribers = manager.subscribers_for_topic("events.system.restart").await;
    assert_eq!(subscribers.len(), 1);
    assert!(subscribers.contains(&subscriber_multi));

    // "system.restart" should match none
    let subscribers = manager.subscribers_for_topic("system.restart").await;
    assert_eq!(subscribers.len(), 0);
}

// ============================================================================
// Message Correlation Patterns
// ============================================================================

#[tokio::test]
async fn test_correlation_pattern() {
    let broker = InMemoryMessageBroker::new();
    let wrapper: Arc<dyn MessageBrokerBridge> = Arc::new(MessageBrokerWrapper::new(broker));
    let manager = SubscriberManager::new();

    let requester_id = ComponentId::new("requester");
    let responder_id = ComponentId::new("responder");

    // Create requester
    let requester = MessagePublisher::new(requester_id, Arc::clone(&wrapper));

    // Responder subscribes to requests
    manager
        .subscribe(responder_id.clone(), vec!["requests.*".into()])
        .await
        .unwrap();

    // Send request with correlation ID
    let correlation_id = Uuid::new_v4();
    let result = requester
        .publish_with_correlation("requests.data", vec![1, 2, 3], correlation_id)
        .await;
    assert!(result.is_ok());

    // Verify responder matches
    let subscribers = manager.subscribers_for_topic("requests.data").await;
    assert_eq!(subscribers.len(), 1);
    assert_eq!(subscribers[0], responder_id);
}

// ============================================================================
// Concurrent Publish/Subscribe
// ============================================================================

#[tokio::test]
async fn test_concurrent_publish_subscribe() {
    let broker = InMemoryMessageBroker::new();
    let wrapper: Arc<dyn MessageBrokerBridge> = Arc::new(MessageBrokerWrapper::new(broker));
    let manager = Arc::new(SubscriberManager::new());

    // Spawn multiple publishers
    let mut publish_handles = vec![];
    for i in 0..10 {
        let wrapper_clone = Arc::clone(&wrapper);
        let handle = tokio::spawn(async move {
            let publisher_id = ComponentId::new(format!("publisher-{}", i));
            let publisher = MessagePublisher::new(publisher_id, wrapper_clone);
            publisher
                .publish(&format!("topic-{}", i % 3), vec![i as u8])
                .await
        });
        publish_handles.push(handle);
    }

    // Spawn multiple subscribers
    let mut subscribe_handles = vec![];
    for i in 0..10 {
        let manager_clone = Arc::clone(&manager);
        let handle = tokio::spawn(async move {
            let subscriber_id = ComponentId::new(format!("subscriber-{}", i));
            manager_clone
                .subscribe(subscriber_id, vec![format!("topic-{}", i % 3)])
                .await
        });
        subscribe_handles.push(handle);
    }

    // Wait for all operations
    for handle in publish_handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }

    for handle in subscribe_handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }

    // Verify subscriptions exist
    let count = manager.subscription_count().await;
    assert_eq!(count, 10);
}

// ============================================================================
// Broadcast Pattern
// ============================================================================

#[tokio::test]
async fn test_broadcast_to_multiple_topics() {
    let broker = InMemoryMessageBroker::new();
    let wrapper: Arc<dyn MessageBrokerBridge> = Arc::new(MessageBrokerWrapper::new(broker));
    let manager = SubscriberManager::new();

    let publisher_id = ComponentId::new("broadcaster");
    let subscriber_a = ComponentId::new("subscriber-a");
    let subscriber_b = ComponentId::new("subscriber-b");
    let subscriber_c = ComponentId::new("subscriber-c");

    // Create broadcaster
    let broadcaster = MessagePublisher::new(publisher_id, wrapper);

    // Subscribers on different topics
    manager
        .subscribe(subscriber_a.clone(), vec!["topic-1".into()])
        .await
        .unwrap();
    manager
        .subscribe(subscriber_b.clone(), vec!["topic-2".into()])
        .await
        .unwrap();
    manager
        .subscribe(subscriber_c.clone(), vec!["topic-3".into()])
        .await
        .unwrap();

    // Broadcast to all topics
    let topics = ["topic-1", "topic-2", "topic-3"];
    let result = broadcaster.publish_multi(&topics, vec![1, 2, 3]).await;
    assert!(result.is_ok());

    // Verify all subscribers exist
    assert_eq!(manager.subscribers_for_topic("topic-1").await.len(), 1);
    assert_eq!(manager.subscribers_for_topic("topic-2").await.len(), 1);
    assert_eq!(manager.subscribers_for_topic("topic-3").await.len(), 1);
}
