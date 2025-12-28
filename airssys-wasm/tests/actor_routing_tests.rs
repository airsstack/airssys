#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

//! Integration tests for actor address routing.
//! - ComponentActor receives messages

// Layer 2: Third-party crate imports
// (none)
use std::path::PathBuf;

// Layer 3: Internal module imports
use airssys_rt::broker::InMemoryMessageBroker;
use airssys_rt::system::{ActorSystem, SystemConfig};
use airssys_wasm::actor::{ComponentMessage, ComponentRegistry, ComponentSpawner};
use airssys_wasm::core::{CapabilitySet, ComponentId, ComponentMetadata};

fn create_test_metadata() -> ComponentMetadata {
    ComponentMetadata {
        name: "test-component".to_string(),
        version: "1.0.0".to_string(),
        author: "Test".to_string(),
        description: None,
        max_memory_bytes: 64 * 1024 * 1024,
        max_fuel: 1_000_000,
        timeout_seconds: 5,
    }
}

#[tokio::test]
async fn test_end_to_end_message_routing() {
    // Setup
    let broker = InMemoryMessageBroker::new();
    let actor_system = ActorSystem::new(SystemConfig::default(), broker.clone());
    let registry = ComponentRegistry::new();
    let spawner = ComponentSpawner::new(actor_system, registry.clone(), broker);
    let router = spawner.create_router();

    // Spawn component
    let component_id = ComponentId::new("test-component");
    let wasm_path = PathBuf::from("./test.wasm");
    let metadata = create_test_metadata();
    let caps = CapabilitySet::new();

    let actor_address = spawner
        .spawn_component(component_id.clone(), wasm_path, metadata, caps)
        .await
        .expect("Failed to spawn component");

    // Verify registration
    assert!(router.component_exists(&component_id));

    // Send message via router
    let message = ComponentMessage::HealthCheck;
    router
        .send_message(&component_id, message)
        .await
        .expect("Failed to route message");

    // Note: Full verification requires ComponentActor to handle message
    // and emit observable side effect (e.g., metric, log, response)
    // This is verified in ComponentActor integration tests (Task 1.3)

    // Cleanup: Stop actor system
    drop(actor_address);
}

#[tokio::test]
async fn test_routing_to_nonexistent_component() {
    let broker = InMemoryMessageBroker::new();
    let actor_system = ActorSystem::new(SystemConfig::default(), broker.clone());
    let registry = ComponentRegistry::new();
    let spawner = ComponentSpawner::new(actor_system, registry.clone(), broker);
    let router = spawner.create_router();

    let nonexistent = ComponentId::new("does-not-exist");
    let message = ComponentMessage::HealthCheck;

    let result = router.send_message(&nonexistent, message).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("not found"));
}

#[tokio::test]
async fn test_broadcast_to_multiple_components() {
    let broker = InMemoryMessageBroker::new();
    let actor_system = ActorSystem::new(SystemConfig::default(), broker.clone());
    let registry = ComponentRegistry::new();
    let spawner = ComponentSpawner::new(actor_system, registry.clone(), broker);
    let router = spawner.create_router();

    // Spawn 3 components
    let mut component_ids = Vec::new();
    for i in 0..3 {
        let component_id = ComponentId::new(format!("component-{}", i));
        let wasm_path = PathBuf::from(format!("./test{}.wasm", i));
        let metadata = create_test_metadata();
        let caps = CapabilitySet::new();

        spawner
            .spawn_component(component_id.clone(), wasm_path, metadata, caps)
            .await
            .expect("Failed to spawn component");

        component_ids.push(component_id);
    }

    // Broadcast message
    let message = ComponentMessage::HealthCheck;
    router
        .broadcast_message(&component_ids, message)
        .await
        .expect("Broadcast failed");

    // All components should have received message
    // (verification would require observable side effects in ComponentActor)
}

