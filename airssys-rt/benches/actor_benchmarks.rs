//! Actor System Benchmarks
//!
//! Measures baseline performance of actor lifecycle operations:
//! - Single actor spawn latency
//! - Batch actor spawn (10 actors)
//! - Message processing throughput

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

// Layer 1: Standard library imports
use std::fmt;
use std::hint::black_box;
use std::time::Duration;

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use criterion::{criterion_group, criterion_main, Criterion};
use serde::{Deserialize, Serialize};

// Layer 3: Internal module imports
use airssys_rt::broker::in_memory::InMemoryMessageBroker;
use airssys_rt::broker::MessageBroker;
use airssys_rt::util::ActorAddress;
use airssys_rt::{Actor, ActorContext, Message};

/// Simple test message for benchmarking
#[derive(Debug, Clone, Serialize, Deserialize)]
struct BenchMessage {
    value: u64,
}

impl Message for BenchMessage {
    const MESSAGE_TYPE: &'static str = "BenchMessage";
}

/// Simple error type for benchmarking
#[derive(Debug)]
struct BenchError;

impl fmt::Display for BenchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BenchError")
    }
}

impl std::error::Error for BenchError {}

/// Simple test actor that counts messages
struct CounterActor {
    count: u64,
}

impl CounterActor {
    fn new() -> Self {
        Self { count: 0 }
    }
}

#[async_trait]
impl Actor for CounterActor {
    type Message = BenchMessage;
    type Error = BenchError;

    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,
        _context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        self.count += message.value;
        Ok(())
    }
}

/// Benchmark: Spawn a single actor (setup overhead)
fn actor_spawn_single(c: &mut Criterion) {
    c.bench_function("actor_spawn_single", |b| {
        b.iter(|| {
            // Measure actor creation and context setup
            let actor = CounterActor::new();
            let address = ActorAddress::anonymous();
            let broker = InMemoryMessageBroker::<BenchMessage>::new();
            let context = ActorContext::new(address, broker);

            black_box((actor, context));
        });
    });
}

/// Benchmark: Spawn 10 actors in batch
fn actor_spawn_batch_small(c: &mut Criterion) {
    c.bench_function("actor_spawn_batch_small", |b| {
        b.iter(|| {
            let mut actors = Vec::with_capacity(10);

            // Create 10 actors with contexts
            for _ in 0..10 {
                let actor = CounterActor::new();
                let address = ActorAddress::anonymous();
                let broker = InMemoryMessageBroker::<BenchMessage>::new();
                let context = ActorContext::new(address, broker);
                actors.push((actor, context));
            }

            black_box(actors);
        });
    });
}

/// Benchmark: Message processing throughput
fn actor_message_throughput(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("actor_message_throughput", |b| {
        b.to_async(&rt).iter(|| async {
            let mut actor = CounterActor::new();
            let address = ActorAddress::anonymous();
            let broker = InMemoryMessageBroker::<BenchMessage>::new();
            let mut context = ActorContext::new(address, broker);

            // Process 100 messages
            for i in 0..100 {
                let msg = BenchMessage { value: i };
                let _ = actor.handle_message(msg, &mut context).await;
            }

            black_box(actor);
        });
    });
}

/// Configure criterion for resource-conscious benchmarking
fn configure_criterion() -> Criterion {
    Criterion::default()
        .sample_size(30) // Reduced for resource constraints
        .measurement_time(Duration::from_secs(5)) // Shorter measurement
        .warm_up_time(Duration::from_secs(2)) // Shorter warm-up
        .without_plots() // Save disk I/O
}

criterion_group! {
    name = benches;
    config = configure_criterion();
    targets =
        actor_spawn_single,
        actor_spawn_batch_small,
        actor_message_throughput
}

criterion_main!(benches);
