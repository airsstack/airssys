#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

//! Benchmark for WASM component execution performance.
//!
//! Measures the performance of executing WASM component functions.

// Allow panic in benchmarks (setup code only)

// Layer 1: Standard library imports
use std::collections::HashMap;
use std::hint::black_box;

// Layer 2: External crate imports
use criterion::{criterion_group, criterion_main, Criterion};

// Layer 3: Internal module imports
use airssys_wasm::core::{
    capability::CapabilitySet,
    component::{ComponentId, ComponentInput},
    runtime::{ExecutionContext, RuntimeEngine},
};
use airssys_wasm::prelude::ResourceLimits;

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

/// Benchmark: Component function execution.
fn bench_execute_function(c: &mut Criterion) {
    let rt =
        tokio::runtime::Runtime::new().unwrap_or_else(|e| panic!("Failed to create runtime: {e}"));
    let engine = WasmEngine::new().unwrap_or_else(|e| panic!("Failed to create engine: {e}"));
    let bytes = create_minimal_component();

    // Pre-load component
    let component_id = ComponentId::new("bench-component");
    let handle = rt.block_on(async {
        engine
            .load_component(&component_id, &bytes)
            .await
            .unwrap_or_else(|e| panic!("Component loading failed: {e}"))
    });

    c.bench_function("execute_function", |b| {
        b.to_async(&rt).iter(|| async {
            // Create execution context
            let context = ExecutionContext {
                component_id: component_id.clone(),
                limits: ResourceLimits {
                    max_memory_bytes: 1024 * 1024, // 1MB
                    max_fuel: 100_000,             // 100K fuel
                    timeout_seconds: 5,            // 5s
                },
                capabilities: CapabilitySet::new(),
                timeout_ms: 5000,
            };

            // Create input
            let input = ComponentInput {
                data: Vec::new(),
                codec: 0,
                metadata: HashMap::new(),
            };

            // Execute
            let result = engine
                .execute(
                    black_box(&handle),
                    black_box("hello"),
                    black_box(input),
                    black_box(context),
                )
                .await;

            black_box(result).unwrap_or_else(|e| panic!("Execution failed: {e}"))
        });
    });
}

criterion_group!(benches, bench_execute_function);
criterion_main!(benches);
