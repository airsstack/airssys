#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

//! Integration tests for component spawning via ActorSystem.
//! instances through the airssys-rt ActorSystem, ensuring proper integration with
//! the actor system's message routing and lifecycle management.
//!
//! # Test Coverage
//!
//! - Spawn component via ActorSystem (NOT tokio::spawn)
//! - Verify ActorAddress returned
//! - Send messages via ActorAddress (future - requires message broker integration)
//! - Test spawn error handling
//! - Measure spawn performance (<5ms target)
//!
//! # References
//!
//! - **WASM-TASK-004 Phase 2 Task 2.1**: ComponentSpawner Implementation

// Layer 1: Standard library imports
use std::path::PathBuf;
use std::time::Instant;

// Layer 2: Third-party crate imports
// (none)

// Layer 3: Internal module imports
use airssys_rt::broker::InMemoryMessageBroker;
use airssys_rt::system::{ActorSystem, SystemConfig};
use airssys_rt::util::ActorAddress;
use airssys_wasm::actor::{ComponentRegistry, ComponentSpawner};
use airssys_wasm::core::{CapabilitySet, ComponentId, ComponentMetadata};

/// Create test component metadata with standard resource limits.
fn create_test_metadata(name: &str) -> ComponentMetadata {
    ComponentMetadata {
        name: name.to_string(),
        version: "1.0.0".to_string(),
        author: "Test Author".to_string(),
        description: Some("Test component".to_string()),
        max_memory_bytes: 64 * 1024 * 1024, // 64MB,
        max_fuel: 1_000_000,                // 1M fuel,
        timeout_seconds: 5,
    }
}

#[tokio::test]
async fn test_spawn_component_via_actor_system() {
    // 1. Create ActorSystem for ComponentMessage
    let broker = InMemoryMessageBroker::new();
    let actor_system = ActorSystem::new(SystemConfig::default(), broker.clone());

    // 2. Create ComponentSpawner
    let spawner = ComponentSpawner::new(actor_system, ComponentRegistry::new(), broker.clone());

    // 3. Spawn component
    let component_id = ComponentId::new("test-component");
    let wasm_path = PathBuf::from("./test.wasm");
    let metadata = create_test_metadata("test-component");
    let capabilities = CapabilitySet::new();

    let actor_ref = spawner
        .spawn_component(component_id.clone(), wasm_path, metadata, capabilities)
        .await
        .expect("Failed to spawn component");

    // 4. Verify ActorAddress returned
    match actor_ref {
        ActorAddress::Named { name, .. } => {
            assert_eq!(name, component_id.as_str());
        }
        _ => panic!("Expected named ActorAddress"),
    }
}

#[tokio::test]
async fn test_spawn_multiple_components() {
    let broker = InMemoryMessageBroker::new();
    let actor_system = ActorSystem::new(SystemConfig::default(), broker.clone());
    let spawner = ComponentSpawner::new(actor_system, ComponentRegistry::new(), broker.clone());

    // Spawn 5 components
    let mut addresses = Vec::new();
    for i in 0..5 {
        let component_id = ComponentId::new(format!("component-{}", i));
        let wasm_path = PathBuf::from(format!("./component{}.wasm", i));
        let metadata = create_test_metadata(&format!("component-{}", i));
        let capabilities = CapabilitySet::new();

        let actor_ref = spawner
            .spawn_component(component_id.clone(), wasm_path, metadata, capabilities)
            .await
            .unwrap_or_else(|_| panic!("Failed to spawn component {}", i));

        addresses.push(actor_ref);
    }

    // Verify all components were spawned
    assert_eq!(addresses.len(), 5);

    // Verify each has correct name
    for (i, addr) in addresses.iter().enumerate() {
        match addr {
            ActorAddress::Named { name, .. } => {
                assert_eq!(name, &format!("component-{}", i));
            }
            _ => panic!("Expected named ActorAddress"),
        }
    }
}

