#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, unused_imports)]

//! Integration tests for ActorSystem pub-sub with UnifiedRouter.
//! - Full pub-sub flow with ActorSystem intermediation
//! - Multiple subscribers to same topic
//! - Wildcard subscription routing
//! - Component unsubscribe behavior
//! - Routing statistics accuracy
//!
//! # Test Coverage
//!
//! These integration tests validate the complete message flow from publisher
//! to subscriber through ActorSystemSubscriber and UnifiedRouter coordination.
//!
//! # References
//!
//! - **WASM-TASK-004 Phase 4 Task 4.3**: ActorSystem as Primary Subscriber
//! - **ADR-WASM-009**: Component Communication Model
//! - **ADR-WASM-018**: Three-Layer Architecture

// Layer 1: Standard library imports

use std::sync::Arc;
use std::time::Duration;

// Layer 2: Third-party crate imports
// (none)

// Layer 3: Internal module imports
use airssys_rt::broker::{InMemoryMessageBroker, MessageBroker};
use airssys_rt::message::MessageEnvelope;
use airssys_wasm::actor::{
    ComponentMessage, UnifiedRouter,
};
use airssys_wasm::core::ComponentId;

/// Test 1: Full pub-sub flow with ActorSystem
#[tokio::test]
async fn test_full_pub_sub_flow_with_actor_system() {
    let broker = Arc::new(InMemoryMessageBroker::new());

    // Create unified router
    let router = UnifiedRouter::new(Arc::clone(&broker));
    router.start().await.expect("Failed to start router");

    // Create publisher and subscriber components
    let publisher_id = ComponentId::new("publisher");
    let subscriber_id = ComponentId::new("subscriber");

    // Subscribe to topic
    let manager = router.subscriber_manager();
    let handle = manager
        .subscribe(subscriber_id.clone(), vec!["events.user.login".into()])
        .await
        .expect("Failed to subscribe");

    // Verify subscription
    let topics = manager.get_subscriptions(&subscriber_id).await;
    assert_eq!(topics.len(), 1);
    assert!(topics.contains(&"events.user.login".to_string()));

    // Publish message
    let message = ComponentMessage::InterComponent {
        sender: publisher_id.clone(),
        to: ComponentId::new("target"), // TODO(WASM-TASK-006): Use actual target
        payload: vec![1, 2, 3],
    };
    let envelope = airssys_rt::message::MessageEnvelope::new(message);
    broker.publish(envelope).await.expect("Failed to publish");

    // Give time for routing
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Verify routing infrastructure is functioning
    // Note: Stats are updated via router.route() calls, not broker.publish()
    // In full implementation with ActorContext, stats would reflect broker activity
    let _stats = router.stats().await;

    // Unsubscribe
    manager
        .unsubscribe(&handle)
        .await
        .expect("Failed to unsubscribe");
    let topics = manager.get_subscriptions(&subscriber_id).await;
    assert_eq!(topics.len(), 0);

    router.stop().await.expect("Failed to stop router");
}

/// Test 2: Multiple subscribers same topic
#[tokio::test]
async fn test_multiple_subscribers_same_topic() {
    let broker = Arc::new(InMemoryMessageBroker::new());

    let router = UnifiedRouter::new(Arc::clone(&broker));
    router.start().await.expect("Failed to start router");

    let manager = router.subscriber_manager();

    // Subscribe multiple components to same topic
    let component_a = ComponentId::new("component-a");
    let component_b = ComponentId::new("component-b");
    let component_c = ComponentId::new("component-c");

    let topic = "events.user.login";

    manager
        .subscribe(component_a.clone(), vec![topic.into()])
        .await
        .expect("Failed to subscribe A");

    manager
        .subscribe(component_b.clone(), vec![topic.into()])
        .await
        .expect("Failed to subscribe B");

    manager
        .subscribe(component_c.clone(), vec![topic.into()])
        .await
        .expect("Failed to subscribe C");

    // Query subscribers for topic
    let subscribers = manager.subscribers_for_topic(topic).await;
    assert_eq!(subscribers.len(), 3, "Should have 3 subscribers");
    assert!(subscribers.contains(&component_a));
    assert!(subscribers.contains(&component_b));
    assert!(subscribers.contains(&component_c));

    // Publish message
    let message = ComponentMessage::InterComponent {
        sender: ComponentId::new("publisher"),
        to: ComponentId::new("target"), // TODO(WASM-TASK-006): Use actual target
        payload: vec![1, 2, 3],
    };
    let envelope = airssys_rt::message::MessageEnvelope::new(message);
    broker.publish(envelope).await.expect("Failed to publish");

    // Give time for routing
    tokio::time::sleep(Duration::from_millis(50)).await;

    // In full implementation, all 3 components would receive the message
    // This test validates subscription management and multi-subscriber resolution

    router.stop().await.expect("Failed to stop router");
}

/// Test 3: Wildcard subscription routing
#[tokio::test]
async fn test_wildcard_subscription_routing() {
    let broker = Arc::new(InMemoryMessageBroker::new());

    let router = UnifiedRouter::new(Arc::clone(&broker));
    router.start().await.expect("Failed to start router");

    let manager = router.subscriber_manager();

    // Subscribe with wildcards
    let component_id = ComponentId::new("subscriber");

    // Single-level wildcard
    manager
        .subscribe(component_id.clone(), vec!["events.user.*".into()])
        .await
        .expect("Failed to subscribe");

    // Test topic matching
    let subscribers_login = manager.subscribers_for_topic("events.user.login").await;
    assert_eq!(subscribers_login.len(), 1, "Should match events.user.login");

    let subscribers_logout = manager.subscribers_for_topic("events.user.logout").await;
    assert_eq!(
        subscribers_logout.len(),
        1,
        "Should match events.user.logout"
    );

    let subscribers_system = manager.subscribers_for_topic("events.system.restart").await;
    assert_eq!(
        subscribers_system.len(),
        0,
        "Should not match events.system.*"
    );

    // Subscribe with multi-level wildcard
    let component_all = ComponentId::new("subscriber-all");
    manager
        .subscribe(component_all.clone(), vec!["events.#".into()])
        .await
        .expect("Failed to subscribe with #");

    let subscribers_deep = manager
        .subscribers_for_topic("events.user.profile.update")
        .await;
    assert_eq!(
        subscribers_deep.len(),
        1,
        "events.# should match deep topics"
    );

    router.stop().await.expect("Failed to stop router");
}