#[tokio::test]
async fn test_try_broadcast_with_mixed_results() {
    let broker = InMemoryMessageBroker::new();
    let actor_system = ActorSystem::new(SystemConfig::default(), broker.clone());
    let registry = ComponentRegistry::new();
    let spawner = ComponentSpawner::new(actor_system, registry.clone(), broker);
    let router = spawner.create_router();

    // Spawn one component
    let existing = ComponentId::new("existing");
    let wasm_path = PathBuf::from("./existing.wasm");
    spawner
        .spawn_component(
            existing.clone(),
            wasm_path,
            create_test_metadata(),
            CapabilitySet::new(),
        )
        .await
        .unwrap();

    // Try broadcast to existing and nonexistent components
    let targets = vec![
        existing.clone(),
        ComponentId::new("nonexistent-1"),
        ComponentId::new("nonexistent-2"),
    ];

    let results = router
        .try_broadcast_message(&targets, ComponentMessage::HealthCheck)
        .await;

    assert_eq!(results.len(), 3);
    assert!(results[0].1.is_ok()); // existing
    assert!(results[1].1.is_err()); // nonexistent-1
    assert!(results[2].1.is_err()); // nonexistent-2
}

#[tokio::test]
async fn test_router_registry_integration() {
    let broker = InMemoryMessageBroker::new();
    let actor_system = ActorSystem::new(SystemConfig::default(), broker.clone());
    let registry = ComponentRegistry::new();
    let spawner = ComponentSpawner::new(actor_system, registry.clone(), broker);
    let router = spawner.create_router();

    // Initial state
    assert_eq!(router.component_count().unwrap(), 0);

    // Spawn components
    let component_id1 = ComponentId::new("component-1");
    let component_id2 = ComponentId::new("component-2");

    spawner
        .spawn_component(
            component_id1.clone(),
            PathBuf::from("./test1.wasm"),
            create_test_metadata(),
            CapabilitySet::new(),
        )
        .await
        .unwrap();

    spawner
        .spawn_component(
            component_id2.clone(),
            PathBuf::from("./test2.wasm"),
            create_test_metadata(),
            CapabilitySet::new(),
        )
        .await
        .unwrap();

    // Verify registry integration
    assert_eq!(router.component_count().unwrap(), 2);
    assert!(router.component_exists(&component_id1));
    assert!(router.component_exists(&component_id2));

    // Verify can route to both
    router
        .send_message(&component_id1, ComponentMessage::HealthCheck)
        .await
        .unwrap();
    router
        .send_message(&component_id2, ComponentMessage::HealthCheck)
        .await
        .unwrap();

    // Unregister one component
    registry.unregister(&component_id1).unwrap();

    // Verify router sees the change
    assert_eq!(router.component_count().unwrap(), 1);
    assert!(!router.component_exists(&component_id1));
    assert!(router.component_exists(&component_id2));

    // Verify can't route to unregistered component
    let result = router
        .send_message(&component_id1, ComponentMessage::HealthCheck)
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_concurrent_message_routing() {
    use tokio::task;

    let broker = InMemoryMessageBroker::new();
    let actor_system = ActorSystem::new(SystemConfig::default(), broker.clone());
    let registry = ComponentRegistry::new();
    let spawner = ComponentSpawner::new(actor_system, registry.clone(), broker);
    let router = spawner.create_router();

    // Spawn 10 components
    let mut component_ids = Vec::new();
    for i in 0..10 {
        let component_id = ComponentId::new(format!("concurrent-{}", i));
        let wasm_path = PathBuf::from(format!("./test{}.wasm", i));

        spawner
            .spawn_component(
                component_id.clone(),
                wasm_path,
                create_test_metadata(),
                CapabilitySet::new(),
            )
            .await
            .unwrap();

        component_ids.push(component_id);
    }

    // Route messages concurrently
    let mut handles = vec![];
    for component_id in component_ids {
        let router_clone = router.clone();

        let handle = task::spawn(async move {
            for _ in 0..10 {
                router_clone
                    .send_message(&component_id, ComponentMessage::HealthCheck)
                    .await
                    .expect("Failed to route message");
            }
        });

        handles.push(handle);
    }

    // Wait for all routing tasks to complete
    for handle in handles {
        handle.await.expect("Task panicked");
    }

    // All routing operations should have succeeded
    assert_eq!(registry.count().unwrap(), 10);
}
