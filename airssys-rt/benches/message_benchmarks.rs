//! Message Passing Benchmarks
//!
//! Measures baseline performance of message routing and delivery:
//! - Point-to-point message send/receive latency
//! - Sustained message throughput
//! - Broadcast to multiple actors (10 actors)
//! - Mailbox enqueue/dequeue operations

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
use criterion::{criterion_group, criterion_main, Criterion};
use serde::{Deserialize, Serialize};

// Layer 3: Internal module imports
use airssys_rt::broker::in_memory::InMemoryMessageBroker;
use airssys_rt::broker::MessageBroker;
use airssys_rt::mailbox::traits::MailboxSender;
use airssys_rt::mailbox::{AtomicMetrics, BoundedMailbox};
use airssys_rt::message::{Message, MessageEnvelope};
use airssys_rt::{Actor, ActorContext};

/// Test message for benchmarking
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestMessage {
    id: u64,
    payload: String,
}

impl Message for TestMessage {
    const MESSAGE_TYPE: &'static str = "TestMessage";
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

/// Benchmark: Point-to-point message send/receive
fn message_send_receive(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("message_send_receive", |b| {
        b.to_async(&rt).iter(|| async {
            let broker = InMemoryMessageBroker::<TestMessage>::new();
            let mut stream = broker.subscribe().await.unwrap();

            // Send message
            let msg = TestMessage {
                id: 1,
                payload: "test".to_string(),
            };
            let envelope = MessageEnvelope::new(msg);

            broker.publish(envelope.clone()).await.unwrap();

            // Receive message
            let received = stream.recv().await;

            black_box(received);
        });
    });
}

/// Benchmark: Sustained message throughput
fn message_throughput(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("message_throughput", |b| {
        b.to_async(&rt).iter(|| async {
            let broker = InMemoryMessageBroker::<TestMessage>::new();
            let mut stream = broker.subscribe().await.unwrap();

            // Send 100 messages
            for i in 0..100 {
                let msg = TestMessage {
                    id: i,
                    payload: format!("message_{i}"),
                };
                let envelope = MessageEnvelope::new(msg);
                broker.publish(envelope).await.unwrap();
            }

            // Receive all messages
            let mut count = 0;
            while count < 100 {
                if stream.recv().await.is_some() {
                    count += 1;
                }
            }

            black_box(count);
        });
    });
}

/// Benchmark: Broadcast to 10 actors
fn message_broadcast_small(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("message_broadcast_small", |b| {
        b.to_async(&rt).iter(|| async {
            let broker = InMemoryMessageBroker::<TestMessage>::new();

            // Create 10 subscribers
            let mut streams = Vec::with_capacity(10);
            for _ in 0..10 {
                let stream = broker.subscribe().await.unwrap();
                streams.push(stream);
            }

            // Broadcast message
            let msg = TestMessage {
                id: 1,
                payload: "broadcast".to_string(),
            };
            let envelope = MessageEnvelope::new(msg);

            broker.publish(envelope).await.unwrap();

            // Each subscriber receives the message
            for stream in &mut streams {
                let _ = stream.recv().await;
            }

            black_box(streams);
        });
    });
}

/// Benchmark: Mailbox enqueue/dequeue operations
fn mailbox_operations(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("mailbox_operations", |b| {
        b.to_async(&rt).iter(|| async {
            let (mailbox, sender) = BoundedMailbox::<TestMessage, AtomicMetrics>::new(1000);

            // Enqueue 100 messages
            for i in 0..100 {
                let msg = TestMessage {
                    id: i,
                    payload: format!("msg_{i}"),
                };
                let envelope = MessageEnvelope::new(msg);
                sender.send(envelope).await.unwrap();
            }

            black_box((mailbox, sender));
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
        message_send_receive,
        message_throughput,
        message_broadcast_small,
        mailbox_operations
}

criterion_main!(benches);
