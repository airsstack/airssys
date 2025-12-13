//! Integration tests for ComponentRegistry.
//!
//! This test suite verifies that ComponentRegistry correctly tracks component instances
//! with O(1) lookup performance and thread-safe concurrent access.
//!
//! # Test Coverage
//!
//! - Register component instance
//! - Lookup by ComponentId (verify O(1))
//! - Unregister component
//! - Lookup non-existent component (error handling)
//! - Concurrent access (thread safety)
//! - Performance benchmarks
//!
//! # References
//!
//! - **WASM-TASK-004 Phase 2 Task 2.2**: ComponentRegistry Implementation

#![expect(clippy::expect_used, reason = "expect is acceptable in test code for clear error messages")]
#![expect(clippy::unwrap_used, reason = "unwrap is acceptable in test code for convenience")]
#![expect(clippy::expect_fun_call, reason = "format! in expect is acceptable in test code")]
#![expect(clippy::panic, reason = "panic is acceptable in test code for assertion failures")]

// Layer 1: Standard library imports
use std::sync::Arc;
use std::time::Instant;

// Layer 2: Third-party crate imports
use tokio::task;

// Layer 3: Internal module imports
use airssys_wasm::actor::ComponentRegistry;
use airssys_wasm::core::{ComponentId, WasmError};
use airssys_rt::util::ActorAddress;

#[test]
fn test_registry_creation() {
    let registry = ComponentRegistry::new();
    assert_eq!(registry.count().expect("Failed to get count"), 0);
}

#[test]
fn test_register_single_component() {
    let registry = ComponentRegistry::new();
    let component_id = ComponentId::new("test-component");
    let actor_addr = ActorAddress::named("test-component");

    let result = registry.register(component_id.clone(), actor_addr.clone());
    assert!(result.is_ok());
    assert_eq!(registry.count().expect("Failed to get count"), 1);

    // Verify lookup
    let found = registry.lookup(&component_id).unwrap();
    assert_eq!(found, actor_addr);
}

#[test]
fn test_register_multiple_components() {
    let registry = ComponentRegistry::new();

    const COUNT: usize = 100;
    for i in 0..COUNT {
        let component_id = ComponentId::new(format!("component-{}", i));
        let actor_addr = ActorAddress::named(format!("component-{}", i));

        registry
            .register(component_id, actor_addr)
            .expect(&format!("Failed to register component {}", i));
    }

    assert_eq!(registry.count().expect("Failed to get count"), COUNT);
}

#[test]
fn test_lookup_component_o1_performance() {
    let registry = ComponentRegistry::new();

    // Register 1000 components
    const COUNT: usize = 1000;
    for i in 0..COUNT {
        let component_id = ComponentId::new(format!("component-{}", i));
        let actor_addr = ActorAddress::named(format!("component-{}", i));
        registry.register(component_id, actor_addr).unwrap();
    }

    assert_eq!(registry.count().expect("Failed to get count"), COUNT);

    // Measure lookup time for first component
    let target_id = ComponentId::new("component-0");
    let start = Instant::now();
    let _found = registry.lookup(&target_id).unwrap();
    let elapsed_first = start.elapsed();

    // Measure lookup time for last component
    let target_id = ComponentId::new(format!("component-{}", COUNT - 1));
    let start = Instant::now();
    let _found = registry.lookup(&target_id).unwrap();
    let elapsed_last = start.elapsed();

    println!(
        "Lookup time (first): {:?}, Lookup time (last): {:?}",
        elapsed_first, elapsed_last
    );

    // Both lookups should be O(1) - similar time regardless of position
    // Target: <100ns, but allow up to 10µs for test variability/CI/debuginfo builds
    assert!(elapsed_first.as_nanos() < 10_000, "First lookup too slow: {:?}", elapsed_first);
    assert!(elapsed_last.as_nanos() < 10_000, "Last lookup too slow: {:?}", elapsed_last);

    // Verify similar performance (within 50x factor for O(1) guarantee - generous for test variability)
    let ratio = elapsed_last.as_nanos() as f64 / elapsed_first.as_nanos().max(1) as f64;
    assert!(
        ratio < 50.0,
        "Lookup time varies too much (ratio: {:.2}), not O(1)",
        ratio
    );
}

#[test]
fn test_lookup_nonexistent_component() {
    let registry = ComponentRegistry::new();
    let component_id = ComponentId::new("nonexistent");

    let result = registry.lookup(&component_id);
    assert!(result.is_err());

    match result {
        Err(WasmError::ComponentNotFound { .. }) => {}
        _ => panic!("Expected ComponentNotFound error"),
    }
}

