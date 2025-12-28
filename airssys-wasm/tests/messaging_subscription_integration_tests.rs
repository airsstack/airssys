#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

//! Integration tests for MessagingSubscriptionService (WASM-TASK-006 Task 1.3).

//!
//! This test suite validates the ActorSystem Event Subscription Infrastructure:
//! - Service initialization and subscription to MessageBroker
//! - Component registration during spawn
//! - Full stack message routing (broker → subscriber → mailbox)
//! - Graceful shutdown during message flow
//! - Concurrent registrations stress test
//! - Integration with Task 1.1 and 1.2 implementations
//!
//! # Architecture Context (ADR-WASM-020)
//!
//! These tests prove that `MessagingSubscriptionService` correctly:
//! - Coordinates ActorSystemSubscriber lifecycle
//! - Enables component registration for message delivery
//! - Provides address resolution via ComponentRegistry
//! - Handles errors gracefully without crashing
//!
//! # Test Coverage
//!
//! 1. End-to-end subscription initialization
//! 2. Component registration with subscription service
//! 3. Full stack message routing (broker → mailbox)
//! 4. Graceful shutdown during message flow
//! 5. Component unregistration during active messaging
//! 6. Concurrent registrations stress test
//! 7. Integration with Task 1.1/1.2 implementations
//!
//! # References
//!
//! - **ADR-WASM-020**: Message Delivery Ownership Architecture
//! - **KNOWLEDGE-WASM-024**: Component Messaging Clarifications (Phase 1 scope)
//! - **KNOWLEDGE-WASM-026**: Message Delivery Architecture - Final Decision
//! - **WASM-TASK-006 Task 1.3**: ActorSystem Event Subscription Infrastructure

// Test code: expect is acceptable
// Test code: unwrap is acceptable
// Test code: panic is acceptable for assertion failures

// Layer 1: Standard library imports
use std::sync::Arc;
use std::time::Duration;

// Layer 2: Third-party crate imports
use tokio::sync::mpsc;
use tokio::time::timeout;

// Layer 3: Internal module imports
use airssys_rt::broker::{InMemoryMessageBroker, MessageBroker};
use airssys_rt::message::MessageEnvelope;
use airssys_rt::util::ActorAddress;
use airssys_wasm::actor::{
    ComponentMessage, ComponentRegistry, MessagingSubscriptionService, SubscriberManager,
};
use airssys_wasm::core::ComponentId;

// ============================================================================
// Test 1: End-to-End Subscription Initialization
// ============================================================================

/// Proves MessagingSubscriptionService correctly initializes and subscribes to broker.
#[tokio::test]
async fn test_subscription_service_initialization() {
    // Setup
    let broker = Arc::new(InMemoryMessageBroker::new());
    let registry = ComponentRegistry::new();
    let subscriber_manager = Arc::new(SubscriberManager::new());

    // Create subscription service
    let service =
        MessagingSubscriptionService::new(Arc::clone(&broker), registry, subscriber_manager);

    // Verify initial state
    let status = service.status().await;
    assert!(
        !status.is_running,
        "Service should not be running initially"
    );
    assert_eq!(status.registered_components, 0);

    // Start service
    let result = service.start().await;
    assert!(
        result.is_ok(),
        "Service start should succeed: {:?}",
        result.err()
    );

    // Verify running state
    let status = service.status().await;
    assert!(status.is_running, "Service should be running after start");

    // Verify broker has subscriber
    let subscriber_count = broker.subscriber_count().await;
    assert!(
        subscriber_count >= 1,
        "Broker should have at least one subscriber after service start"
    );

    // Cleanup
    service.stop().await.expect("Service stop should succeed");

    // Verify stopped state
    let status = service.status().await;
    assert!(
        !status.is_running,
        "Service should not be running after stop"
    );
}

// ============================================================================
// Test 2: Component Registration During Spawn
// ============================================================================

