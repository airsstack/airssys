//! ComponentActor messaging performance benchmarks.
//!
//! Validates:
//! - Direct message routing (MessageRouter)
//! - Request-response patterns (CorrelationTracker)
//! - Pub-sub broadcasting (MessageBroker)
//! - Throughput and concurrency
//!
//! Performance targets:
//! - Message latency: < 100μs P99
//! - Request-response: < 10ms P99
//! - Pub-sub fanout: < 1ms P99 (10 subscribers)
//! - Throughput: > 10,000 msg/sec
//!
//! CRITICAL: All benchmarks must have < 5% variance across 5 runs.

#![expect(clippy::unwrap_used, reason = "unwrap is acceptable in benchmark code")]

// Layer 1: Standard library imports
use std::hint::black_box;
use std::sync::Arc;
use std::time::Duration;

// Layer 2: Third-party crate imports
use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use tokio::sync::oneshot;
use uuid::Uuid;

// Layer 3: Internal module imports
use airssys_rt::broker::in_memory::InMemoryMessageBroker;
use airssys_rt::util::ActorAddress;
use airssys_wasm::actor::{
    ComponentMessage, ComponentRegistry, CorrelationTracker, MessageRouter,
    PendingRequest, RequestMessage, ResponseMessage,
};
use airssys_wasm::core::ComponentId;

// ============================================================================
// Category A: Direct Messaging Benchmarks (2 benchmarks)
// ============================================================================

/// Benchmark 1: MessageRouter construction
///
/// Measures MessageRouter::new() overhead
///
/// Target: < 100μs P99
fn bench_message_router_construction(c: &mut Criterion) {
    c.bench_function("message_router_construction", |b| {
        b.iter(|| {
            let registry = ComponentRegistry::new();
            let broker = Arc::new(InMemoryMessageBroker::<ComponentMessage>::new());

            let router = MessageRouter::new(registry, broker);

            black_box(router)
        });
    });
}

/// Benchmark 2: Message routing overhead (registry lookup only)
///
/// Measures MessageRouter::send_message() latency
///
/// Target: < 100μs P99
fn bench_message_routing_overhead(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("message_routing_overhead", |b| {
        b.to_async(&rt).iter(|| async {
            let registry = ComponentRegistry::new();
            let broker = Arc::new(InMemoryMessageBroker::<ComponentMessage>::new());
            let router = MessageRouter::new(registry.clone(), broker);

            // Register a component
            let target_id = ComponentId::new("target");
            let target_addr = ActorAddress::named("target-actor");
            registry.register(target_id.clone(), target_addr).unwrap();

            // Measure routing latency
            let message = ComponentMessage::HealthCheck;
            let result = router.send_message(&target_id, message).await;

            black_box(result)
        });
    });
}

// ============================================================================
// Category B: Request-Response Benchmarks (3 benchmarks)
// ============================================================================

/// Benchmark 3: CorrelationTracker construction
///
/// Measures CorrelationTracker::new() overhead
///
/// Target: < 10μs P99
fn bench_correlation_tracker_construction(c: &mut Criterion) {
    c.bench_function("correlation_tracker_construction", |b| {
        b.iter(|| {
            let tracker = CorrelationTracker::new();

            black_box(tracker)
        });
    });
}

/// Benchmark 4: Correlation tracking overhead (register + resolve)
///
/// Measures full correlation lifecycle
///
/// Target: < 50μs P99
fn bench_correlation_tracking_overhead(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("correlation_tracking_overhead", |b| {
        b.to_async(&rt).iter(|| async {
            let tracker = CorrelationTracker::new();

            let (tx, _rx) = oneshot::channel();
            let corr_id = Uuid::new_v4();
            let from = ComponentId::new("requester");
            let to = ComponentId::new("responder");

            let pending = PendingRequest {
                correlation_id: corr_id,
                response_tx: tx,
                requested_at: tokio::time::Instant::now(),
                timeout: Duration::from_secs(5),
                from: from.clone(),
                to: to.clone(),
            };

            // Register pending request
            tracker.register_pending(pending).await.ok();

            // Resolve with response
            let response = ResponseMessage::success(
                corr_id,
                to,
                from,
                vec![1, 2, 3],
            );

            let result = tracker.resolve(corr_id, response).await;

            black_box(result)
        });
    });
}

/// Benchmark 5: RequestMessage construction
///
/// Measures RequestMessage::new() overhead
///
/// Target: < 10μs P99
fn bench_request_message_construction(c: &mut Criterion) {
    c.bench_function("request_message_construction", |b| {
        b.iter(|| {
            let from = ComponentId::new("requester");
            let to = ComponentId::new("responder");
            let payload = vec![1, 2, 3, 4, 5];

            let request = RequestMessage::new(
                black_box(from),
                black_box(to),
                black_box(payload),
                black_box(5000),
            );

            black_box(request)
        });
    });
}

// ============================================================================
// Category C: Pub-Sub Broadcasting Benchmarks (3 benchmarks)
// ============================================================================

