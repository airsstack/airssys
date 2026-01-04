#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

//! Integration tests for message delivery to component mailboxes.
//! - `ActorSystemSubscriber` ACTUALLY DELIVERS messages to mailboxes
//! - End-to-end message flow from broker → subscriber → component mailbox
//! - Error handling for unregistered components
//! - Concurrent registration and delivery
//!
//! # Architecture Context (ADR-WASM-020)
//!
//! These tests prove that the previously STUBBED `route_message_to_subscribers()`
//! now performs ACTUAL message delivery via the `mailbox_senders` map.
//!
//! # Test Coverage
//!
//! 1. End-to-end message delivery (broker publish → mailbox receive)
//! 2. Multiple messages delivered in order
//! 3. Unregistered component handling (graceful error)
//! 4. Concurrent registration and delivery
//! 5. Message delivery with correlation ID
//! 6. Multiple components receiving independent messages
//!
//! # References
//!
//! - **ADR-WASM-020**: Message Delivery Ownership Architecture
//! - **KNOWLEDGE-WASM-026**: Message Delivery Architecture - Final Decision
//! - **WASM-TASK-006 Task 1.1 Remediation**: Fix stubbed message routing
//! - **task-006-phase-1-task-1.1-remediation-plan.md**: Implementation plan

// Layer 1: Standard library imports
use std::sync::Arc;
use std::time::Duration;

// Layer 2: Third-party crate imports
use tokio::sync::mpsc;
use tokio::time::timeout;

// Layer 3: Internal module imports
use airssys_rt::broker::{InMemoryMessageBroker, MessageBroker};
use airssys_rt::message::MessageEnvelope;
use airssys_wasm::actor::{ActorSystemSubscriber, ComponentMessage, SubscriberManager};
use airssys_wasm::core::ComponentId;

// ============================================================================
// Test 1: End-to-End Message Delivery
// ============================================================================

/// CRITICAL TEST: Proves a message published to broker arrives in target component's mailbox.
///
/// This is THE test that was missing - it verifies that `route_message_to_subscribers()`
/// now performs ACTUAL delivery instead of being STUBBED.
#[tokio::test]
async fn test_end_to_end_message_delivery() {
    // Setup
    let broker = Arc::new(InMemoryMessageBroker::new());
    let subscriber_manager = Arc::new(SubscriberManager::new());

    // Step 1: Create ActorSystemSubscriber
    let mut subscriber = ActorSystemSubscriber::new(Arc::clone(&broker), subscriber_manager);

    // Step 2: Create channel to receive messages (simulates component mailbox)
    let (tx, mut rx) = mpsc::unbounded_channel::<ComponentMessage>();

    // Step 3: Register mailbox sender with ActorSystemSubscriber
    let target_id = ComponentId::new("target-component");
    subscriber
        .register_mailbox(target_id.clone(), tx)
        .await
        .expect("Failed to register mailbox");

    // Step 4: Start subscriber (spawns routing task)
    subscriber
        .start()
        .await
        .expect("Failed to start subscriber");

    // Step 5: Publish message to broker
    let message = ComponentMessage::InterComponent {
        sender: ComponentId::new("sender"),
        to: target_id.clone(),
        payload: vec![1, 2, 3, 4, 5],
    };
    let envelope = MessageEnvelope::new(message.clone());
    broker.publish(envelope).await.expect("Failed to publish");

    // Step 6: CRITICAL - Verify message ACTUALLY arrives in mailbox
    let receive_result = timeout(Duration::from_millis(500), rx.recv()).await;

    match receive_result {
        Ok(Some(received_message)) => {
            // SUCCESS: Message was delivered!
            match received_message {
                ComponentMessage::InterComponent {
                    sender,
                    to,
                    payload,
                } => {
                    assert_eq!(sender.as_str(), "sender");
                    assert_eq!(to.as_str(), "target-component");
                    assert_eq!(payload, vec![1, 2, 3, 4, 5]);
                }
                _ => panic!("Wrong message type received"),
            }
        }
        Ok(None) => {
            panic!("Channel closed - message was NOT delivered");
        }
        Err(_) => {
            panic!("TIMEOUT - message was NOT delivered within 500ms");
        }
    }

    // Cleanup
    subscriber.stop().await.expect("Failed to stop subscriber");
}

// ============================================================================
// Test 2: Multiple Messages Delivered In Order
// ============================================================================

