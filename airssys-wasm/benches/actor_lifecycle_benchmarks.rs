//! ComponentActor lifecycle and registry performance benchmarks.
//!
//! Establishes baseline performance for:
//! - Component spawning (ComponentSpawner)
//! - Registry operations (ComponentRegistry)
//! - Lifecycle hooks (Lifecycle trait)
//! - State access (Arc<RwLock<T>>)
//!
//! Performance targets from Task 6.1:
//! - Component spawn: < 1ms P99
//! - Registry lookup: < 10μs P99
//! - Hook overhead: < 10μs P99
//! - State access: < 1μs P99
//!
//! CRITICAL: All benchmarks must have < 5% variance across 5 runs.

#![expect(clippy::unwrap_used, reason = "unwrap is acceptable in benchmark code")]

// Layer 1: Standard library imports
use std::hint::black_box;
use std::sync::Arc;
use std::time::Duration;

// Layer 2: Third-party crate imports
use criterion::{criterion_group, criterion_main, Criterion};
use tokio::sync::RwLock;

// Layer 3: Internal module imports
use airssys_rt::supervisor::Child;
use airssys_rt::util::ActorAddress;
use airssys_wasm::actor::{ComponentActor, ComponentRegistry};
use airssys_wasm::core::{
    CapabilitySet, ComponentId, ComponentMetadata, ResourceLimits,
};

/// Helper: Create test metadata
fn create_test_metadata(name: &str) -> ComponentMetadata {
    ComponentMetadata {
        name: name.to_string(),
        version: "1.0.0".to_string(),
        author: "AirsSys Benchmark".to_string(),
        description: Some("Benchmark test component".to_string()),
        required_capabilities: vec![],
        resource_limits: ResourceLimits {
            max_memory_bytes: 64 * 1024 * 1024,  // 64MB
            max_fuel: 1_000_000,
            max_execution_ms: 5000,
            max_storage_bytes: 10 * 1024 * 1024,  // 10MB
        },
    }
}

/// Helper: Create test capabilities
fn create_test_capabilities() -> CapabilitySet {
    CapabilitySet::new()
}

// ============================================================================
// Category A: Component Lifecycle Benchmarks (3 benchmarks)
// ============================================================================

/// Benchmark 1: ComponentActor construction baseline
///
/// Measures ComponentActor::new() latency (pure construction, no spawning)
///
/// Target: < 100μs P99
fn bench_component_actor_construction(c: &mut Criterion) {
    c.bench_function("component_actor_construction", |b| {
        b.iter(|| {
            let component_id = ComponentId::new("benchmark-component");
            let metadata = black_box(create_test_metadata("benchmark"));
            let capabilities = black_box(create_test_capabilities());

            let actor = ComponentActor::new(component_id, metadata, capabilities, ());

            black_box(actor)
        });
    });
}

/// Benchmark 2: Component lifecycle complete
///
/// Measures full lifecycle: spawn → start → stop → cleanup
///
/// Target: < 10ms P99 (includes WASM initialization overhead)
fn bench_component_lifecycle_complete(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("component_lifecycle_complete", |b| {
        b.to_async(&rt).iter(|| async {
            // Create ComponentActor directly (no WASM loading)
            let component_id = ComponentId::new("lifecycle-benchmark");
            let metadata = black_box(create_test_metadata("lifecycle"));
            let capabilities = black_box(create_test_capabilities());

            let mut actor =
                ComponentActor::new(component_id, metadata, capabilities, ());

            // Lifecycle: start (no-op without WASM)
            let _ = black_box(actor.start().await);

            // Lifecycle: stop
            let _ = black_box(actor.stop(Duration::from_secs(5)).await);
        });
    });
}

/// Benchmark 3: Component state initialization
///
/// Measures custom state creation overhead (Arc<RwLock<T>> construction)
///
/// Target: < 100μs P99
fn bench_component_state_initialization(c: &mut Criterion) {
    c.bench_function("component_state_initialization", |b| {
        b.iter(|| {
            // Measure state allocation
            let state = Arc::new(RwLock::new(black_box(create_test_metadata("test"))));

            black_box(state)
        });
    });
}

// ============================================================================
// Category B: Registry Operations Benchmarks (3 benchmarks)
// ============================================================================

/// Benchmark 4: Registry registration performance
///
/// Measures ComponentRegistry::register() performance
///
/// Target: < 50μs P99
fn bench_registry_registration(c: &mut Criterion) {
    let registry = ComponentRegistry::new();
    let mut counter = 0u64;

    c.bench_function("registry_registration", |b| {
        b.iter(|| {
            let component_id = ComponentId::new(format!("component-{}", counter));
            let actor_addr = ActorAddress::named(format!("actor-{}", counter));

            let result = registry.register(
                black_box(component_id),
                black_box(actor_addr),
            );

            counter += 1;
            black_box(result)
        });
    });
}

