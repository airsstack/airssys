#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

//! Fire-and-Forget Messaging Performance Benchmarks (WASM-TASK-006 Phase 2 Task 2.3)
//!
//! Lightweight benchmarks validating fire-and-forget messaging performance.
//! Designed for quick execution with minimal resource usage.
//!
//! # Benchmarks (5-6 essential only)
//!
//! - **Overhead Breakdown**:
//!   - `bench_host_validation_overhead` (~50ns target)
//!   - `bench_broker_publish_overhead` (~211ns target)
//!   - `bench_total_fire_and_forget_latency` (~280ns target)
//!
//! - **Throughput**:
//!   - `bench_throughput_single_sender` (>10,000 msg/sec target)
//!   - `bench_sustained_throughput` (100 messages)
//!
//! # Performance Targets (from ADR-WASM-009)
//!
//! | Metric | Target | Source |
//! |--------|--------|--------|
//! | Host Validation | ~50ns | ADR-WASM-009 |
//! | MessageBroker Routing | ~211ns | RT-TASK-008 |
//! | Total Latency | ~280ns | ADR-WASM-009 |
//! | Throughput | >10,000 msg/sec | ADR-WASM-009 |
//!
//! # Resource Optimization
//!
//! - 10 samples (not 100)
//! - 1 second measurement window (not 5s)
//! - Small sample sizes (10-50 messages)
//! - Total benchmark suite: <30 seconds

// Layer 1: Standard library imports
use std::hint::black_box;
use std::sync::Arc;
use std::time::Duration;

// Layer 2: Third-party crate imports
use criterion::{criterion_group, criterion_main, Criterion, Throughput};

// Layer 3: Internal module imports
use airssys_rt::broker::in_memory::InMemoryMessageBroker;
use airssys_rt::MessageBroker;
use airssys_rt::MessageEnvelope;
use airssys_wasm::actor::ComponentMessage;
use airssys_wasm::core::{
    bridge::{HostCallContext, HostFunction},
    Capability, CapabilitySet, ComponentId, MulticodecPrefix, TopicPattern,
};
use airssys_wasm::host_system::{CorrelationTracker, TimeoutHandler};
use airssys_wasm::messaging::MessagingService;
use airssys_wasm::runtime::{create_host_context, SendMessageHostFunction};

// ============================================================================
// Helper Functions
// ============================================================================

/// Helper function to create a MessagingService for benchmarks
fn create_messaging_service() -> Arc<MessagingService> {
    use airssys_rt::broker::InMemoryMessageBroker;

    let correlation_tracker = Arc::new(CorrelationTracker::new());
    let timeout_handler = Arc::new(TimeoutHandler::new());
    let broker = Arc::new(InMemoryMessageBroker::new());
    Arc::new(MessagingService::new(broker, correlation_tracker, timeout_handler))
}

/// Create encoded args for send-message host function.
///
/// Format: `[target_len: u32 LE][target_bytes][message_bytes]`
fn encode_send_args(target: &str, message: &[u8]) -> Vec<u8> {
    let mut args = (target.len() as u32).to_le_bytes().to_vec();
    args.extend_from_slice(target.as_bytes());
    args.extend_from_slice(message);
    args
}

/// Create a message with multicodec prefix.
fn create_prefixed_message(payload: &[u8]) -> Vec<u8> {
    MulticodecPrefix::Borsh.create_message(payload)
}

/// Create a context with full messaging capabilities.
fn create_messaging_context(component_id: &str) -> HostCallContext {
    let mut caps = CapabilitySet::new();
    caps.grant(Capability::Messaging(TopicPattern::new("*")));
    create_host_context(ComponentId::new(component_id), caps)
}

// ============================================================================
// Benchmark 1: Host Validation Overhead (~50ns target)
// ============================================================================

/// Measures the overhead of SendMessageHostFunction argument parsing
/// and multicodec validation (not including broker publish).
///
/// Target: ~50ns per validation
fn bench_host_validation_overhead(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("fire_and_forget_host_validation", |b| {
        b.to_async(&rt).iter(|| async {
            let messaging = create_messaging_service();
            let func = SendMessageHostFunction::new(Arc::clone(&messaging));
            let context = create_messaging_context("sender");

            // Create small message with Borsh prefix
            let payload = b"test";
            let message = create_prefixed_message(payload);
            let args = encode_send_args("receiver", &message);

            // Execute (includes validation + broker publish)
            let result = func.execute(&context, args).await;
            black_box(result)
        });
    });
}

// ============================================================================
// Benchmark 2: Broker Publish Overhead (~211ns target)
// ============================================================================