/// Tests that multiple messages are delivered to the mailbox in the order they were published.
#[tokio::test]
async fn test_multiple_messages_delivered_in_order() {
    let broker = Arc::new(InMemoryMessageBroker::new());
    let subscriber_manager = Arc::new(SubscriberManager::new());

    let mut subscriber = ActorSystemSubscriber::new(Arc::clone(&broker), subscriber_manager);

    let (tx, mut rx) = mpsc::unbounded_channel::<ComponentMessage>();
    let target_id = ComponentId::new("ordered-target");

    subscriber
        .register_mailbox(target_id.clone(), tx)
        .await
        .expect("Failed to register mailbox");
    subscriber.start().await.expect("Failed to start");

    // Publish 5 messages with sequential payloads
    for i in 0u8..5 {
        let message = ComponentMessage::InterComponent {
            sender: ComponentId::new("sender"),
            to: target_id.clone(),
            payload: vec![i],
        };
        broker
            .publish(MessageEnvelope::new(message))
            .await
            .expect("Failed to publish");
    }

    // Receive all 5 messages in order
    for i in 0u8..5 {
        let received = timeout(Duration::from_millis(500), rx.recv())
            .await
            .expect("timeout waiting for message")
            .expect("channel closed");

        match received {
            ComponentMessage::InterComponent { payload, .. } => {
                assert_eq!(
                    payload,
                    vec![i],
                    "Messages arrived out of order at index {}",
                    i
                );
            }
            _ => panic!("Wrong message type at index {}", i),
        }
    }

    subscriber.stop().await.expect("Failed to stop");
}

// ============================================================================
// Test 3: Message to Unregistered Component Handled Gracefully
// ============================================================================

/// Tests that sending to an unregistered component logs an error but doesn't crash.
#[tokio::test]
async fn test_message_to_unregistered_component_handled_gracefully() {
    let broker = Arc::new(InMemoryMessageBroker::new());
    let subscriber_manager = Arc::new(SubscriberManager::new());

    let mut subscriber = ActorSystemSubscriber::new(Arc::clone(&broker), subscriber_manager);

    // Do NOT register any mailbox
    subscriber.start().await.expect("Failed to start");

    // Publish message to non-existent component
    let message = ComponentMessage::InterComponent {
        sender: ComponentId::new("sender"),
        to: ComponentId::new("non-existent"),
        payload: vec![1, 2, 3],
    };
    broker
        .publish(MessageEnvelope::new(message))
        .await
        .expect("Failed to publish");

    // Give time for processing (error should be logged, not crashed)
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Subscriber should still be running (error logged, not crashed)
    assert!(
        subscriber.is_running(),
        "Subscriber crashed when routing to unregistered component"
    );

    subscriber.stop().await.expect("Failed to stop");
}

// ============================================================================
// Test 4: Concurrent Registration and Delivery
// ============================================================================

/// Tests that registration and message delivery work correctly under concurrent access.
#[tokio::test]
async fn test_concurrent_registration_and_delivery() {
    let broker = Arc::new(InMemoryMessageBroker::new());
    let subscriber_manager = Arc::new(SubscriberManager::new());

    let subscriber = Arc::new(tokio::sync::RwLock::new(ActorSystemSubscriber::new(
        Arc::clone(&broker),
        subscriber_manager,
    )));

    // Start subscriber
    {
        let mut sub = subscriber.write().await;
        sub.start().await.expect("Failed to start");
    }

    // Spawn tasks to register components concurrently
    let mut handles = vec![];
    let mut receivers = vec![];

    for i in 0..5 {
        let subscriber_clone = Arc::clone(&subscriber);
        let component_id = ComponentId::new(format!("component-{}", i));
        let (tx, rx) = mpsc::unbounded_channel::<ComponentMessage>();
        receivers.push((component_id.clone(), rx));

        let handle = tokio::spawn(async move {
            // Small delay to stagger registrations
            tokio::time::sleep(Duration::from_millis(i as u64 * 10)).await;

            let sub = subscriber_clone.read().await;
            sub.register_mailbox(component_id, tx).await
        });
        handles.push(handle);
    }

    // Wait for all registrations
    for handle in handles {
        handle
            .await
            .expect("Task panicked")
            .expect("Registration failed");
    }

    // Allow time for registrations to settle
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Publish messages to each component
    for i in 0..5 {
        let target = ComponentId::new(format!("component-{}", i));
        let message = ComponentMessage::InterComponent {
            sender: ComponentId::new("test-sender"),
            to: target,
            payload: vec![i as u8],
        };
        broker
            .publish(MessageEnvelope::new(message))
            .await
            .expect("Failed to publish");
    }

    // Give time for delivery
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify each component received its message
    for (component_id, mut rx) in receivers {
        let received = rx.try_recv();
        assert!(
            received.is_ok(),
            "Component {} did not receive message",
            component_id.as_str()
        );
    }

    // Cleanup
    {
        let mut sub = subscriber.write().await;
        sub.stop().await.expect("Failed to stop");
    }
}