#[test]
fn test_unregister_component() {
    let registry = ComponentRegistry::new();
    let component_id = ComponentId::new("test-component");
    let actor_addr = ActorAddress::named("test-component");

    // Register
    registry
        .register(component_id.clone(), actor_addr)
        .unwrap();
    assert_eq!(registry.count().expect("Failed to get count"), 1);

    // Unregister
    registry.unregister(&component_id).unwrap();
    assert_eq!(registry.count().expect("Failed to get count"), 0);

    // Verify component is gone
    let result = registry.lookup(&component_id);
    assert!(result.is_err());
}

#[test]
fn test_unregister_nonexistent_component() {
    let registry = ComponentRegistry::new();
    let component_id = ComponentId::new("nonexistent");

    // Should succeed silently
    let result = registry.unregister(&component_id);
    assert!(result.is_ok());
    assert_eq!(registry.count().expect("Failed to get count"), 0);
}

#[test]
fn test_register_overwrites_existing() {
    let registry = ComponentRegistry::new();
    let component_id = ComponentId::new("test");
    let addr1 = ActorAddress::named("test-v1");
    let addr2 = ActorAddress::named("test-v2");

    // Register first time
    registry.register(component_id.clone(), addr1).unwrap();
    assert_eq!(registry.count().expect("Failed to get count"), 1);

    // Register again with different address
    registry.register(component_id.clone(), addr2.clone()).unwrap();
    assert_eq!(registry.count().expect("Failed to get count"), 1);

    // Verify new address
    let found = registry.lookup(&component_id).unwrap();
    assert_eq!(found, addr2);
}

#[tokio::test]
async fn test_concurrent_reads() {
    let registry = Arc::new(ComponentRegistry::new());

    // Register 10 components
    for i in 0..10 {
        let component_id = ComponentId::new(format!("component-{}", i));
        let actor_addr = ActorAddress::named(format!("component-{}", i));
        registry.register(component_id, actor_addr).unwrap();
    }

    // Spawn 50 concurrent readers
    let mut handles = vec![];
    for reader_id in 0..50 {
        let registry_clone = Arc::clone(&registry);
        let component_idx = reader_id % 10; // Each reader reads one of 10 components

        let handle = task::spawn(async move {
            let component_id = ComponentId::new(format!("component-{}", component_idx));
            let expected_name = format!("component-{}", component_idx);

            // Perform 100 lookups
            for _ in 0..100 {
                let found = registry_clone
                    .lookup(&component_id)
                    .expect("Lookup failed");
                
                // Compare names only (ActorAddress IDs will differ)
                assert_eq!(found.name().unwrap(), expected_name);
            }
        });

        handles.push(handle);
    }

    // Wait for all readers to complete
    for handle in handles {
        handle.await.unwrap();
    }

    println!("Successfully completed 50 readers × 100 lookups = 5000 concurrent lookups");
}