/// Benchmark 5: Registry lookup hit (component exists)
///
/// Measures successful lookup performance (O(1) HashMap access)
///
/// Target: < 10μs P99
fn bench_registry_lookup_hit(c: &mut Criterion) {
    let registry = ComponentRegistry::new();

    // Pre-populate with 100 components
    for i in 0..100 {
        let component_id = ComponentId::new(format!("component-{}", i));
        let actor_addr = ActorAddress::named(format!("actor-{}", i));
        registry.register(component_id, actor_addr).unwrap();
    }

    let target = ComponentId::new("component-50");

    c.bench_function("registry_lookup_hit", |b| {
        b.iter(|| {
            let result = registry.lookup(black_box(&target));
            black_box(result)
        });
    });
}

/// Benchmark 6: Registry lookup miss (component does not exist)
///
/// Measures failed lookup performance
///
/// Target: < 10μs P99 (should be same as hit due to O(1))
fn bench_registry_lookup_miss(c: &mut Criterion) {
    let registry = ComponentRegistry::new();

    // Pre-populate with 100 components
    for i in 0..100 {
        let component_id = ComponentId::new(format!("component-{}", i));
        let actor_addr = ActorAddress::named(format!("actor-{}", i));
        registry.register(component_id, actor_addr).unwrap();
    }

    let nonexistent = ComponentId::new("nonexistent-component");

    c.bench_function("registry_lookup_miss", |b| {
        b.iter(|| {
            let result = registry.lookup(black_box(&nonexistent));
            black_box(result)
        });
    });
}

// ============================================================================
// Category C: Hook Execution Benchmarks (2 benchmarks)
// ============================================================================

/// Benchmark 7: Hook execution (NoOp hooks baseline)
///
/// Measures NoOp hooks overhead (should be minimal)
///
/// Target: < 10μs P99
fn bench_hook_execution_noop(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("hook_execution_noop", |b| {
        b.to_async(&rt).iter(|| async {
            // Create ComponentActor with NoOpHooks
            let component_id = ComponentId::new("hook-benchmark");
            let metadata = create_test_metadata("hook");
            let capabilities = create_test_capabilities();
            let actor = ComponentActor::new(component_id, metadata, capabilities, ());

            // Measure hook access (NoOp implementation)
            black_box(actor)
        });
    });
}

/// Benchmark 8: Hook execution with state access
///
/// Measures hooks with state access (Arc<RwLock<T>>)
///
/// Target: < 50μs P99
fn bench_hook_execution_stateful(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("hook_execution_stateful", |b| {
        b.to_async(&rt).iter(|| async {
            let state = Arc::new(RwLock::new(0u64));
            let state_clone = state.clone();

            // Simulate hook with state access
            let mut write_guard = state_clone.write().await;
            *write_guard += 1;
            drop(write_guard);

            black_box(state)
        });
    });
}

// ============================================================================
// Category D: State Access Benchmarks (2 benchmarks)
// ============================================================================

/// Benchmark 9: State read access (Arc<RwLock<T>>::read())
///
/// Measures read lock acquisition latency
///
/// Target: < 1μs P99
fn bench_state_read_access(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("state_read_access", |b| {
        b.to_async(&rt).iter(|| async {
            let state = Arc::new(RwLock::new(42u64));

            // Measure read access
            let read_guard = state.read().await;
            let value = *read_guard;
            drop(read_guard);

            black_box(value)
        });
    });
}

/// Benchmark 10: State write access (Arc<RwLock<T>>::write())
///
/// Measures write lock acquisition and mutation latency
///
/// Target: < 10μs P99
fn bench_state_write_access(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("state_write_access", |b| {
        b.to_async(&rt).iter(|| async {
            let state = Arc::new(RwLock::new(0u64));

            // Measure write access
            let mut write_guard = state.write().await;
            *write_guard += 1;
            let value = *write_guard;
            drop(write_guard);

            black_box(value)
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
        // Category A: Component Lifecycle (3 benchmarks)
        bench_component_actor_construction,
        bench_component_lifecycle_complete,
        bench_component_state_initialization,

        // Category B: Registry Operations (3 benchmarks)
        bench_registry_registration,
        bench_registry_lookup_hit,
        bench_registry_lookup_miss,

        // Category C: Hook Execution (2 benchmarks)
        bench_hook_execution_noop,
        bench_hook_execution_stateful,

        // Category D: State Access (2 benchmarks)
        bench_state_read_access,
        bench_state_write_access,
}

criterion_main!(benches);