// ============================================================================
// Test 5: Message Delivery With Correlation ID
// ============================================================================

/// Tests that messages with correlation IDs are delivered correctly.
#[tokio::test]
async fn test_message_delivery_with_correlation_id() {
    let broker = Arc::new(InMemoryMessageBroker::new());
    let subscriber_manager = Arc::new(SubscriberManager::new());

    let mut subscriber = ActorSystemSubscriber::new(Arc::clone(&broker), subscriber_manager);

    let (tx, mut rx) = mpsc::unbounded_channel::<ComponentMessage>();
    let target_id = ComponentId::new("correlation-target");
    let correlation_id = uuid::Uuid::new_v4();

    subscriber
        .register_mailbox(target_id.clone(), tx)
        .await
        .expect("Failed to register mailbox");
    subscriber.start().await.expect("Failed to start");

    // Publish message with correlation ID
    let message = ComponentMessage::InterComponentWithCorrelation {
        sender: ComponentId::new("requester"),
        to: target_id.clone(),
        payload: vec![10, 20, 30],
        correlation_id,
    };
    broker
        .publish(MessageEnvelope::new(message))
        .await
        .expect("Failed to publish");

    // Receive and verify correlation ID
    let received = timeout(Duration::from_millis(500), rx.recv())
        .await
        .expect("timeout")
        .expect("channel closed");

    match received {
        ComponentMessage::InterComponentWithCorrelation {
            sender,
            to,
            payload,
            correlation_id: recv_id,
        } => {
            assert_eq!(sender.as_str(), "requester");
            assert_eq!(to.as_str(), "correlation-target");
            assert_eq!(payload, vec![10, 20, 30]);
            assert_eq!(recv_id, correlation_id, "Correlation ID mismatch");
        }
        _ => panic!("Wrong message type received"),
    }

    subscriber.stop().await.expect("Failed to stop");
}

// ============================================================================
// Test 6: Multiple Components Receiving Independent Messages
// ============================================================================