#[tokio::test]
async fn test_concurrent_reads_and_writes() {
    let registry = Arc::new(ComponentRegistry::new());

    // Pre-register 5 components
    for i in 0..5 {
        let component_id = ComponentId::new(format!("initial-{}", i));
        let actor_addr = ActorAddress::named(format!("initial-{}", i));
        registry.register(component_id, actor_addr).unwrap();
    }

    let mut handles = vec![];

    // Spawn 10 readers
    for _reader_id in 0..10 {
        let registry_clone = Arc::clone(&registry);
        let handle = task::spawn(async move {
            for i in 0..5 {
                let component_id = ComponentId::new(format!("initial-{}", i));
                let expected_name = format!("initial-{}", i);
                if let Ok(addr) = registry_clone.lookup(&component_id) {
                    assert_eq!(addr.name().unwrap(), expected_name);
                }
                tokio::time::sleep(tokio::time::Duration::from_micros(10)).await;
            }
        });
        handles.push(handle);
    }

    // Spawn 5 writers (registering new components)
    for writer_id in 0..5 {
        let registry_clone = Arc::clone(&registry);
        let handle = task::spawn(async move {
            for i in 0..10 {
                let component_id = ComponentId::new(format!("writer-{}-{}", writer_id, i));
                let actor_addr = ActorAddress::named(format!("writer-{}-{}", writer_id, i));
                registry_clone.register(component_id, actor_addr).unwrap();
                tokio::time::sleep(tokio::time::Duration::from_micros(10)).await;
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify final count (5 initial + 5 writers × 10 = 55)
    assert_eq!(registry.count().expect("Failed to get count"), 55);
    println!("Successfully completed concurrent reads and writes");
}

#[test]
fn test_registry_clone_shares_data() {
    let registry1 = ComponentRegistry::new();
    let component_id = ComponentId::new("test");
    let actor_addr = ActorAddress::named("test");

    registry1
        .register(component_id.clone(), actor_addr.clone())
        .unwrap();

    // Clone registry (Arc clone, shares data)
    let registry2 = registry1.clone();

    // Both registries see the same data
    assert_eq!(registry1.count().expect("Failed to get count"), 1);
    assert_eq!(registry2.count().expect("Failed to get count"), 1);

    // Lookup works in both
    let found1 = registry1.lookup(&component_id).unwrap();
    let found2 = registry2.lookup(&component_id).unwrap();
    assert_eq!(found1, actor_addr);
    assert_eq!(found2, actor_addr);

    // Register in registry2, visible in registry1
    let component_id2 = ComponentId::new("test2");
    let actor_addr2 = ActorAddress::named("test2");
    registry2.register(component_id2.clone(), actor_addr2.clone()).unwrap();

    assert_eq!(registry1.count().expect("Failed to get count"), 2);
    assert_eq!(registry2.count().expect("Failed to get count"), 2);

    let found = registry1.lookup(&component_id2).unwrap();
    assert_eq!(found, actor_addr2);
}

#[test]
fn test_lookup_performance_benchmark() {
    let registry = ComponentRegistry::new();

    // Register 10,000 components
    const COUNT: usize = 10_000;
    for i in 0..COUNT {
        let component_id = ComponentId::new(format!("component-{}", i));
        let actor_addr = ActorAddress::named(format!("component-{}", i));
        registry.register(component_id, actor_addr).unwrap();
    }

    // Benchmark 1000 lookups
    const LOOKUPS: usize = 1000;
    let start = Instant::now();
    for i in 0..LOOKUPS {
        let component_id = ComponentId::new(format!("component-{}", i % COUNT));
        let _found = registry.lookup(&component_id).unwrap();
    }
    let elapsed = start.elapsed();

    let avg_lookup_ns = elapsed.as_nanos() / LOOKUPS as u128;
    println!(
        "Average lookup time: {}ns (target: <100ns, {} components in registry)",
        avg_lookup_ns, COUNT
    );

    // Verify reasonable performance (target <100ns, allow up to 5µs for test variability/CI)
    assert!(
        avg_lookup_ns < 5000,
        "Average lookup time too high: {}ns",
        avg_lookup_ns
    );
}

#[test]
fn test_default_implementation() {
    let registry = ComponentRegistry::default();
    assert_eq!(registry.count().expect("Failed to get count"), 0);
}

#[test]
fn test_registry_with_complex_component_ids() {
    let registry = ComponentRegistry::new();

    // Test various ComponentId patterns
    let test_ids = vec![
        "simple",
        "with-dashes",
        "with_underscores",
        "with.dots",
        "MixedCase123",
        "veryLongComponentIdWithManyCharactersToTestPerformance123456789",
    ];

    for id_str in test_ids {
        let component_id = ComponentId::new(id_str);
        let actor_addr = ActorAddress::named(id_str);

        registry.register(component_id.clone(), actor_addr.clone()).unwrap();

        let found = registry.lookup(&component_id).unwrap();
        assert_eq!(found, actor_addr);
    }

    assert_eq!(registry.count().expect("Failed to get count"), 6);
}

#[test]
fn test_unregister_middle_component() {
    let registry = ComponentRegistry::new();

    // Register 5 components
    for i in 0..5 {
        let component_id = ComponentId::new(format!("component-{}", i));
        let actor_addr = ActorAddress::named(format!("component-{}", i));
        registry.register(component_id, actor_addr).unwrap();
    }

    assert_eq!(registry.count().expect("Failed to get count"), 5);

    // Unregister middle component
    let middle_id = ComponentId::new("component-2");
    registry.unregister(&middle_id).unwrap();

    assert_eq!(registry.count().expect("Failed to get count"), 4);

    // Verify middle component is gone
    assert!(registry.lookup(&middle_id).is_err());

    // Verify other components still exist
    for i in [0, 1, 3, 4] {
        let component_id = ComponentId::new(format!("component-{}", i));
        let found = registry.lookup(&component_id).unwrap();
        assert_eq!(found.name().unwrap(), format!("component-{}", i));
    }
}