/// Measures MessageBroker publish latency in isolation.
///
/// Target: ~211ns per publish (from RT-TASK-008)
fn bench_broker_publish_overhead(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("fire_and_forget_broker_publish", |b| {
        b.to_async(&rt).iter(|| async {
            let broker = Arc::new(InMemoryMessageBroker::<ComponentMessage>::new());

            let message = ComponentMessage::InterComponent {
                sender: ComponentId::new("sender"),
                to: ComponentId::new("receiver"),
                payload: vec![0x50, 0x01, 1, 2, 3, 4], // Borsh prefix + data
            };

            let envelope = MessageEnvelope::new(message);
            let result = broker.publish(envelope).await;
            black_box(result)
        });
    });
}

// ============================================================================
// Benchmark 3: Total Fire-and-Forget Latency (~280ns target)
// ============================================================================

/// Measures end-to-end fire-and-forget latency:
/// Host validation + multicodec parsing + broker publish.
///
/// Target: ~280ns total latency (50ns + 211ns + overhead)
fn bench_total_fire_and_forget_latency(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Pre-create messaging service for more realistic benchmark
    let messaging = create_messaging_service();

    c.bench_function("fire_and_forget_total_latency", |b| {
        b.to_async(&rt).iter(|| {
            let messaging_clone = Arc::clone(&messaging);
            async move {
                let func = SendMessageHostFunction::new(messaging_clone);
                let context = create_messaging_context("component-a");

                let payload = b"fire-and-forget message payload";
                let message = create_prefixed_message(payload);
                let args = encode_send_args("component-b", &message);

                let result = func.execute(&context, args).await;
                black_box(result)
            }
        });
    });
}

// ============================================================================
// Benchmark 4: Single Sender Throughput (>10,000 msg/sec target)
// ============================================================================

/// Measures throughput with a single sender sending 50 messages.
///
/// Target: >10,000 msg/sec
fn bench_throughput_single_sender(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("fire_and_forget_throughput");
    group.throughput(Throughput::Elements(50));

    group.bench_function("single_sender_50_msgs", |b| {
        b.to_async(&rt).iter(|| async {
            let messaging = create_messaging_service();
            let func = SendMessageHostFunction::new(Arc::clone(&messaging));
            let context = create_messaging_context("high-throughput-sender");

            // Send 50 messages (reduced from 1000 for resource constraints)
            for i in 0..50 {
                let payload = format!("msg-{i}").into_bytes();
                let message = create_prefixed_message(&payload);
                let args = encode_send_args("receiver", &message);

                let result = func.execute(&context, args.clone()).await;
                black_box(result.ok());
            }

            black_box(())
        });
    });

    group.finish();
}

// ============================================================================
// Benchmark 5: Sustained Throughput (100 messages)
// ============================================================================

/// Measures sustained throughput with 100 messages.
/// Reduced from 1000 for resource constraints.
///
/// Target: >10,000 msg/sec sustained
fn bench_sustained_throughput(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("fire_and_forget_sustained");
    group.throughput(Throughput::Elements(100));

    group.bench_function("sustained_100_msgs", |b| {
        b.to_async(&rt).iter(|| async {
            let messaging = create_messaging_service();
            let func = SendMessageHostFunction::new(Arc::clone(&messaging));
            let context = create_messaging_context("sustained-sender");

            // Send 100 messages (reduced from 1000 for resource constraints)
            for _ in 0..100 {
                let payload = b"sustained-message";
                let message = create_prefixed_message(payload);
                let args = encode_send_args("sustained-receiver", &message);

                let result = func.execute(&context, args.clone()).await;
                black_box(result.ok());
            }

            black_box(())
        });
    });

    group.finish();
}

// ============================================================================
// Criterion Configuration (LIGHTWEIGHT for resource constraints)
// ============================================================================

/// Configure Criterion for fast, lightweight benchmarks.
///
/// Optimized for limited local resources:
/// - 10 samples (not 100)
/// - 1 second measurement window (not 5s)
/// - 500ms warmup (not 2s)
/// - Total benchmark time: <30 seconds
fn criterion_config() -> Criterion {
    Criterion::default()
        .warm_up_time(Duration::from_millis(500)) // Quick warmup
        .measurement_time(Duration::from_secs(1)) // 1s measurement window
        .sample_size(10) // 10 samples for speed
        .significance_level(0.05) // 95% confidence
        .noise_threshold(0.05) // 5% noise tolerance (relaxed for speed)
        .without_plots() // Reduce I/O overhead
}

// ============================================================================
// Benchmark Registration
// ============================================================================

criterion_group! {
    name = fire_and_forget_benches;
    config = criterion_config();
    targets =
        // Overhead Breakdown (3 benchmarks)
        bench_host_validation_overhead,
        bench_broker_publish_overhead,
        bench_total_fire_and_forget_latency,

        // Throughput (2 benchmarks)
        bench_throughput_single_sender,
        bench_sustained_throughput,
}

criterion_main!(fire_and_forget_benches);