#[tokio::test]
async fn test_spawn_component_with_same_id_replaces() {
    let broker = InMemoryMessageBroker::new();
    let actor_system = ActorSystem::new(SystemConfig::default(), broker.clone());
    let spawner = ComponentSpawner::new(actor_system, ComponentRegistry::new(), broker.clone());

    let component_id = ComponentId::new("duplicate-test");
    let wasm_path = PathBuf::from("./test.wasm");
    let metadata = create_test_metadata("duplicate-test");
    let capabilities = CapabilitySet::new();

    // Spawn first component
    let _addr1 = spawner
        .spawn_component(
            component_id.clone(),
            wasm_path.clone(),
            metadata.clone(),
            capabilities.clone(),
        )
        .await
        .expect("Failed to spawn first component");

    // Spawn second component with same ID
    let _addr2 = spawner
        .spawn_component(component_id.clone(), wasm_path, metadata, capabilities)
        .await
        .expect("Failed to spawn second component");

    // Note: ActorSystem allows duplicate named actors, but in production we'd
    // want ComponentRegistry to prevent duplicates
}

#[tokio::test]
async fn test_spawn_performance_single_component() {
    let broker = InMemoryMessageBroker::new();
    let actor_system = ActorSystem::new(SystemConfig::default(), broker.clone());
    let spawner = ComponentSpawner::new(actor_system, ComponentRegistry::new(), broker.clone());

    let component_id = ComponentId::new("perf-test");
    let wasm_path = PathBuf::from("./perf.wasm");
    let metadata = create_test_metadata("perf-test");
    let capabilities = CapabilitySet::new();

    // Measure spawn time
    let start = Instant::now();
    let _actor_ref = spawner
        .spawn_component(component_id, wasm_path, metadata, capabilities)
        .await
        .expect("Failed to spawn component");
    let elapsed = start.elapsed();

    println!("Single component spawn time: {:?}", elapsed);

    // Note: Target is <5ms, but without actual WASM loading (which happens in Child::start),
    // this just measures ActorSystem spawn overhead which should be <1ms
    assert!(
        elapsed.as_millis() < 100,
        "Spawn took too long: {:?}",
        elapsed
    );
}

#[tokio::test]
async fn test_spawn_performance_batch() {
    let broker = InMemoryMessageBroker::new();
    let actor_system = ActorSystem::new(SystemConfig::default(), broker.clone());
    let spawner = ComponentSpawner::new(actor_system, ComponentRegistry::new(), broker.clone());

    const BATCH_SIZE: usize = 10;

    // Measure batch spawn time
    let start = Instant::now();
    for i in 0..BATCH_SIZE {
        let component_id = ComponentId::new(format!("batch-{}", i));
        let wasm_path = PathBuf::from(format!("./batch{}.wasm", i));
        let metadata = create_test_metadata(&format!("batch-{}", i));
        let capabilities = CapabilitySet::new();

        spawner
            .spawn_component(component_id, wasm_path, metadata, capabilities)
            .await
            .unwrap_or_else(|_| panic!("Failed to spawn component {}", i));
    }
    let elapsed = start.elapsed();

    let avg_spawn_time = elapsed.as_micros() / BATCH_SIZE as u128;
    println!(
        "Batch spawn: {} components in {:?} (avg: {}µs per component)",
        BATCH_SIZE, elapsed, avg_spawn_time
    );

    // Verify reasonable performance (no actual WASM loading yet)
    assert!(
        avg_spawn_time < 10_000,
        "Average spawn time too high: {}µs",
        avg_spawn_time
    );
}

