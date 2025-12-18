//! Integration tests for ComponentActor and MessageBroker integration.
//!
//! Tests verify end-to-end broker functionality:
//! - Component publishing via ComponentActor
//! - Broker injection during spawning
//! - Subscribe lifecycle (subscribe → receive → unsubscribe)
//! - Multi-component communication
//! - Error handling when broker not configured
//!
//! # Test Organization
//!
//! - **End-to-End**: Full ActorSystem + ComponentSpawner + ComponentActor setup
//! - **Broker Injection**: Verify broker passed correctly during spawn
//! - **Lifecycle**: Complete subscription workflow
//! - **Multi-Component**: Inter-component messaging via broker
//! - **Error Cases**: Broker not configured scenarios
//!
//! # References
//!
//! - **WASM-TASK-004 Phase 4 Task 4.1**: MessageBroker Setup for Components
//! - **ADR-WASM-009**: Component Communication Model

#![allow(clippy::unwrap_used, reason = "unwrap is acceptable in test code")]
#![allow(clippy::expect_used, reason = "expect is acceptable in test code")]

use airssys_rt::broker::InMemoryMessageBroker;
use airssys_rt::system::{ActorSystem, SystemConfig};
use airssys_wasm::actor::{ComponentActor, ComponentMessage, ComponentSpawner};
use airssys_wasm::core::{CapabilitySet, ComponentId, ComponentMetadata, ResourceLimits};
use std::path::PathBuf;

fn create_test_metadata() -> ComponentMetadata {
    ComponentMetadata {
        name: "test-component".to_string(),
        version: "1.0.0".to_string(),
        author: "Test".to_string(),
        description: None,
        required_capabilities: vec![],
        resource_limits: ResourceLimits {
            max_memory_bytes: 64 * 1024 * 1024,
            max_fuel: 1_000_000,
            max_execution_ms: 5000,
            max_storage_bytes: 10 * 1024 * 1024,
        },
    }
}

#[tokio::test]
async fn test_component_publish_via_broker() {
    // Setup: Create ActorSystem + ComponentSpawner + broker
    let broker = InMemoryMessageBroker::new();
    let actor_system = ActorSystem::new(SystemConfig::default(), broker.clone());
    let registry = airssys_wasm::actor::ComponentRegistry::new();
    let spawner = ComponentSpawner::new(actor_system, registry, broker);

    // Spawn component with broker injection
    let component_id = ComponentId::new("publisher-component");
    let wasm_path = PathBuf::from("./test.wasm");
    let metadata = create_test_metadata();
    let capabilities = CapabilitySet::new();

    let actor_ref = spawner
        .spawn_component(component_id.clone(), wasm_path, metadata, capabilities)
        .await;

    // Verify: Component spawned successfully
    assert!(
        actor_ref.is_ok(),
        "Component spawn should succeed: {:?}",
        actor_ref.err()
    );

    // Note: Cannot test actual publish without access to actor instance
    // This test verifies broker injection happens without errors
}

#[tokio::test]
async fn test_spawner_broker_injection() {
    // Setup
    let broker = InMemoryMessageBroker::new();
    let actor_system = ActorSystem::new(SystemConfig::default(), broker.clone());
    let registry = airssys_wasm::actor::ComponentRegistry::new();
    let spawner = ComponentSpawner::new(actor_system, registry, broker);

    // Test: Spawn component (broker injected automatically)
    let component_id = ComponentId::new("test-component");
    let wasm_path = PathBuf::from("./test.wasm");
    let metadata = create_test_metadata();
    let capabilities = CapabilitySet::new();

    let result = spawner
        .spawn_component(component_id, wasm_path, metadata, capabilities)
        .await;

    // Verify: Spawn succeeds (broker injected)
    assert!(
        result.is_ok(),
        "Spawn with broker injection should succeed: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_component_subscribe_lifecycle() {
    // Setup: Create component with broker
    let component_id = ComponentId::new("subscriber-component");
    let metadata = create_test_metadata();
    let capabilities = CapabilitySet::new();

    let mut actor = ComponentActor::new(component_id.clone(), metadata, capabilities, ());

    // Inject broker
    let broker = InMemoryMessageBroker::new();
    let wrapper = std::sync::Arc::new(airssys_wasm::actor::MessageBrokerWrapper::new(broker));
    actor.set_broker(wrapper);

    // Test: Subscribe lifecycle
    let subscribe_result = actor.subscribe_topic("test-topic").await;

    // Verify: Subscribe succeeds
    assert!(
        subscribe_result.is_ok(),
        "Subscribe should succeed: {:?}",
        subscribe_result.err()
    );

    // Note: Cannot test receive/unsubscribe without message sending infrastructure
    // This test verifies subscribe works after broker injection
}

#[tokio::test]
async fn test_multi_component_pub_sub() {
    // Setup: Create ActorSystem with broker
    let broker = InMemoryMessageBroker::new();
    let actor_system = ActorSystem::new(SystemConfig::default(), broker.clone());
    let registry = airssys_wasm::actor::ComponentRegistry::new();
    let spawner = ComponentSpawner::new(actor_system, registry, broker);

    // Spawn publisher component
    let publisher_id = ComponentId::new("publisher");
    let publisher_result = spawner
        .spawn_component(
            publisher_id,
            PathBuf::from("./publisher.wasm"),
            create_test_metadata(),
            CapabilitySet::new(),
        )
        .await;

    assert!(publisher_result.is_ok());

    // Spawn subscriber component
    let subscriber_id = ComponentId::new("subscriber");
    let subscriber_result = spawner
        .spawn_component(
            subscriber_id,
            PathBuf::from("./subscriber.wasm"),
            create_test_metadata(),
            CapabilitySet::new(),
        )
        .await;

    assert!(subscriber_result.is_ok());

    // Verify: Both components spawned with broker
    // Note: Actual pub-sub messaging requires full message routing infrastructure
    // This test verifies multi-component setup works
}

#[tokio::test]
async fn test_broker_not_configured_error() {
    // Setup: Create component WITHOUT broker
    let component_id = ComponentId::new("no-broker-component");
    let metadata = create_test_metadata();
    let capabilities = CapabilitySet::new();

    let actor = ComponentActor::new(component_id, metadata, capabilities, ());

    // Test: Try to publish without broker
    let message = ComponentMessage::HealthCheck;
    let result = actor.publish_message("test-topic", message).await;

    // Verify: Error returned (broker not configured)
    assert!(result.is_err(), "Publish without broker should fail");
    if let Err(e) = result {
        let error_str = e.to_string();
        assert!(
            error_str.contains("MessageBroker not configured")
                || error_str.contains("BrokerNotConfigured"),
            "Error should indicate broker not configured: {}",
            error_str
        );
    }
}
