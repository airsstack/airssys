// Standard library imports
use std::collections::HashMap;
use std::sync::Arc;

// Third-party imports
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

// Internal imports
use airssys_wasm::core::{
    CapabilitySet, ComponentId, ComponentInput, ExecutionContext, ResourceLimits, RuntimeEngine,
};
use airssys_wasm::runtime::WasmEngine;

/// Helper: Create minimal component for benchmarking
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
    .expect("WAT compilation should succeed")
}

/// Helper: Create component with increasing complexity
fn create_sized_component(size_category: &str) -> Vec<u8> {
    let func_count = match size_category {
        "1kb" => 10,
        "10kb" => 100,
        "100kb" => 1000,
        _ => 10,
    };

    // Generate multiple simple functions to increase code size
    let mut funcs = String::new();
    for i in 0..func_count {
        funcs.push_str(&format!(
            r#"
                (func $func_{i} (result i32)
                    i32.const {i}
                )
            "#
        ));
    }

    let mut exports = String::new();
    for i in 0..func_count {
        exports.push_str(&format!(
            r#"
                (export "func_{i}" (func $func_{i}))
            "#
        ));
    }

    let wat = format!(
        r#"
        (component
            (core module $sized
                {funcs}
                {exports}
            )
            (core instance $m (instantiate $sized))
            (func (export "main") (result s32)
                (canon lift (core func $m "func_0"))
            )
        )
        "#
    );

    wat::parse_str(&wat).expect("WAT compilation should succeed")
}

/// Benchmark: Cold start instantiation (load + compile + instantiate)
fn bench_cold_start_minimal(c: &mut Criterion) {
    let mut group = c.benchmark_group("instantiation/cold_start");
    
    // Resource-conscious configuration
    group.sample_size(30);
    group.measurement_time(std::time::Duration::from_secs(5));
    
    group.bench_function("minimal_component", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        
        b.to_async(&rt).iter(|| async {
            // Fresh engine for each iteration (true cold start)
            let engine = WasmEngine::new().expect("Engine creation should succeed");
            let component_id = ComponentId::new("bench-minimal");
            let bytes = create_minimal_component();
            
            // Cold start: load + compile + instantiate
            let _handle = engine
                .load_component(black_box(&component_id), black_box(&bytes))
                .await
                .expect("Component loading should succeed");
        });
    });
    
    group.finish();
}

/// Benchmark: Warm start instantiation (reuse compiled module)
fn bench_warm_start_minimal(c: &mut Criterion) {
    let mut group = c.benchmark_group("instantiation/warm_start");
    
    group.sample_size(30);
    group.measurement_time(std::time::Duration::from_secs(5));
    
    group.bench_function("minimal_component", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        
        // Pre-create engine (amortize engine creation cost)
        let engine = WasmEngine::new().expect("Engine creation should succeed");
        let bytes = create_minimal_component();
        
        b.to_async(&rt).iter(|| async {
            let component_id = ComponentId::new("bench-minimal");
            // Warm start: instantiate pre-compiled module
            let _handle = engine
                .load_component(black_box(&component_id), black_box(&bytes))
                .await
                .expect("Component loading should succeed");
        });
    });
    
    group.finish();
}

/// Benchmark: Component size scaling (1KB, 10KB, 100KB)
fn bench_component_size_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("instantiation/size_scaling");
    
    group.sample_size(30);
    group.measurement_time(std::time::Duration::from_secs(5));
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    let engine = Arc::new(WasmEngine::new().expect("Engine creation should succeed"));
    
    for size in &["1kb", "10kb", "100kb"] {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let component_bytes = create_sized_component(size);
            let engine = Arc::clone(&engine);
            
            b.to_async(&rt).iter(|| {
                let engine = Arc::clone(&engine);
                let bytes = component_bytes.clone();
                async move {
                    let component_id = ComponentId::new(&format!("bench-{size}"));
                    let _handle = engine
                        .load_component(black_box(&component_id), black_box(&bytes))
                        .await
                        .expect("Component loading should succeed");
                }
            });
        });
    }
    
    group.finish();
}

criterion_group!(
    instantiation_benches,
    bench_cold_start_minimal,
    bench_warm_start_minimal,
    bench_component_size_scaling
);

criterion_main!(instantiation_benches);