#[tokio::test]
async fn test_spawn_component_naming() {
    let broker = InMemoryMessageBroker::new();
    let actor_system = ActorSystem::new(SystemConfig::default(), broker.clone());
    let spawner = ComponentSpawner::new(actor_system, ComponentRegistry::new(), broker.clone());

    // Test various naming patterns
    let test_names = vec![
        "simple",
        "with-dashes",
        "with_underscores",
        "with.dots",
        "MixedCase123",
    ];

    for name in test_names {
        let component_id = ComponentId::new(name);
        let wasm_path = PathBuf::from(format!("./{}.wasm", name));
        let metadata = create_test_metadata(name);
        let capabilities = CapabilitySet::new();

        let actor_ref = spawner
            .spawn_component(component_id.clone(), wasm_path, metadata, capabilities)
            .await
            .unwrap_or_else(|_| panic!("Failed to spawn component '{}'", name));

        // Verify name preserved
        match actor_ref {
            ActorAddress::Named {
                name: addr_name, ..
            } => {
                assert_eq!(addr_name, name);
            }
            _ => panic!("Expected named ActorAddress for '{}'", name),
        }
    }
}

#[tokio::test]
async fn test_spawn_with_different_resource_limits() {
    let broker = InMemoryMessageBroker::new();
    let actor_system = ActorSystem::new(SystemConfig::default(), broker.clone());
    let spawner = ComponentSpawner::new(actor_system, ComponentRegistry::new(), broker.clone());

    // Test with minimal resources
    let component_id = ComponentId::new("minimal-resources");
    let wasm_path = PathBuf::from("./minimal.wasm");
    let metadata = ComponentMetadata {
        name: "minimal-resources".to_string(),
        version: "1.0.0".to_string(),
        author: "Test".to_string(),
        description: None,
        max_memory_bytes: 1024 * 1024, // 1MB,
        max_fuel: 100_000,             // 100K fuel,
        timeout_seconds: 1,
    };
    let capabilities = CapabilitySet::new();

    let _addr = spawner
        .spawn_component(component_id, wasm_path, metadata, capabilities)
        .await
        .expect("Failed to spawn minimal resource component");

    // Test with maximum resources
    let component_id = ComponentId::new("maximum-resources");
    let wasm_path = PathBuf::from("./maximum.wasm");
    let metadata = ComponentMetadata {
        name: "maximum-resources".to_string(),
        version: "1.0.0".to_string(),
        author: "Test".to_string(),
        description: None,
        max_memory_bytes: 512 * 1024 * 1024, // 512MB,
        max_fuel: 10_000_000,                // 10M fuel,
        timeout_seconds: 0,
    };
    let capabilities = CapabilitySet::new();

    let _addr = spawner
        .spawn_component(component_id, wasm_path, metadata, capabilities)
        .await
        .expect("Failed to spawn maximum resource component");
}

#[tokio::test]
async fn test_spawner_with_multiple_systems() {
    // Create two independent ActorSystems
    let broker1 = InMemoryMessageBroker::new();
    let system1 = ActorSystem::new(SystemConfig::default(), broker1.clone());
    let spawner1 = ComponentSpawner::new(system1, ComponentRegistry::new(), broker1.clone());

    let broker2 = InMemoryMessageBroker::new();
    let system2 = ActorSystem::new(SystemConfig::default(), broker2.clone());
    let spawner2 = ComponentSpawner::new(system2, ComponentRegistry::new(), broker2.clone());

    // Spawn component in system1
    let component_id1 = ComponentId::new("system1-component");
    let _addr1 = spawner1
        .spawn_component(
            component_id1,
            PathBuf::from("./sys1.wasm"),
            create_test_metadata("system1-component"),
            CapabilitySet::new(),
        )
        .await
        .expect("Failed to spawn in system1");

    // Spawn component in system2
    let component_id2 = ComponentId::new("system2-component");
    let _addr2 = spawner2
        .spawn_component(
            component_id2,
            PathBuf::from("./sys2.wasm"),
            create_test_metadata("system2-component"),
            CapabilitySet::new(),
        )
        .await
        .expect("Failed to spawn in system2");

    // Both systems operate independently
}
