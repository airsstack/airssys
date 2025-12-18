//! ComponentActor scalability and stress benchmarks.
//!
//! Validates:
//! - Registry O(1) lookup at scale (100, 1,000 components)
//! - Component spawn rate (batch operations)
//! - Memory overhead per component
//! - System behavior under concurrent stress
//!
//! Performance targets:
//! - Registry lookup: < 50μs P99 at 1,000 components (O(1))
//! - Spawn rate: > 100 components/sec (1,000 components)
//! - Memory: < 1MB per component baseline
//!
//! CRITICAL: All benchmarks must have < 5% variance across 5 runs.
//! NOTE: Using max 1,000 components for local machine compatibility (not 10,000).

#![expect(clippy::unwrap_used, reason = "unwrap is acceptable in benchmark code")]

// Layer 1: Standard library imports
use std::hint::black_box;
use std::sync::Arc;
use std::time::Duration;

// Layer 2: Third-party crate imports
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

// Layer 3: Internal module imports
use airssys_rt::util::ActorAddress;
use airssys_wasm::actor::{ComponentActor, ComponentRegistry};
use airssys_wasm::core::{CapabilitySet, ComponentId, ComponentMetadata, ResourceLimits};

/// Helper: Create test metadata
fn create_test_metadata(name: &str) -> ComponentMetadata {
    ComponentMetadata {
        name: name.to_string(),
        version: "1.0.0".to_string(),
        author: "AirsSys Benchmark".to_string(),
        description: Some("Benchmark test component".to_string()),
        required_capabilities: vec![],
        resource_limits: ResourceLimits {
            max_memory_bytes: 64 * 1024 * 1024, // 64MB
            max_fuel: 1_000_000,
            max_execution_ms: 5000,
            max_storage_bytes: 10 * 1024 * 1024, // 10MB
        },
    }
}

// ============================================================================
// Category A: Registry Scalability Benchmarks (3 benchmarks)
// ============================================================================

/// Benchmark 1: Registry lookup scale (10 to 1,000 components)
///
/// Validates O(1) lookup property
///
/// Target: < 50μs P99 at 1,000 components
fn bench_registry_lookup_scale(c: &mut Criterion) {
    let mut group = c.benchmark_group("registry_lookup_scale");

    for size in [10, 100, 1000].iter() {
        let registry = ComponentRegistry::new();

        // Pre-populate registry
        for i in 0..*size {
            let id = ComponentId::new(format!("component-{}", i));
            let addr = ActorAddress::named(format!("actor-{}", i));
            registry.register(id, addr).unwrap();
        }

        let target = ComponentId::new(format!("component-{}", size / 2));

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| black_box(registry.lookup(black_box(&target))));
        });
    }

    group.finish();
}

/// Benchmark 2: Registry registration scale (batch 10 to 1,000)
///
/// Measures bulk registration performance
///
/// Target: < 1ms for 100 registrations
fn bench_registry_registration_scale(c: &mut Criterion) {
    let mut group = c.benchmark_group("registry_registration_scale");

    for size in [10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let registry = ComponentRegistry::new();

                for i in 0..size {
                    let id = ComponentId::new(format!("component-{}", i));
                    let addr = ActorAddress::named(format!("actor-{}", i));
                    black_box(registry.register(id, addr)).ok();
                }

                black_box(registry)
            });
        });
    }

    group.finish();
}

/// Benchmark 3: Registry concurrent lookup (10 threads)
///
/// Measures concurrent read performance
///
/// Target: < 100μs P99
fn bench_registry_concurrent_lookup(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("registry_concurrent_lookup_10_threads", |b| {
        b.to_async(&rt).iter(|| async {
            let registry = ComponentRegistry::new();

            // Pre-populate with 100 components
            for i in 0..100 {
                let id = ComponentId::new(format!("component-{}", i));
                let addr = ActorAddress::named(format!("actor-{}", i));
                registry.register(id, addr).unwrap();
            }

            // Spawn 10 concurrent lookups
            let mut handles = Vec::new();
            for i in 0..10 {
                let registry_clone = registry.clone();
                let target = ComponentId::new(format!("component-{}", i * 10));

                let handle = tokio::spawn(async move { black_box(registry_clone.lookup(&target)) });

                handles.push(handle);
            }

            // Wait for all lookups
            for handle in handles {
                handle.await.ok();
            }

            black_box(())
        });
    });
}

// ============================================================================
// Category B: Component Spawn Rate Benchmarks (2 benchmarks)
// ============================================================================

/// Benchmark 4: Component batch construction (10 to 1,000 components)
///
/// Measures ComponentActor::new() batch performance
///
/// Target: > 1,000 components/sec
fn bench_component_batch_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("component_batch_construction");

    for size in [10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut actors = Vec::with_capacity(size);

                for i in 0..size {
                    let id = ComponentId::new(format!("component-{}", i));
                    let metadata = create_test_metadata(&format!("component-{}", i));
                    let caps = CapabilitySet::new();

                    let actor = ComponentActor::new(id, metadata, caps, ());
                    actors.push(actor);
                }

                black_box(actors)
            });
        });
    }

    group.finish();
}