/// Proves component registration integrates with subscription service.
#[tokio::test]
async fn test_component_registration_with_subscription_service() {
    // Setup
    let broker = Arc::new(InMemoryMessageBroker::new());
    let registry = ComponentRegistry::new();
    let subscriber_manager = Arc::new(SubscriberManager::new());

    let service = MessagingSubscriptionService::new(
        Arc::clone(&broker),
        registry.clone(),
        subscriber_manager,
    );

    // Start service
    service.start().await.expect("Service start should succeed");

    // Register component (simulates ComponentSpawner)
    let (tx, mut rx) = mpsc::unbounded_channel::<ComponentMessage>();
    let component_id = ComponentId::new("test-component");

    let result = service.register_component(component_id.clone(), tx).await;
    assert!(
        result.is_ok(),
        "Component registration should succeed: {:?}",
        result.err()
    );

    // Verify registered count
    assert_eq!(
        service.registered_component_count().await,
        1,
        "Should have 1 registered component"
    );

    // Publish message targeting the component
    let message = ComponentMessage::InterComponent {
        sender: ComponentId::new("sender"),
        to: component_id.clone(),
        payload: vec![1, 2, 3, 4, 5],
    };
    broker
        .publish(MessageEnvelope::new(message))
        .await
        .expect("Publish should succeed");

    // Verify message arrives in component's mailbox
    let receive_result = timeout(Duration::from_millis(500), rx.recv()).await;
    match receive_result {
        Ok(Some(ComponentMessage::InterComponent {
            sender,
            to,
            payload,
        })) => {
            assert_eq!(sender.as_str(), "sender");
            assert_eq!(to.as_str(), "test-component");
            assert_eq!(payload, vec![1, 2, 3, 4, 5]);
        }
        Ok(Some(_)) => panic!("Wrong message type received"),
        Ok(None) => panic!("Channel closed - message was NOT delivered"),
        Err(_) => panic!("TIMEOUT - message was NOT delivered within 500ms"),
    }

    // Cleanup
    service.stop().await.expect("Service stop should succeed");
}

// ============================================================================
// Test 3: Full Stack Message Routing
// ============================================================================

/// End-to-end: broker publish → subscriber → mailbox → verification
#[tokio::test]
async fn test_full_stack_message_routing() {
    // Setup
    let broker = Arc::new(InMemoryMessageBroker::new());
    let registry = ComponentRegistry::new();
    let subscriber_manager = Arc::new(SubscriberManager::new());

    let service = MessagingSubscriptionService::new(
        Arc::clone(&broker),
        registry.clone(),
        subscriber_manager,
    );

    service.start().await.expect("Service start should succeed");

    // Register 3 components
    let (tx_a, mut rx_a) = mpsc::unbounded_channel::<ComponentMessage>();
    let (tx_b, mut rx_b) = mpsc::unbounded_channel::<ComponentMessage>();
    let (tx_c, mut rx_c) = mpsc::unbounded_channel::<ComponentMessage>();

    let comp_a = ComponentId::new("component-a");
    let comp_b = ComponentId::new("component-b");
    let comp_c = ComponentId::new("component-c");

    service
        .register_component(comp_a.clone(), tx_a)
        .await
        .unwrap();
    service
        .register_component(comp_b.clone(), tx_b)
        .await
        .unwrap();
    service
        .register_component(comp_c.clone(), tx_c)
        .await
        .unwrap();

    assert_eq!(service.registered_component_count().await, 3);

    // Publish message targeting B
    let message_to_b = ComponentMessage::InterComponent {
        sender: ComponentId::new("external-sender"),
        to: comp_b.clone(),
        payload: vec![66], // 'B'
    };
    broker
        .publish(MessageEnvelope::new(message_to_b))
        .await
        .unwrap();

    // Verify ONLY B receives the message
    let recv_b = timeout(Duration::from_millis(500), rx_b.recv()).await;
    match recv_b {
        Ok(Some(ComponentMessage::InterComponent { payload, .. })) => {
            assert_eq!(payload, vec![66], "Component B should receive payload [66]");
        }
        Ok(Some(_)) => panic!("Wrong message type received by B"),
        Ok(None) => panic!("Channel B closed"),
        Err(_) => panic!("TIMEOUT waiting for B to receive message"),
    }

    // A and C should NOT receive the message (non-blocking check)
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(
        rx_a.try_recv().is_err(),
        "Component A should NOT receive message for B"
    );
    assert!(
        rx_c.try_recv().is_err(),
        "Component C should NOT receive message for B"
    );

    // Cleanup
    service.stop().await.expect("Service stop should succeed");
}

