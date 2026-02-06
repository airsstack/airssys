//! Integration tests for ComponentSpawner.
//!
//! Tests verify end-to-end component spawning with real ActorSystem and
//! MessageBroker instances. These tests validate the full lifecycle:
//! load → validate → wrap → spawn → register.

use std::sync::Arc;

use airssys_rt::broker::InMemoryMessageBroker;
use airssys_rt::system::{ActorSystem, SystemConfig};
use airssys_wasm::component::registry::ComponentRegistry;
use airssys_wasm::component::spawner::{ComponentSpawner, SpawnerError};
use airssys_wasm::component::wrapper::ComponentActorMessage;
use airssys_wasm::core::component::handle::ComponentHandle;
use airssys_wasm::core::component::id::ComponentId;
use airssys_wasm::core::component::message::ComponentMessage;
use airssys_wasm::core::runtime::errors::WasmError;
use airssys_wasm::core::runtime::traits::{ComponentLoader, RuntimeEngine};

// ========================================
// Mock RuntimeEngine for Testing
// ========================================

struct MockRuntimeEngine;

impl RuntimeEngine for MockRuntimeEngine {
    fn load_component(
        &self,
        id: &ComponentId,
        _bytes: &[u8],
    ) -> Result<ComponentHandle, WasmError> {
        Ok(ComponentHandle::new(id.clone(), 1))
    }

    fn unload_component(&self, _handle: &ComponentHandle) -> Result<(), WasmError> {
        Ok(())
    }

    fn call_handle_message(
        &self,
        _handle: &ComponentHandle,
        _message: &ComponentMessage,
    ) -> Result<Option<airssys_wasm::core::component::message::MessagePayload>, WasmError> {
        Ok(None)
    }

    fn call_handle_callback(
        &self,
        _handle: &ComponentHandle,
        _message: &ComponentMessage,
    ) -> Result<(), WasmError> {
        Ok(())
    }
}

// ========================================
// Mock ComponentLoader for Testing
// ========================================

struct MockComponentLoader {
    fail_load: bool,
    fail_validation: bool,
}

impl MockComponentLoader {
    fn new() -> Self {
        Self {
            fail_load: false,
            fail_validation: false,
        }
    }

    fn with_load_failure() -> Self {
        Self {
            fail_load: true,
            fail_validation: false,
        }
    }

    fn with_validation_failure() -> Self {
        Self {
            fail_load: false,
            fail_validation: true,
        }
    }
}

impl ComponentLoader for MockComponentLoader {
    fn load_bytes(&self, _id: &ComponentId) -> Result<Vec<u8>, WasmError> {
        if self.fail_load {
            Err(WasmError::ComponentNotFound(
                "Mock load failure".to_string(),
            ))
        } else {
            Ok(vec![0x00, 0x61, 0x73, 0x6d]) // Mock WASM magic bytes
        }
    }

    fn validate(&self, _bytes: &[u8]) -> Result<(), WasmError> {
        if self.fail_validation {
            Err(WasmError::InvalidComponent(
                "Mock validation failure".to_string(),
            ))
        } else {
            Ok(())
        }
    }
}

// ========================================
// Helper Functions
// ========================================

fn create_test_id(name: &str) -> ComponentId {
    ComponentId::new("test", name, "v1")
}

// ========================================
// Integration Tests
// ========================================

#[tokio::test]
async fn test_spawn_full_lifecycle_integration() {
    let broker = InMemoryMessageBroker::<ComponentActorMessage>::new();
    let system = ActorSystem::new(SystemConfig::default(), broker);

    let engine = Arc::new(MockRuntimeEngine);
    let loader = Arc::new(MockComponentLoader::new());
    let registry = Arc::new(ComponentRegistry::new());

    let spawner = ComponentSpawner::new(engine, loader, registry);

    let id = create_test_id("lifecycle-test");

    // Spawn component
    let result = spawner.spawn(&system, id.clone()).await;
    assert!(result.is_ok(), "Spawn should succeed");
    let address = result.unwrap();
    assert!(address.name().is_some(), "Actor should have a name");

    // Verify component is registered
    assert!(
        spawner.is_spawned(&id).unwrap(),
        "Component should be registered"
    );
    assert_eq!(
        spawner.spawned_count().unwrap(),
        1,
        "Registry should have 1 component"
    );

    // Stop component
    let stop_result = spawner.stop(&id);
    assert!(stop_result.is_ok(), "Stop should succeed");
    assert!(
        !spawner.is_spawned(&id).unwrap(),
        "Component should be unregistered"
    );
    assert_eq!(
        spawner.spawned_count().unwrap(),
        0,
        "Registry should be empty"
    );

    // Cleanup
    system.force_shutdown().await;
}

#[tokio::test]
async fn test_spawn_duplicate_rejected_integration() {
    let broker = InMemoryMessageBroker::<ComponentActorMessage>::new();
    let system = ActorSystem::new(SystemConfig::default(), broker);

    let engine = Arc::new(MockRuntimeEngine);
    let loader = Arc::new(MockComponentLoader::new());
    let registry = Arc::new(ComponentRegistry::new());

    let spawner = ComponentSpawner::new(engine, loader, registry);

    let id = create_test_id("duplicate-test");

    // First spawn succeeds
    let result1 = spawner.spawn(&system, id.clone()).await;
    assert!(result1.is_ok(), "First spawn should succeed");

    // Second spawn of same ID fails
    let result2 = spawner.spawn(&system, id).await;
    assert!(result2.is_err(), "Duplicate spawn should fail");

    match result2 {
        Err(SpawnerError::AlreadySpawned(msg)) => {
            assert!(
                msg.contains("test/duplicate-test/v1"),
                "Error should contain component ID"
            );
        }
        _ => panic!("Expected AlreadySpawned error"),
    }

    // Cleanup
    system.force_shutdown().await;
}

