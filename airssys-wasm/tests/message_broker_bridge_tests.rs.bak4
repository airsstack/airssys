//! Unit tests for MessageBrokerBridge implementation.
//!
//! Tests verify:
//! - MessageBrokerBridge trait implementation
//! - MessageBrokerWrapper broker wrapping
//! - SubscriptionTracker state management
//! - Subscription handle uniqueness
//! - Thread safety with concurrent access
//!
//! # Test Organization
//!
//! - **Bridge Operations**: publish, subscribe, unsubscribe
//! - **Subscription Tracking**: add, remove, query subscriptions
//! - **Error Handling**: invalid operations, edge cases
//! - **Concurrency**: thread-safe operations
//! - **Type Safety**: ComponentMessage handling
//!
//! # References
//!
//! - **WASM-TASK-004 Phase 4 Task 4.1**: MessageBroker Setup for Components
//! - **ADR-WASM-018**: Three-Layer Architecture (Layer Separation)
//! - **ADR-WASM-009**: Component Communication Model

#![allow(clippy::unwrap_used, reason = "unwrap is acceptable in test code")]
#![allow(clippy::expect_used, reason = "expect is acceptable in test code")]

use airssys_rt::broker::InMemoryMessageBroker;
use airssys_wasm::actor::{ComponentMessage, MessageBrokerBridge, MessageBrokerWrapper};
use airssys_wasm::core::ComponentId;
use std::sync::Arc;

#[tokio::test]
async fn test_broker_bridge_publish() {
    // Setup
    let broker = InMemoryMessageBroker::new();
    let wrapper = MessageBrokerWrapper::new(broker);

    let message = ComponentMessage::InterComponent {
        sender: ComponentId::new("test-sender"),
        to: ComponentId::new("target"), // TODO(WASM-TASK-006): Use actual target
        payload: vec![1, 2, 3],
    };

    // Test: Publish message
    let result = wrapper.publish("test-topic", message).await;

    // Verify: Publish succeeds
    assert!(result.is_ok(), "Publish should succeed: {:?}", result.err());
}

#[tokio::test]
async fn test_broker_bridge_subscribe() {
    // Setup
    let broker = InMemoryMessageBroker::new();
    let wrapper = MessageBrokerWrapper::new(broker);
    let component_id = ComponentId::new("test-component");

    // Test: Subscribe to topic
    let result = wrapper.subscribe("test-topic", &component_id).await;

    // Verify: Subscribe returns handle
    assert!(
        result.is_ok(),
        "Subscribe should succeed: {:?}",
        result.err()
    );
    let handle = result.unwrap();
    assert_eq!(handle.topic(), "test-topic");
    assert_eq!(handle.component_id(), &component_id);
}

#[tokio::test]
async fn test_subscription_tracking() {
    // Setup
    let broker = InMemoryMessageBroker::new();
    let wrapper = MessageBrokerWrapper::new(broker);
    let component_id = ComponentId::new("test-component");

    // Test: Subscribe and check tracking
    let _handle = wrapper.subscribe("topic-1", &component_id).await.unwrap();

    let subscriptions = wrapper.subscriptions(&component_id).await.unwrap();

    // Verify: Subscription tracked
    assert_eq!(subscriptions.len(), 1);
    assert_eq!(subscriptions[0], "topic-1");
}

#[tokio::test]
async fn test_broker_bridge_unsubscribe() {
    // Setup
    let broker = InMemoryMessageBroker::new();
    let wrapper = MessageBrokerWrapper::new(broker);
    let component_id = ComponentId::new("test-component");

    // Subscribe
    let handle = wrapper.subscribe("topic-1", &component_id).await.unwrap();

    // Test: Unsubscribe
    let result = wrapper.unsubscribe(handle).await;

    // Verify: Unsubscribe succeeds
    assert!(
        result.is_ok(),
        "Unsubscribe should succeed: {:?}",
        result.err()
    );

    // Verify: Subscription removed
    let subscriptions = wrapper.subscriptions(&component_id).await.unwrap();
    assert_eq!(subscriptions.len(), 0);
}