// ============================================================================
// Test 4: Graceful Shutdown During Message Flow
// ============================================================================

/// Proves shutdown doesn't lose in-flight messages (or handles gracefully).
#[tokio::test]
async fn test_graceful_shutdown_during_message_flow() {
    // Setup
    let broker = Arc::new(InMemoryMessageBroker::new());
    let registry = ComponentRegistry::new();
    let subscriber_manager = Arc::new(SubscriberManager::new());

    let service = MessagingSubscriptionService::new(
        Arc::clone(&broker),
        registry.clone(),
        subscriber_manager,
    );

    service.start().await.expect("Service start should succeed");

    // Register component
    let (tx, _rx) = mpsc::unbounded_channel::<ComponentMessage>();
    let component_id = ComponentId::new("shutdown-test");
    service
        .register_component(component_id.clone(), tx)
        .await
        .unwrap();

    // Start publishing messages
    for i in 0..10u8 {
        let message = ComponentMessage::InterComponent {
            sender: ComponentId::new("sender"),
            to: component_id.clone(),
            payload: vec![i],
        };
        broker.publish(MessageEnvelope::new(message)).await.unwrap();
    }

    // Call stop() during message flow
    let stop_result = service.stop().await;

    // Stop should succeed (no panics)
    assert!(
        stop_result.is_ok(),
        "Graceful shutdown should succeed: {:?}",
        stop_result.err()
    );

    // Service should be stopped
    let status = service.status().await;
    assert!(!status.is_running, "Service should be stopped after stop()");
}

// ============================================================================
// Test 5: Component Unregistration During Active Messaging
// ============================================================================

/// Proves unregistration is handled gracefully during active messaging.
#[tokio::test]
async fn test_unregister_during_active_messaging() {
    // Setup
    let broker = Arc::new(InMemoryMessageBroker::new());
    let registry = ComponentRegistry::new();
    let subscriber_manager = Arc::new(SubscriberManager::new());

    let service = MessagingSubscriptionService::new(
        Arc::clone(&broker),
        registry.clone(),
        subscriber_manager,
    );

    service.start().await.expect("Service start should succeed");

    // Register component
    let (tx, mut rx) = mpsc::unbounded_channel::<ComponentMessage>();
    let component_id = ComponentId::new("unregister-active-test");
    service
        .register_component(component_id.clone(), tx)
        .await
        .unwrap();

    // Publish first message
    let message1 = ComponentMessage::InterComponent {
        sender: ComponentId::new("sender"),
        to: component_id.clone(),
        payload: vec![1],
    };
    broker
        .publish(MessageEnvelope::new(message1))
        .await
        .unwrap();

    // Wait for message to be delivered
    let recv = timeout(Duration::from_millis(500), rx.recv()).await;
    assert!(recv.is_ok(), "First message should be delivered");

    // Unregister component
    service.unregister_component(&component_id).await.unwrap();
    assert_eq!(service.registered_component_count().await, 0);

    // Publish second message (should be dropped gracefully, not crash)
    let message2 = ComponentMessage::InterComponent {
        sender: ComponentId::new("sender"),
        to: component_id.clone(),
        payload: vec![2],
    };
    broker
        .publish(MessageEnvelope::new(message2))
        .await
        .unwrap();

    // Give time for routing to process (error logged, not crashed)
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Service should still be running
    let status = service.status().await;
    assert!(
        status.is_running,
        "Service should still be running after unregister"
    );

    // Cleanup
    service.stop().await.expect("Service stop should succeed");
}

// ============================================================================
// Test 6: Concurrent Registrations Stress Test
// ============================================================================