/// Benchmark 6: Pub-sub fanout (10 subscribers)
///
/// Measures message broadcast to 10 components
///
/// Target: < 1ms P99
fn bench_pubsub_fanout_10(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("pubsub_fanout_10", |b| {
        b.to_async(&rt).iter(|| async {
            let registry = ComponentRegistry::new();
            let broker = Arc::new(InMemoryMessageBroker::<ComponentMessage>::new());
            let router = MessageRouter::new(registry.clone(), broker);

            // Register 10 subscribers
            for i in 0..10 {
                let id = ComponentId::new(format!("subscriber-{}", i));
                let addr = ActorAddress::named(format!("subscriber-actor-{}", i));
                registry.register(id, addr).unwrap();
            }

            // Broadcast to all 10
            let message = ComponentMessage::HealthCheck;
            for i in 0..10 {
                let id = ComponentId::new(format!("subscriber-{}", i));
                let _ = router.send_message(&id, message.clone()).await;
            }

            black_box(())
        });
    });
}

/// Benchmark 7: Pub-sub fanout (100 subscribers)
///
/// Measures message broadcast to 100 components
///
/// Target: < 10ms P99
fn bench_pubsub_fanout_100(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("pubsub_fanout_100", |b| {
        b.to_async(&rt).iter(|| async {
            let registry = ComponentRegistry::new();
            let broker = Arc::new(InMemoryMessageBroker::<ComponentMessage>::new());
            let router = MessageRouter::new(registry.clone(), broker);

            // Register 100 subscribers
            for i in 0..100 {
                let id = ComponentId::new(format!("subscriber-{}", i));
                let addr = ActorAddress::named(format!("subscriber-actor-{}", i));
                registry.register(id, addr).unwrap();
            }

            // Broadcast to all 100
            let message = ComponentMessage::HealthCheck;
            for i in 0..100 {
                let id = ComponentId::new(format!("subscriber-{}", i));
                let _ = router.send_message(&id, message.clone()).await;
            }

            black_box(())
        });
    });
}

/// Benchmark 8: Registry subscription management (register 10 components)
///
/// Measures bulk registration overhead
///
/// Target: < 500μs P99
fn bench_subscription_management(c: &mut Criterion) {
    c.bench_function("subscription_management", |b| {
        b.iter(|| {
            let registry = ComponentRegistry::new();

            // Register 10 components
            for i in 0..10 {
                let id = ComponentId::new(format!("component-{}", i));
                let addr = ActorAddress::named(format!("actor-{}", i));
                black_box(registry.register(id, addr)).ok();
            }

            black_box(registry)
        });
    });
}

// ============================================================================
// Category D: Throughput Testing Benchmarks (2 benchmarks)
// ============================================================================

/// Benchmark 9: Sustained message throughput (1,000 messages)
///
/// Measures msg/sec throughput
///
/// Target: > 10,000 msg/sec
fn bench_sustained_message_throughput(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("sustained_throughput");
    group.throughput(Throughput::Elements(1000));

    group.bench_function("1000_messages", |b| {
        b.to_async(&rt).iter(|| async {
            let registry = ComponentRegistry::new();
            let broker = Arc::new(InMemoryMessageBroker::<ComponentMessage>::new());
            let router = MessageRouter::new(registry.clone(), broker);

            // Register target
            let target_id = ComponentId::new("target");
            let target_addr = ActorAddress::named("target-actor");
            registry.register(target_id.clone(), target_addr).unwrap();

            // Send 1,000 messages
            for _ in 0..1000 {
                let message = ComponentMessage::HealthCheck;
                let _ = router.send_message(&target_id, message).await;
            }

            black_box(())
        });
    });

    group.finish();
}

/// Benchmark 10: Concurrent senders (10 components sending simultaneously)
///
/// Measures concurrent routing performance
///
/// Target: < 10ms P99 for 10 concurrent senders
fn bench_concurrent_senders(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("concurrent_senders_10", |b| {
        b.to_async(&rt).iter(|| async {
            let registry = ComponentRegistry::new();
            let broker = Arc::new(InMemoryMessageBroker::<ComponentMessage>::new());
            let router = MessageRouter::new(registry.clone(), broker);

            // Register target
            let target_id = ComponentId::new("target");
            let target_addr = ActorAddress::named("target-actor");
            registry.register(target_id.clone(), target_addr).unwrap();

            // Spawn 10 concurrent senders
            let mut handles = Vec::new();
            for _ in 0..10 {
                let router_clone = router.clone();
                let target_clone = target_id.clone();

                let handle = tokio::spawn(async move {
                    let message = ComponentMessage::HealthCheck;
                    let _ = router_clone.send_message(&target_clone, message).await;
                });

                handles.push(handle);
            }

            // Wait for all senders
            for handle in handles {
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
        // Category A: Direct Messaging (2 benchmarks)
        bench_message_router_construction,
        bench_message_routing_overhead,

        // Category B: Request-Response (3 benchmarks)
        bench_correlation_tracker_construction,
        bench_correlation_tracking_overhead,
        bench_request_message_construction,

        // Category C: Pub-Sub Broadcasting (3 benchmarks)
        bench_pubsub_fanout_10,
        bench_pubsub_fanout_100,
        bench_subscription_management,

        // Category D: Throughput Testing (2 benchmarks)
        bench_sustained_message_throughput,
        bench_concurrent_senders,
}

criterion_main!(benches);
