#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

//! Benchmark for WASM component loading performance.
//!
//! Measures the performance of loading and parsing WASM components with Wasmtime.

// Allow panic in benchmarks (setup code only)

// Layer 1: Standard library imports
use std::hint::black_box;

// Layer 2: External crate imports
use criterion::{criterion_group, criterion_main, Criterion};

// Layer 3: Internal module imports
use airssys_wasm::core::{runtime::RuntimeEngine, ComponentId};
use airssys_wasm::runtime::WasmEngine;

/// Create minimal WASM component for benchmarking.
fn create_minimal_component() -> Vec<u8> {
    wat::parse_str(
        r#"
        (component
            (core module $minimal
                (func (export "hello") (result i32)
                    i32.const 42
                )
            )
            (core instance $m (instantiate $minimal))
            (func (export "hello") (result s32)
                (canon lift (core func $m "hello"))
            )
        )
        "#,
    )
    .unwrap_or_else(|e| panic!("WAT compilation failed: {e}"))
}

/// Benchmark: Component loading from bytes.
fn bench_load_component(c: &mut Criterion) {
    let rt =
        tokio::runtime::Runtime::new().unwrap_or_else(|e| panic!("Failed to create runtime: {e}"));
    let engine = WasmEngine::new().unwrap_or_else(|e| panic!("Failed to create engine: {e}"));
    let bytes = create_minimal_component();

    c.bench_function("load_component", |b| {
        b.to_async(&rt).iter(|| async {
            let component_id = ComponentId::new("bench-component");
            let result = engine
                .load_component(black_box(&component_id), black_box(&bytes))
                .await;

            black_box(result).unwrap_or_else(|e| panic!("Component loading failed: {e}"))
        });
    });
}

/// Benchmark: Engine creation overhead.
fn bench_engine_creation(c: &mut Criterion) {
    c.bench_function("engine_creation", |b| {
        b.iter(|| {
            let engine = WasmEngine::new();
            black_box(engine).unwrap_or_else(|e| panic!("Engine creation failed: {e}"))
        });
    });
}

criterion_group!(benches, bench_engine_creation, bench_load_component);
criterion_main!(benches);
