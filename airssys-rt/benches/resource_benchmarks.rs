//! Resource Usage Benchmarks
//!
//! Measures baseline resource consumption:
//! - Memory footprint per actor (incremental: 1 → 10 → 50)
//! - Memory comparison: bounded vs unbounded mailboxes

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(dead_code)]
#![allow(unused_imports)]

// Layer 1: Standard library imports
use std::fmt;
use std::hint::black_box;
use std::time::Duration;

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use serde::{Deserialize, Serialize};

// Layer 3: Internal module imports
use airssys_rt::broker::in_memory::InMemoryMessageBroker;
use airssys_rt::broker::MessageBroker;
use airssys_rt::mailbox::{AtomicMetrics, BoundedMailbox, UnboundedMailbox};
use airssys_rt::util::ActorAddress;
use airssys_rt::{Actor, ActorContext, Message};

/// Test message
#[derive(Debug, Clone, Serialize, Deserialize)]
struct BenchMessage {
    value: u64,
}

impl Message for BenchMessage {
    const MESSAGE_TYPE: &'static str = "BenchMessage";
}

/// Simple error type
#[derive(Debug)]
struct BenchError;

impl fmt::Display for BenchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BenchError")
    }
}

impl std::error::Error for BenchError {}

/// Test actor
struct TestActor {
    state: u64,
}

impl TestActor {
    fn new() -> Self {
        Self { state: 0 }
    }
}

#[async_trait]
impl Actor for TestActor {
    type Message = BenchMessage;
    type Error = BenchError;

    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,
        _context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        self.state += message.value;
        Ok(())
    }
}

/// Benchmark: Memory per actor (incremental scaling)
fn memory_per_actor_baseline(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_per_actor");

    // Test with 1, 10, and 50 actors
    for actor_count in [1, 10, 50] {
        group.bench_with_input(
            BenchmarkId::from_parameter(actor_count),
            &actor_count,
            |b, &count| {
                b.iter(|| {
                    let mut actors = Vec::with_capacity(count);

                    // Create N actors with contexts
                    for i in 0..count {
                        let actor = TestActor::new();
                        let address = ActorAddress::named(format!("actor-{i}"));
                        let broker = InMemoryMessageBroker::<BenchMessage>::new();
                        let context = ActorContext::new(address, broker);
                        actors.push((actor, context));
                    }

                    black_box(actors);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Mailbox memory comparison
fn mailbox_memory_comparison(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("mailbox_memory");

    // Bounded mailbox
    group.bench_function("bounded_mailbox_100", |b| {
        b.to_async(&rt).iter(|| async {
            let mailboxes: Vec<_> = (0..10)
                .map(|_| BoundedMailbox::<BenchMessage, _>::new(100))
                .collect();

            black_box(mailboxes);
        });
    });

    // Unbounded mailbox
    group.bench_function("unbounded_mailbox", |b| {
        b.to_async(&rt).iter(|| async {
            let mailboxes: Vec<_> = (0..10)
                .map(|_| UnboundedMailbox::<BenchMessage, AtomicMetrics>::new())
                .collect();

            black_box(mailboxes);
        });
    });

    group.finish();
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
        memory_per_actor_baseline,
        mailbox_memory_comparison
}

criterion_main!(benches);