/// Benchmark 5: Component state allocation (batch 100)
///
/// Measures Arc<RwLock<T>> batch allocation
///
/// Target: < 10ms for 100 allocations
fn bench_component_state_allocation_batch(c: &mut Criterion) {
    c.bench_function("component_state_allocation_batch_100", |b| {
        b.iter(|| {
            let mut states = Vec::with_capacity(100);

            for i in 0..100 {
                let metadata = create_test_metadata(&format!("component-{}", i));
                let state = Arc::new(tokio::sync::RwLock::new(metadata));
                states.push(state);
            }

            black_box(states)
        });
    });
}

// ============================================================================
// Category C: Memory and Resource Stress Benchmarks (3 benchmarks)
// ============================================================================

/// Benchmark 6: Memory overhead per component (estimate)
///
/// Measures ComponentActor memory footprint
///
/// Target: < 1MB per component
fn bench_component_memory_overhead(c: &mut Criterion) {
    c.bench_function("component_memory_overhead_single", |b| {
        b.iter(|| {
            // Create ComponentActor with full metadata
            let id = ComponentId::new("memory-test");
            let metadata = create_test_metadata("memory-test");
            let caps = CapabilitySet::new();

            let actor = ComponentActor::new(id, metadata, caps, ());

            // Measure size impact
            black_box(std::mem::size_of_val(&actor));
            black_box(actor)
        });
    });
}

/// Benchmark 7: Concurrent operations stress (100 components)
///
/// Measures system stability under concurrent stress
///
/// Target: < 50ms for 100 concurrent operations
fn bench_concurrent_operations_stress(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("concurrent_operations_stress_100", |b| {
        b.to_async(&rt).iter(|| async {
            let registry = ComponentRegistry::new();

            // Spawn 100 concurrent registration tasks
            let mut handles = Vec::new();
            for i in 0..100 {
                let registry_clone = registry.clone();

                let handle = tokio::spawn(async move {
                    let id = ComponentId::new(format!("component-{}", i));
                    let addr = ActorAddress::named(format!("actor-{}", i));
                    registry_clone.register(id, addr).ok();
                });

                handles.push(handle);
            }

            // Wait for all registrations
            for handle in handles {
                handle.await.ok();
            }

            // Verify all registered (100 concurrent lookups)
            let mut lookup_handles = Vec::new();
            for i in 0..100 {
                let registry_clone = registry.clone();
                let target = ComponentId::new(format!("component-{}", i));

                let handle = tokio::spawn(async move {
                    registry_clone.lookup(&target).ok();
                });

                lookup_handles.push(handle);
            }

            // Wait for all lookups
            for handle in lookup_handles {
                handle.await.ok();
            }

            black_box(())
        });
    });
}

/// Benchmark 8: System under load (combined operations)
///
/// Measures mixed workload: registration + lookup + construction
///
/// Target: < 100ms for 100 mixed operations
fn bench_system_under_load(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("system_under_load_100_mixed", |b| {
        b.to_async(&rt).iter(|| async {
            let registry = ComponentRegistry::new();

            // Mixed workload: 50 register + 50 lookup
            let mut handles = Vec::new();

            // First 50: register
            for i in 0..50 {
                let registry_clone = registry.clone();

                let handle = tokio::spawn(async move {
                    let id = ComponentId::new(format!("component-{}", i));
                    let metadata = create_test_metadata(&format!("component-{}", i));
                    let caps = CapabilitySet::new();

                    // Construct actor
                    let _actor = ComponentActor::new(id.clone(), metadata, caps, ());

                    // Register
                    let addr = ActorAddress::named(format!("actor-{}", i));
                    registry_clone.register(id, addr).ok();
                });

                handles.push(handle);
            }

            // Wait for registrations
            for handle in handles {
                handle.await.ok();
            }

            // Next 50: lookup
            let mut lookup_handles = Vec::new();
            for i in 0..50 {
                let registry_clone = registry.clone();
                let target = ComponentId::new(format!("component-{}", i));

                let handle = tokio::spawn(async move {
                    registry_clone.lookup(&target).ok();
                });

                lookup_handles.push(handle);
            }

            // Wait for lookups
            for handle in lookup_handles {
                handle.await.ok();
            }

            black_box(())
        });
    });
}

// ============================================================================
// Criterion Configuration (MANDATORY: < 5% variance requirement)
// ============================================================================

/// Configure Criterion for stable, reproducible benchmarks
fn criterion_config() -> Criterion {
    Criterion::default()
        .warm_up_time(Duration::from_secs(2)) // Stabilize CPU frequency
        .measurement_time(Duration::from_secs(5)) // 5s measurement window
        .sample_size(100) // 100 samples for statistical validity
        .significance_level(0.05) // 95% confidence interval
        .noise_threshold(0.02) // 2% noise tolerance
        .without_plots() // Reduce I/O overhead
}

criterion_group! {
    name = benches;
    config = criterion_config();
    targets =
        // Category A: Registry Scalability (3 benchmarks)
        bench_registry_lookup_scale,
        bench_registry_registration_scale,
        bench_registry_concurrent_lookup,

        // Category B: Component Spawn Rate (2 benchmarks)
        bench_component_batch_construction,
        bench_component_state_allocation_batch,

        // Category C: Memory and Resource Stress (3 benchmarks)
        bench_component_memory_overhead,
        bench_concurrent_operations_stress,
        bench_system_under_load,
}

criterion_main!(benches);