/// Stress test: concurrent registrations don't cause race conditions.
#[tokio::test]
async fn test_concurrent_registrations() {
    // Setup
    let broker = Arc::new(InMemoryMessageBroker::new());
    let registry = ComponentRegistry::new();
    let subscriber_manager = Arc::new(SubscriberManager::new());

    let service = Arc::new(MessagingSubscriptionService::new(
        Arc::clone(&broker),
        registry.clone(),
        subscriber_manager,
    ));

    service.start().await.expect("Service start should succeed");

    // Spawn 20 tasks, each registering a component
    let mut handles = vec![];
    let mut receivers = vec![];

    for i in 0..20 {
        let service_clone = Arc::clone(&service);
        let component_id = ComponentId::new(format!("concurrent-{}", i));
        let (tx, rx) = mpsc::unbounded_channel::<ComponentMessage>();
        receivers.push((component_id.clone(), rx));

        let handle = tokio::spawn(async move {
            // Small random-ish delay to stagger registrations
            tokio::time::sleep(Duration::from_micros((i * 100) as u64)).await;
            service_clone.register_component(component_id, tx).await
        });
        handles.push(handle);
    }

    // Wait for all registrations
    for handle in handles {
        let result = handle.await.expect("Task panicked");
        assert!(
            result.is_ok(),
            "Registration should succeed: {:?}",
            result.err()
        );
    }

    // Allow time for registrations to settle
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Verify all 20 registered
    assert_eq!(
        service.registered_component_count().await,
        20,
        "Should have 20 registered components"
    );

    // Publish message to each component
    for i in 0..20 {
        let target = ComponentId::new(format!("concurrent-{}", i));
        let message = ComponentMessage::InterComponent {
            sender: ComponentId::new("test-sender"),
            to: target,
            payload: vec![i as u8],
        };
        broker.publish(MessageEnvelope::new(message)).await.unwrap();
    }

    // Give time for delivery
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Verify each component received its message
    for (component_id, mut rx) in receivers {
        let received = rx.try_recv();
        assert!(
            received.is_ok(),
            "Component {} did not receive message",
            component_id.as_str()
        );

        // Verify correct payload
        if let Ok(ComponentMessage::InterComponent { payload, .. }) = received {
            let expected_payload = component_id
                .as_str()
                .strip_prefix("concurrent-")
                .and_then(|s| s.parse::<u8>().ok())
                .unwrap_or(255);
            assert_eq!(
                payload,
                vec![expected_payload],
                "Component {} got wrong payload",
                component_id.as_str()
            );
        }
    }

    // Cleanup
    service.stop().await.expect("Service stop should succeed");
}

// ============================================================================
// Test 7: Integration with Task 1.1/1.2 Implementation
// ============================================================================

/// Proves Task 1.3 infrastructure works with Task 1.1 & 1.2 implementation.
#[tokio::test]
async fn test_integration_with_message_delivery_and_reception() {
    // Setup - using MessagingSubscriptionService as the coordinator
    let broker = Arc::new(InMemoryMessageBroker::new());
    let registry = ComponentRegistry::new();
    let subscriber_manager = Arc::new(SubscriberManager::new());

    // Also register in registry (simulating full spawn flow)
    let component_id = ComponentId::new("full-integration-component");
    let actor_addr = ActorAddress::named("full-integration-component");
    registry
        .register(component_id.clone(), actor_addr.clone())
        .unwrap();

    let service = MessagingSubscriptionService::new(
        Arc::clone(&broker),
        registry.clone(),
        Arc::clone(&subscriber_manager),
    );

    // Start subscription service
    service.start().await.expect("Service start should succeed");

    // Register mailbox (Task 1.1: ActorSystemSubscriber's mailbox_senders)
    let (tx, mut rx) = mpsc::unbounded_channel::<ComponentMessage>();
    service
        .register_component(component_id.clone(), tx)
        .await
        .unwrap();

    // Verify address resolution (Task 1.3: ComponentRegistry integration)
    let resolved = service.resolve_address(&component_id);
    assert!(resolved.is_some(), "Address resolution should work");
    assert_eq!(resolved.unwrap(), actor_addr);

    // Test correlation message (Task 1.2: message with correlation ID)
    let correlation_id = uuid::Uuid::new_v4();
    let message = ComponentMessage::InterComponentWithCorrelation {
        sender: ComponentId::new("requester"),
        to: component_id.clone(),
        payload: vec![10, 20, 30],
        correlation_id,
    };
    broker.publish(MessageEnvelope::new(message)).await.unwrap();

    // Verify message reception (proves Task 1.1 delivery + Task 1.2 reception ready)
    let recv = timeout(Duration::from_millis(500), rx.recv()).await;
    match recv {
        Ok(Some(ComponentMessage::InterComponentWithCorrelation {
            sender,
            to,
            payload,
            correlation_id: recv_id,
        })) => {
            assert_eq!(sender.as_str(), "requester");
            assert_eq!(to.as_str(), "full-integration-component");
            assert_eq!(payload, vec![10, 20, 30]);
            assert_eq!(
                recv_id, correlation_id,
                "Correlation ID should be preserved"
            );
        }
        Ok(Some(_)) => panic!("Wrong message type received"),
        Ok(None) => panic!("Channel closed"),
        Err(_) => panic!("TIMEOUT - integration test failed"),
    }

    // Verify status reflects activity
    let status = service.status().await;
    assert!(status.is_running);
    assert_eq!(status.registered_components, 1);

    // Cleanup
    service.stop().await.expect("Service stop should succeed");
}