/// Tests that multiple components each receive only their own messages.
#[tokio::test]
async fn test_multiple_components_independent_messages() {
    let broker = Arc::new(InMemoryMessageBroker::new());
    let subscriber_manager = Arc::new(SubscriberManager::new());

    let subscriber = ActorSystemSubscriber::new(Arc::clone(&broker), subscriber_manager);

    // Register 3 components
    let (tx_a, rx_a) = mpsc::unbounded_channel::<ComponentMessage>();
    let (tx_b, rx_b) = mpsc::unbounded_channel::<ComponentMessage>();
    let (tx_c, rx_c) = mpsc::unbounded_channel::<ComponentMessage>();

    let comp_a = ComponentId::new("component-a");
    let comp_b = ComponentId::new("component-b");
    let comp_c = ComponentId::new("component-c");

    subscriber
        .register_mailbox(comp_a.clone(), tx_a)
        .await
        .unwrap();
    subscriber
        .register_mailbox(comp_b.clone(), tx_b)
        .await
        .unwrap();
    subscriber
        .register_mailbox(comp_c.clone(), tx_c)
        .await
        .unwrap();

    // Note: subscriber.start() requires &mut self, so we need a mutable binding
    // For this test, we'll directly test the routing function

    // Instead, let's test using direct function calls to route_message_to_subscribers
    // This is testing the routing logic, not the background task

    use std::collections::HashMap;
    use tokio::sync::RwLock;

    let mailbox_senders: RwLock<HashMap<ComponentId, mpsc::UnboundedSender<ComponentMessage>>> =
        RwLock::new(HashMap::new());

    // Re-register in the test-local map
    let (tx_a2, mut rx_a2) = mpsc::unbounded_channel::<ComponentMessage>();
    let (tx_b2, mut rx_b2) = mpsc::unbounded_channel::<ComponentMessage>();
    let (tx_c2, mut rx_c2) = mpsc::unbounded_channel::<ComponentMessage>();

    {
        let mut senders = mailbox_senders.write().await;
        senders.insert(comp_a.clone(), tx_a2);
        senders.insert(comp_b.clone(), tx_b2);
        senders.insert(comp_c.clone(), tx_c2);
    }

    let sub_mgr = Arc::new(SubscriberManager::new());

    // Send message to component A
    let msg_a = ComponentMessage::InterComponent {
        sender: ComponentId::new("sender"),
        to: comp_a.clone(),
        payload: vec![65], // 'A'
    };
    ActorSystemSubscriber::<InMemoryMessageBroker<ComponentMessage>>::route_message_to_subscribers(
        &mailbox_senders,
        &sub_mgr,
        MessageEnvelope::new(msg_a),
    )
    .await
    .unwrap();

    // Send message to component B
    let msg_b = ComponentMessage::InterComponent {
        sender: ComponentId::new("sender"),
        to: comp_b.clone(),
        payload: vec![66], // 'B'
    };
    ActorSystemSubscriber::<InMemoryMessageBroker<ComponentMessage>>::route_message_to_subscribers(
        &mailbox_senders,
        &sub_mgr,
        MessageEnvelope::new(msg_b),
    )
    .await
    .unwrap();

    // Send message to component C
    let msg_c = ComponentMessage::InterComponent {
        sender: ComponentId::new("sender"),
        to: comp_c.clone(),
        payload: vec![67], // 'C'
    };
    ActorSystemSubscriber::<InMemoryMessageBroker<ComponentMessage>>::route_message_to_subscribers(
        &mailbox_senders,
        &sub_mgr,
        MessageEnvelope::new(msg_c),
    )
    .await
    .unwrap();

    // Verify each component got only its message
    let recv_a = rx_a2.try_recv().unwrap();
    if let ComponentMessage::InterComponent { payload, .. } = recv_a {
        assert_eq!(payload, vec![65], "Component A got wrong message");
    }
    assert!(rx_a2.try_recv().is_err(), "Component A got extra messages");

    let recv_b = rx_b2.try_recv().unwrap();
    if let ComponentMessage::InterComponent { payload, .. } = recv_b {
        assert_eq!(payload, vec![66], "Component B got wrong message");
    }
    assert!(rx_b2.try_recv().is_err(), "Component B got extra messages");

    let recv_c = rx_c2.try_recv().unwrap();
    if let ComponentMessage::InterComponent { payload, .. } = recv_c {
        assert_eq!(payload, vec![67], "Component C got wrong message");
    }
    assert!(rx_c2.try_recv().is_err(), "Component C got extra messages");

    // Clean up original receivers (not used but declared)
    drop(rx_a);
    drop(rx_b);
    drop(rx_c);
}

// ============================================================================
// Test 7: Mailbox Registration Lifecycle
// ============================================================================

/// Tests the full registration/unregistration lifecycle.
#[tokio::test]
async fn test_mailbox_registration_lifecycle() {
    let broker = Arc::new(InMemoryMessageBroker::new());
    let subscriber_manager = Arc::new(SubscriberManager::new());

    let subscriber = ActorSystemSubscriber::new(Arc::clone(&broker), subscriber_manager);

    let component_id = ComponentId::new("lifecycle-test");

    // Initial state: no mailboxes
    assert_eq!(subscriber.mailbox_count().await, 0);

    // Register
    let (tx, _rx) = mpsc::unbounded_channel::<ComponentMessage>();
    subscriber
        .register_mailbox(component_id.clone(), tx)
        .await
        .expect("Failed to register");
    assert_eq!(subscriber.mailbox_count().await, 1);

    // Unregister
    let removed = subscriber.unregister_mailbox(&component_id).await;
    assert!(removed.is_some(), "Should return removed sender");
    assert_eq!(subscriber.mailbox_count().await, 0);

    // Re-register (should work)
    let (tx2, _rx2) = mpsc::unbounded_channel::<ComponentMessage>();
    subscriber
        .register_mailbox(component_id.clone(), tx2)
        .await
        .expect("Failed to re-register");
    assert_eq!(subscriber.mailbox_count().await, 1);

    // Duplicate registration should fail
    let (tx3, _rx3) = mpsc::unbounded_channel::<ComponentMessage>();
    let result = subscriber.register_mailbox(component_id.clone(), tx3).await;
    assert!(result.is_err(), "Duplicate registration should fail");
}