#[tokio::test]
async fn test_multiple_subscriptions() {
    // Setup
    let broker = InMemoryMessageBroker::new();
    let wrapper = MessageBrokerWrapper::new(broker);
    let component_id = ComponentId::new("test-component");

    // Test: Subscribe to multiple topics
    let _handle1 = wrapper.subscribe("topic-1", &component_id).await.unwrap();
    let _handle2 = wrapper.subscribe("topic-2", &component_id).await.unwrap();
    let _handle3 = wrapper.subscribe("topic-3", &component_id).await.unwrap();

    let subscriptions = wrapper.subscriptions(&component_id).await.unwrap();

    // Verify: All subscriptions tracked
    assert_eq!(subscriptions.len(), 3);
    assert!(subscriptions.contains(&"topic-1".to_string()));
    assert!(subscriptions.contains(&"topic-2".to_string()));
    assert!(subscriptions.contains(&"topic-3".to_string()));
}

#[tokio::test]
async fn test_broker_error_handling() {
    // Setup
    let broker = InMemoryMessageBroker::new();
    let wrapper = MessageBrokerWrapper::new(broker);
    let component_id = ComponentId::new("test-component");

    // Subscribe
    let handle = wrapper.subscribe("topic-1", &component_id).await.unwrap();

    // Test: Unsubscribe twice (second should work but be no-op)
    let result1 = wrapper.unsubscribe(handle.clone()).await;
    let result2 = wrapper.unsubscribe(handle).await;

    // Verify: First unsubscribe succeeds
    assert!(result1.is_ok());

    // Verify: Second unsubscribe fails (already removed)
    assert!(result2.is_err(), "Second unsubscribe should fail");
}

#[tokio::test]
async fn test_publish_without_subscriber() {
    // Setup
    let broker = InMemoryMessageBroker::new();
    let wrapper = MessageBrokerWrapper::new(broker);

    let message = ComponentMessage::InterComponent {
        sender: ComponentId::new("test-sender"),
        to: ComponentId::new("target"), // TODO(WASM-TASK-006): Use actual target
        payload: vec![1, 2, 3],
    };

    // Test: Publish without any subscribers (should succeed - fire-and-forget)
    let result = wrapper.publish("no-subscribers-topic", message).await;

    // Verify: Publish succeeds (fire-and-forget semantics)
    assert!(result.is_ok(), "Publish without subscribers should succeed");
}

#[tokio::test]
async fn test_subscription_handle_uniqueness() {
    // Setup
    let broker = InMemoryMessageBroker::new();
    let wrapper = MessageBrokerWrapper::new(broker);
    let component_id = ComponentId::new("test-component");

    // Test: Subscribe twice to same topic
    let handle1 = wrapper.subscribe("topic-1", &component_id).await.unwrap();
    let handle2 = wrapper.subscribe("topic-1", &component_id).await.unwrap();

    // Verify: Different UUIDs even for same topic
    assert_ne!(handle1.handle_id(), handle2.handle_id());
    assert_ne!(handle1, handle2);
}

#[tokio::test]
async fn test_tracker_concurrent_access() {
    // Setup
    let broker = InMemoryMessageBroker::new();
    let wrapper = Arc::new(MessageBrokerWrapper::new(broker));
    let component_id = ComponentId::new("test-component");

    // Test: Concurrent subscriptions
    let mut handles = vec![];
    for i in 0..10 {
        let wrapper_clone = Arc::clone(&wrapper);
        let component_id_clone = component_id.clone();
        let handle = tokio::spawn(async move {
            wrapper_clone
                .subscribe(&format!("topic-{}", i), &component_id_clone)
                .await
                .unwrap()
        });
        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify: All subscriptions tracked
    let subscriptions = wrapper.subscriptions(&component_id).await.unwrap();
    assert_eq!(subscriptions.len(), 10);
}

#[tokio::test]
async fn test_broker_bridge_type_safety() {
    // Setup
    let broker = InMemoryMessageBroker::new();
    let wrapper = MessageBrokerWrapper::new(broker);

    // Test: Different ComponentMessage types
    let msg1 = ComponentMessage::HealthCheck;
    let msg2 = ComponentMessage::Shutdown;
    let msg3 = ComponentMessage::InterComponent {
        sender: ComponentId::new("sender"),
        to: ComponentId::new("target"), // TODO(WASM-TASK-006): Use actual target
        payload: vec![1, 2, 3],
    };

    // Verify: All message types accepted
    assert!(wrapper.publish("topic-1", msg1).await.is_ok());
    assert!(wrapper.publish("topic-2", msg2).await.is_ok());
    assert!(wrapper.publish("topic-3", msg3).await.is_ok());
}