// ============================================================================
// Test 8: Broker Accessor Returns Same Instance
// ============================================================================

/// Verifies broker accessor returns the same broker instance.
#[tokio::test]
async fn test_broker_accessor_returns_same_instance() {
    let broker = Arc::new(InMemoryMessageBroker::new());
    let registry = ComponentRegistry::new();
    let subscriber_manager = Arc::new(SubscriberManager::new());

    let service =
        MessagingSubscriptionService::new(Arc::clone(&broker), registry, subscriber_manager);

    let retrieved_broker = service.broker();

    // Should be the same broker
    assert!(
        Arc::ptr_eq(&broker, &retrieved_broker),
        "Broker accessor should return same instance"
    );
}

// ============================================================================
// Test 9: Registry Accessor Works Correctly
// ============================================================================

/// Verifies registry accessor provides working registry.
#[tokio::test]
async fn test_registry_accessor_works() {
    let broker = Arc::new(InMemoryMessageBroker::new());
    let registry = ComponentRegistry::new();
    let subscriber_manager = Arc::new(SubscriberManager::new());

    // Pre-register something
    let component_id = ComponentId::new("registry-accessor-test");
    let actor_addr = ActorAddress::named("registry-accessor-test");
    registry
        .register(component_id.clone(), actor_addr.clone())
        .unwrap();

    let service = MessagingSubscriptionService::new(broker, registry, subscriber_manager);

    // Get registry and verify lookup works
    let retrieved_registry = service.registry();
    let lookup = retrieved_registry.lookup(&component_id);
    assert!(lookup.is_ok(), "Lookup should succeed");
    assert_eq!(lookup.unwrap(), actor_addr);
}

// ============================================================================
// Test 10: Status Reflects Accurate State
// ============================================================================

/// Verifies status reporting is accurate at each lifecycle stage.
#[tokio::test]
async fn test_status_accuracy_through_lifecycle() {
    let broker = Arc::new(InMemoryMessageBroker::new());
    let registry = ComponentRegistry::new();
    let subscriber_manager = Arc::new(SubscriberManager::new());

    let service =
        MessagingSubscriptionService::new(Arc::clone(&broker), registry, subscriber_manager);

    // Stage 1: Before start
    let status = service.status().await;
    assert!(!status.is_running, "Stage 1: Should not be running");
    assert_eq!(
        status.registered_components, 0,
        "Stage 1: Should have 0 components"
    );

    // Stage 2: After start
    service.start().await.unwrap();
    let status = service.status().await;
    assert!(status.is_running, "Stage 2: Should be running");

    // Stage 3: After registering components
    for i in 0..3 {
        let (tx, _rx) = mpsc::unbounded_channel();
        service
            .register_component(ComponentId::new(format!("status-test-{}", i)), tx)
            .await
            .unwrap();
    }
    let status = service.status().await;
    assert_eq!(
        status.registered_components, 3,
        "Stage 3: Should have 3 components"
    );

    // Stage 4: After unregistering one
    service
        .unregister_component(&ComponentId::new("status-test-1"))
        .await
        .unwrap();
    let status = service.status().await;
    assert_eq!(
        status.registered_components, 2,
        "Stage 4: Should have 2 components after unregister"
    );

    // Stage 5: After stop
    service.stop().await.unwrap();
    let status = service.status().await;
    assert!(
        !status.is_running,
        "Stage 5: Should not be running after stop"
    );

    // Note: registered_components count may still be non-zero after stop
    // as unregistration isn't automatic - this is by design
}