#[tokio::test]
async fn test_spawn_load_failure_does_not_register_integration() {
    let broker = InMemoryMessageBroker::<ComponentActorMessage>::new();
    let system = ActorSystem::new(SystemConfig::default(), broker);

    let engine = Arc::new(MockRuntimeEngine);
    let loader = Arc::new(MockComponentLoader::with_load_failure());
    let registry = Arc::new(ComponentRegistry::new());

    let spawner = ComponentSpawner::new(engine, loader, registry);

    let id = create_test_id("load-fail");

    let result = spawner.spawn(&system, id.clone()).await;
    assert!(result.is_err(), "Spawn should fail due to load error");

    match result {
        Err(SpawnerError::LoadFailed(comp_id, _)) => {
            assert_eq!(comp_id, "test/load-fail/v1");
        }
        _ => panic!("Expected LoadFailed error"),
    }

    // Should NOT be registered
    assert!(
        !spawner.is_spawned(&id).unwrap(),
        "Failed component should not be registered"
    );
    assert_eq!(
        spawner.spawned_count().unwrap(),
        0,
        "Registry should be empty"
    );

    system.force_shutdown().await;
}

#[tokio::test]
async fn test_spawn_validation_failure_does_not_register_integration() {
    let broker = InMemoryMessageBroker::<ComponentActorMessage>::new();
    let system = ActorSystem::new(SystemConfig::default(), broker);

    let engine = Arc::new(MockRuntimeEngine);
    let loader = Arc::new(MockComponentLoader::with_validation_failure());
    let registry = Arc::new(ComponentRegistry::new());

    let spawner = ComponentSpawner::new(engine, loader, registry);

    let id = create_test_id("validate-fail");

    let result = spawner.spawn(&system, id.clone()).await;
    assert!(result.is_err(), "Spawn should fail due to validation error");

    match result {
        Err(SpawnerError::ValidationFailed(comp_id, _)) => {
            assert_eq!(comp_id, "test/validate-fail/v1");
        }
        _ => panic!("Expected ValidationFailed error"),
    }

    // Should NOT be registered
    assert!(
        !spawner.is_spawned(&id).unwrap(),
        "Invalid component should not be registered"
    );

    system.force_shutdown().await;
}

#[tokio::test]
async fn test_spawn_multiple_components_integration() {
    let broker = InMemoryMessageBroker::<ComponentActorMessage>::new();
    let system = ActorSystem::new(SystemConfig::default(), broker);

    let engine = Arc::new(MockRuntimeEngine);
    let loader = Arc::new(MockComponentLoader::new());
    let registry = Arc::new(ComponentRegistry::new());

    let spawner = ComponentSpawner::new(engine, loader, registry);

    let id1 = create_test_id("comp-1");
    let id2 = create_test_id("comp-2");
    let id3 = create_test_id("comp-3");

    let result1 = spawner.spawn(&system, id1.clone()).await;
    assert!(result1.is_ok(), "First spawn should succeed");

    let result2 = spawner.spawn(&system, id2.clone()).await;
    assert!(result2.is_ok(), "Second spawn should succeed");

    let result3 = spawner.spawn(&system, id3.clone()).await;
    assert!(result3.is_ok(), "Third spawn should succeed");

    assert_eq!(
        spawner.spawned_count().unwrap(),
        3,
        "Registry should have 3 components"
    );
    assert!(
        spawner.is_spawned(&id1).unwrap(),
        "comp-1 should be spawned"
    );
    assert!(
        spawner.is_spawned(&id2).unwrap(),
        "comp-2 should be spawned"
    );
    assert!(
        spawner.is_spawned(&id3).unwrap(),
        "comp-3 should be spawned"
    );

    system.force_shutdown().await;
}

#[tokio::test]
async fn test_spawn_actor_name_format_integration() {
    let broker = InMemoryMessageBroker::<ComponentActorMessage>::new();
    let system = ActorSystem::new(SystemConfig::default(), broker);

    let engine = Arc::new(MockRuntimeEngine);
    let loader = Arc::new(MockComponentLoader::new());
    let registry = Arc::new(ComponentRegistry::new());

    let spawner = ComponentSpawner::new(engine, loader, registry);

    let id = ComponentId::new("system", "database", "prod");
    let result = spawner.spawn(&system, id).await;
    assert!(result.is_ok(), "Spawn should succeed");

    let address = result.unwrap();
    // Actor name should be "wasm-component-system/database/prod"
    let actor_name = address.name().expect("Actor should have a name");
    assert!(
        actor_name.starts_with("wasm-component-"),
        "Actor name should have wasm-component- prefix"
    );
    assert!(
        actor_name.contains("system/database/prod"),
        "Actor name should contain full component ID"
    );

    system.force_shutdown().await;
}

#[tokio::test]
async fn test_stop_nonexistent_component_integration() {
    let engine = Arc::new(MockRuntimeEngine);
    let loader = Arc::new(MockComponentLoader::new());
    let registry = Arc::new(ComponentRegistry::new());

    let spawner = ComponentSpawner::new(engine, loader, registry);

    let id = create_test_id("nonexistent");

    let result = spawner.stop(&id);
    assert!(result.is_err(), "Stop of nonexistent component should fail");

    match result {
        Err(SpawnerError::NotSpawned(comp_id)) => {
            assert_eq!(comp_id, "test/nonexistent/v1");
        }
        _ => panic!("Expected NotSpawned error"),
    }
}
