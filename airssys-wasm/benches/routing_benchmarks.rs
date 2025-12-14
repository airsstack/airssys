//! Performance benchmarks for actor address routing.
//!
//! Benchmarks verify <500ns routing latency target from ADR-WASM-009.

#![expect(clippy::unwrap_used, reason = "unwrap is acceptable in benchmark code")]

// Layer 1: Standard library imports
use std::hint::black_box;

// Layer 2: Third-party crate imports
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};

// Layer 3: Internal module imports
use airssys_wasm::actor::ComponentRegistry;
use airssys_wasm::core::ComponentId;
use airssys_rt::util::ActorAddress;

/// Benchmark: Registry lookup performance (O(1) target)
fn bench_lookup_performance(c: &mut Criterion) {
    let registry = ComponentRegistry::new();
    
    // Populate with 100 components
    for i in 0..100 {
        let component_id = ComponentId::new(format!("component-{}", i));
        let actor_addr = ActorAddress::named(format!("actor-{}", i));
        registry.register(component_id, actor_addr).unwrap();
    }

    let target = ComponentId::new("component-50");

    c.bench_function("registry_lookup", |b| {
        b.iter(|| {
            registry.lookup(black_box(&target))
        });
    });
}

/// Benchmark: Registry lookup with varying number of registered components
fn bench_lookup_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("registry_lookup_scalability");

    for size in [10, 100, 1000, 10000].iter() {
        let registry = ComponentRegistry::new();
        
        // Populate with components
        for i in 0..*size {
            let component_id = ComponentId::new(format!("component-{}", i));
            let actor_addr = ActorAddress::named(format!("actor-{}", i));
            registry.register(component_id, actor_addr).unwrap();
        }

        let target = ComponentId::new(format!("component-{}", size / 2));

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                registry.lookup(black_box(&target))
            });
        });
    }
    
    group.finish();
}

/// Benchmark: Registry registration performance
fn bench_registration_performance(c: &mut Criterion) {
    c.bench_function("registry_registration", |b| {
        let registry = ComponentRegistry::new();
        let mut counter = 0;
        
        b.iter(|| {
            let component_id = ComponentId::new(format!("component-{}", counter));
            let actor_addr = ActorAddress::named(format!("actor-{}", counter));
            registry.register(black_box(component_id), black_box(actor_addr)).unwrap();
            counter += 1;
        });
    });
}

/// Benchmark: Concurrent registry operations
fn bench_concurrent_registry_access(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    let registry = ComponentRegistry::new();
    
    // Pre-populate with components
    for i in 0..100 {
        let component_id = ComponentId::new(format!("component-{}", i));
        let actor_addr = ActorAddress::named(format!("actor-{}", i));
        registry.register(component_id, actor_addr).unwrap();
    }

    c.bench_function("concurrent_registry_lookups", |b| {
        b.to_async(&rt).iter(|| async {
            let mut handles = Vec::new();
            
            for i in 0..10 {
                let registry_clone = registry.clone();
                let target = ComponentId::new(format!("component-{}", i * 10));
                
                let handle = tokio::spawn(async move {
                    registry_clone.lookup(&target).unwrap();
                });
                
                handles.push(handle);
            }
            
            for handle in handles {
                handle.await.unwrap();
            }
        });
    });
}

/// Benchmark: ComponentExists check performance
fn bench_component_exists(c: &mut Criterion) {
    let registry = ComponentRegistry::new();
    
    // Populate with components
    for i in 0..100 {
        let component_id = ComponentId::new(format!("component-{}", i));
        let actor_addr = ActorAddress::named(format!("actor-{}", i));
        registry.register(component_id, actor_addr).unwrap();
    }

    let existing = ComponentId::new("component-50");
    let nonexistent = ComponentId::new("nonexistent");

    let mut group = c.benchmark_group("component_exists");
    
    group.bench_function("exists", |b| {
        b.iter(|| {
            registry.lookup(black_box(&existing)).is_ok()
        });
    });

    group.bench_function("not_exists", |b| {
        b.iter(|| {
            registry.lookup(black_box(&nonexistent)).is_err()
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_lookup_performance,
    bench_lookup_scalability,
    bench_registration_performance,
    bench_concurrent_registry_access,
    bench_component_exists,
);
criterion_main!(benches);
