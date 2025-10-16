//! Supervision Benchmarks
//!
//! Measures baseline performance of supervision operations:
//! - Child spawn via builder pattern
//! - Restart strategies comparison (OneForOne, OneForAll, RestForOne)
//! - Small supervision tree operations

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

// Layer 1: Standard library imports
use std::hint::black_box;
use std::time::Duration;

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use criterion::{criterion_group, criterion_main, Criterion};

// Layer 3: Internal module imports
use airssys_rt::monitoring::NoopMonitor;
use airssys_rt::supervisor::{Child, OneForAll, OneForOne, RestForOne, SupervisorNode};

/// Simple test child for benchmarking
#[derive(Clone)]
struct BenchChild {
    #[allow(dead_code)]
    id: u32,
}

impl BenchChild {
    fn new(id: u32) -> Self {
        Self { id }
    }
}

#[async_trait]
impl Child for BenchChild {
    type Error = std::io::Error;

    async fn start(&mut self) -> Result<(), Self::Error> {
        // Minimal work to simulate child start
        Ok(())
    }

    async fn stop(&mut self, _timeout: Duration) -> Result<(), Self::Error> {
        // Minimal work to simulate child stop
        Ok(())
    }
}

/// Benchmark: Child spawn via builder
fn supervisor_child_spawn(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");

    c.bench_function("supervisor_child_spawn", |b| {
        b.to_async(&rt).iter(|| async {
            let mut supervisor: SupervisorNode<OneForOne, BenchChild, _> =
                SupervisorNode::new(OneForOne, NoopMonitor::new());

            // Spawn single child using builder
            let child_id = supervisor
                .child("bench-child")
                .factory(|| BenchChild::new(1))
                .spawn()
                .await
                .expect("Failed to spawn child");

            black_box(child_id);
        });
    });
}

/// Benchmark: Compare restart strategies (spawn overhead)
fn supervisor_restart_strategy(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");

    // OneForOne strategy
    c.bench_function("supervisor_strategy_one_for_one", |b| {
        b.to_async(&rt).iter(|| async {
            let mut supervisor: SupervisorNode<OneForOne, BenchChild, _> =
                SupervisorNode::new(OneForOne, NoopMonitor::new());

            // Spawn child with OneForOne strategy
            let child_id = supervisor
                .child("test-child")
                .factory(|| BenchChild::new(1))
                .spawn()
                .await
                .expect("Failed to spawn");

            black_box(child_id);
        });
    });

    // OneForAll strategy
    c.bench_function("supervisor_strategy_one_for_all", |b| {
        b.to_async(&rt).iter(|| async {
            let mut supervisor: SupervisorNode<OneForAll, BenchChild, _> =
                SupervisorNode::new(OneForAll, NoopMonitor::new());

            // Spawn 3 children with OneForAll strategy
            let child_ids = supervisor
                .children()
                .child("child-1", || BenchChild::new(1))
                .child("child-2", || BenchChild::new(2))
                .child("child-3", || BenchChild::new(3))
                .spawn_all()
                .await
                .expect("Failed to spawn children");

            black_box(child_ids);
        });
    });

    // RestForOne strategy
    c.bench_function("supervisor_strategy_rest_for_one", |b| {
        b.to_async(&rt).iter(|| async {
            let mut supervisor: SupervisorNode<RestForOne, BenchChild, _> =
                SupervisorNode::new(RestForOne, NoopMonitor::new());

            // Spawn 3 children with RestForOne strategy
            let child_ids = supervisor
                .children()
                .child("child-1", || BenchChild::new(1))
                .child("child-2", || BenchChild::new(2))
                .child("child-3", || BenchChild::new(3))
                .spawn_all()
                .await
                .expect("Failed to spawn children");

            black_box(child_ids);
        });
    });
}

/// Benchmark: Small supervision tree operations
fn supervision_tree_small(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("supervision_tree_small", |b| {
        b.to_async(&rt).iter(|| async {
            // Create parent supervisor
            let mut parent: SupervisorNode<OneForOne, BenchChild, _> =
                SupervisorNode::new(OneForOne, NoopMonitor::new());

            // Spawn 3 children (width = 3)
            let _ = parent
                .children()
                .child("child-1", || BenchChild::new(1))
                .child("child-2", || BenchChild::new(2))
                .child("child-3", || BenchChild::new(3))
                .spawn_all()
                .await
                .unwrap();

            black_box(parent);
        });
    });
}

/// Configure criterion for resource-conscious benchmarking
fn configure_criterion() -> Criterion {
    Criterion::default()
        .sample_size(30)
        .measurement_time(Duration::from_secs(5))
        .warm_up_time(Duration::from_secs(2))
        .without_plots()
}

criterion_group! {
    name = benches;
    config = configure_criterion();
    targets =
        supervisor_child_spawn,
        supervisor_restart_strategy,
        supervision_tree_small
}

criterion_main!(benches);