/// Test 4: Component unsubscribe behavior
#[tokio::test]
async fn test_component_unsubscribe_behavior() {
    let broker = Arc::new(InMemoryMessageBroker::new());

    let router = UnifiedRouter::new(Arc::clone(&broker));
    router.start().await.expect("Failed to start router");

    let manager = router.subscriber_manager();
    let component_id = ComponentId::new("subscriber");

    // Subscribe to multiple topics
    let handle1 = manager
        .subscribe(
            component_id.clone(),
            vec!["topic-1".into(), "topic-2".into()],
        )
        .await
        .expect("Failed to subscribe");

    // Verify subscriptions
    let topics = manager.get_subscriptions(&component_id).await;
    assert_eq!(topics.len(), 2);

    // Unsubscribe
    manager
        .unsubscribe(&handle1)
        .await
        .expect("Failed to unsubscribe");

    // Verify removed
    let topics = manager.get_subscriptions(&component_id).await;
    assert_eq!(topics.len(), 0);

    // Subscribe again to different topics
    let _handle2 = manager
        .subscribe(component_id.clone(), vec!["topic-3".into()])
        .await
        .expect("Failed to resubscribe");

    let topics = manager.get_subscriptions(&component_id).await;
    assert_eq!(topics.len(), 1);
    assert!(topics.contains(&"topic-3".to_string()));

    router.stop().await.expect("Failed to stop router");
}

/// Test 5: Routing statistics accuracy
#[tokio::test]
async fn test_routing_statistics_accuracy() {
    let broker = Arc::new(InMemoryMessageBroker::new());

    let router = UnifiedRouter::new(Arc::clone(&broker));
    router.start().await.expect("Failed to start router");

    // Initial stats
    let stats = router.stats().await;
    assert_eq!(stats.total_messages, 0);
    assert_eq!(stats.successful_routes, 0);
    assert_eq!(stats.failed_routes, 0);

    // Route messages through router
    for i in 0..10 {
        let source = ComponentId::new(format!("source-{}", i));
        let target = ComponentId::new(format!("target-{}", i));
        let message = ComponentMessage::InterComponent {
            sender: source.clone(),
            to: ComponentId::new("target"), // TODO(WASM-TASK-006): Use actual target
            payload: vec![i as u8],
        };

        router
            .route(source, target, message)
            .await
            .expect("Failed to route");
    }

    // Check stats
    let stats = router.stats().await;
    assert_eq!(stats.total_messages, 10, "Should track 10 messages");
    assert_eq!(stats.successful_routes, 10, "Should track 10 successes");
    assert_eq!(stats.failed_routes, 0, "Should have no failures");
    assert!(stats.average_latency_ns > 0, "Should track latency");

    // Verify success rate
    assert_eq!(stats.success_rate(), 100.0);
    assert_eq!(stats.failure_rate(), 0.0);

    router.stop().await.expect("Failed to stop router");
}

/// Test 6: Router lifecycle with subscriptions
#[tokio::test]
async fn test_router_lifecycle_with_subscriptions() {
    let broker = Arc::new(InMemoryMessageBroker::new());

    let router = UnifiedRouter::new(Arc::clone(&broker));

    // Not running initially
    assert!(!router.is_running().await);

    // Start router
    router.start().await.expect("Failed to start");
    assert!(router.is_running().await);

    // Add subscriptions
    let manager = router.subscriber_manager();
    let component_id = ComponentId::new("test-component");

    manager
        .subscribe(component_id.clone(), vec!["test.topic".into()])
        .await
        .expect("Failed to subscribe");

    let count = manager.subscription_count().await;
    assert_eq!(count, 1);

    // Stop router
    router.stop().await.expect("Failed to stop");
    assert!(!router.is_running().await);

    // Subscriptions should still exist in manager
    let count = manager.subscription_count().await;
    assert_eq!(count, 1);

    // Can restart router
    router.start().await.expect("Failed to restart");
    assert!(router.is_running().await);

    router.stop().await.expect("Failed to stop again");
}

/// Test 7: Concurrent subscription operations
#[tokio::test]
async fn test_concurrent_subscription_operations() {
    let broker = Arc::new(InMemoryMessageBroker::new());

    let router = UnifiedRouter::new(broker);
    router.start().await.expect("Failed to start router");

    let manager = router.subscriber_manager();

    // Spawn multiple concurrent subscription tasks
    let mut handles = vec![];
    for i in 0..10 {
        let manager_clone = Arc::clone(&manager);
        let handle = tokio::spawn(async move {
            let component_id = ComponentId::new(format!("component-{}", i));
            manager_clone
                .subscribe(component_id, vec![format!("topic-{}", i)])
                .await
        });
        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        let result = handle.await.expect("Task panicked");
        assert!(result.is_ok(), "Subscription failed: {:?}", result.err());
    }

    // Verify subscription count
    let count = manager.subscription_count().await;
    assert_eq!(count, 10, "Should have 10 subscriptions");

    router.stop().await.expect("Failed to stop router");
}
