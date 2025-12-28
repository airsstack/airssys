#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

//! Benchmarks for messaging module.
//!
//! This benchmark suite measures the performance of inter-component
//! messaging operations including fire-and-forget, request-response,
//! and topic-based pub-sub patterns.
//!
//! **Note:** This is a stub placeholder. Actual benchmarks will be
//! implemented in Phase 2 when messaging functionality is fully developed.

use airssys_wasm::actor::message::correlation_tracker::CorrelationTracker;
use airssys_wasm::messaging::{FireAndForget, MulticodecCodec, RequestResponse, ResponseRouter};
use criterion::{criterion_group, criterion_main, Criterion};
use std::sync::Arc;

fn benchmark_fire_and_forget_creation(c: &mut Criterion) {
    c.bench_function("fire_and_forget_creation", |b| {
        b.iter(|| {
            FireAndForget::new();
        });
    });
}

fn benchmark_request_response_creation(c: &mut Criterion) {
    c.bench_function("request_response_creation", |b| {
        b.iter(|| {
            RequestResponse::new();
        });
    });
}

fn benchmark_codec_creation(c: &mut Criterion) {
    c.bench_function("codec_creation", |b| {
        b.iter(|| {
            MulticodecCodec::new();
        });
    });
}

fn benchmark_router_creation(c: &mut Criterion) {
    c.bench_function("router_creation", |b| {
        let tracker = Arc::new(CorrelationTracker::new());
        b.iter(|| {
            ResponseRouter::new(Arc::clone(&tracker));
        });
    });
}

criterion_group!(
    benches,
    benchmark_fire_and_forget_creation,
    benchmark_request_response_creation,
    benchmark_codec_creation,
    benchmark_router_creation,
);
criterion_main!(benches);
