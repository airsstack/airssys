// Standard library imports
use std::sync::Arc;

// Third-party imports
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

// Internal imports
use airssys_wasm::core::{
    ComponentId, ComponentInput, ExecutionContext, ResourceLimits, RuntimeEngine,
};
use airssys_wasm::runtime::WasmEngine;

/// Helper: Create minimal compute component
fn create_minimal_compute() -> Vec<u8> {
    wat::parse_str(
        r#"
        (component
            (core module $compute
                (func (export "add") (param i32 i32) (result i32)
                    local.get 0
                    local.get 1
                    i32.add
                )
            )
            (core instance $m (instantiate $compute))
            (func (export "add") (param "a" s32) (param "b" s32) (result s32)
                (canon lift (core func $m "add"))
            )
        )
        "#,
    )
    .expect("WAT compilation should succeed")
}

/// Helper: Create compute-heavy component (fibonacci)
fn create_compute_heavy() -> Vec<u8> {
    wat::parse_str(
        r#"
        (component
            (core module $fib
                (func $fib (export "fib") (param $n i32) (result i32)
                    (local $a i32)
                    (local $b i32)
                    (local $tmp i32)
                    (local $i i32)
                    
                    ;; Iterative fibonacci to avoid stack overflow
                    i32.const 0
                    local.set $a
                    i32.const 1
                    local.set $b
                    i32.const 0
                    local.set $i
                    
                    block $break
                        loop $continue
                            local.get $i
                            local.get $n
                            i32.ge_u
                            br_if $break
                            
                            local.get $a
                            local.get $b
                            i32.add
                            local.set $tmp
                            
                            local.get $b
                            local.set $a
                            local.get $tmp
                            local.set $b
                            
                            local.get $i
                            i32.const 1
                            i32.add
                            local.set $i
                            br $continue
                        end
                    end
                    
                    local.get $a
                )
            )
            (core instance $m (instantiate $fib))
            (func (export "fib") (param "n" s32) (result s32)
                (canon lift (core func $m "fib"))
            )
        )
        "#,
    )
    .expect("WAT compilation should succeed")
}

/// Helper: Create memory-intensive component
fn create_memory_intensive() -> Vec<u8> {
    wat::parse_str(
        r#"
        (component
            (core module $mem
                (memory (export "memory") 10)  ;; 10 pages = 640KB
                
                (func (export "fill") (param i32) (result i32)
                    (local $i i32)
                    (local $val i32)
                    
                    i32.const 0
                    local.set $i
                    local.get 0
                    local.set $val
                    
                    block $break
                        loop $continue
                            local.get $i
                            i32.const 10000
                            i32.ge_u
                            br_if $break
                            
                            local.get $i
                            local.get $val
                            i32.store
                            
                            local.get $i
                            i32.const 4
                            i32.add
                            local.set $i
                            br $continue
                        end
                    end
                    
                    local.get $val
                )
            )
            (core instance $m (instantiate $mem))
            (func (export "fill") (param "val" s32) (result s32)
                (canon lift (core func $m "fill"))
            )
        )
        "#,
    )
    .expect("WAT compilation should succeed")
}

/// Benchmark: Minimal function overhead (vs native baseline)
fn bench_minimal_function_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("execution/minimal_overhead");
    
    group.sample_size(30);
    group.measurement_time(std::time::Duration::from_secs(5));
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    // Native Rust baseline
    group.bench_function("native_add", |b| {
        b.iter(|| {
            let result = black_box(21) + black_box(21);
            black_box(result)
        });
    });
    
    // WASM execution
    group.bench_function("wasm_add", |b| {
        let engine = WasmEngine::new().expect("Engine creation should succeed");
        let component_id = ComponentId::new("minimal-compute");
        let bytes = create_minimal_compute();
        
        let handle = rt.block_on(async {
            engine
                .load_component(&component_id, &bytes)
                .await
                .expect("Component loading should succeed")
        });
        
        let limits = ComponentResourceLimits::default();
        let context = ExecutionContext::new(ComponentVersion::new(1, 0, 0));
        
        b.to_async(&rt).iter(|| async {
            let input = ComponentInput::from_params(vec![("a", 21.into()), ("b", 21.into())]);
            
            let _output = engine
                .execute(
                    black_box(&handle),
                    black_box("add"),
                    black_box(input),
                    black_box(&context),
                )
                .await
                .expect("Execution should succeed");
        });
    });
    
    group.finish();
}

/// Benchmark: Compute-heavy operations (fibonacci)
fn bench_compute_heavy_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("execution/compute_heavy");
    
    group.sample_size(30);
    group.measurement_time(std::time::Duration::from_secs(5));
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    let engine = WasmEngine::new().expect("Engine creation should succeed");
    let component_id = ComponentId::new("compute-heavy");
    let bytes = create_compute_heavy();
    
    let handle = rt.block_on(async {
        engine
            .load_component(&component_id, &bytes)
            .await
            .expect("Component loading should succeed")
    });
    
    let limits = ComponentResourceLimits::default();
    let context = ExecutionContext::new(ComponentVersion::new(1, 0, 0));
    
    for n in &[10, 20, 30] {
        group.bench_with_input(BenchmarkId::from_parameter(format!("fib_{n}")), n, |b, &n| {
            b.to_async(&rt).iter(|| async {
                let input = ComponentInput::from_params(vec![("n", n.into())]);
                
                let _output = engine
                    .execute(
                        black_box(&handle),
                        black_box("fib"),
                        black_box(input),
                        black_box(&context),
                    )
                    .await
                    .expect("Execution should succeed");
            });
        });
    }
    
    group.finish();
}

/// Benchmark: Memory-intensive operations
fn bench_memory_intensive_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("execution/memory_intensive");
    
    group.sample_size(30);
    group.measurement_time(std::time::Duration::from_secs(5));
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    let engine = WasmEngine::new().expect("Engine creation should succeed");
    let component_id = ComponentId::new("memory-intensive");
    let bytes = create_memory_intensive();
    
    let handle = rt.block_on(async {
        engine
            .load_component(&component_id, &bytes)
            .await
            .expect("Component loading should succeed")
    });
    
    let limits = ComponentResourceLimits::default();
    let context = ExecutionContext::new(ComponentVersion::new(1, 0, 0));
    
    group.bench_function("fill_memory", |b| {
        b.to_async(&rt).iter(|| async {
            let input = ComponentInput::from_params(vec![("val", 42.into())]);
            
            let _output = engine
                .execute(
                    black_box(&handle),
                    black_box("fill"),
                    black_box(input),
                    black_box(&context),
                )
                .await
                .expect("Execution should succeed");
        });
    });
    
    group.finish();
}

/// Benchmark: Async host function overhead (Phase 4 integration)
fn bench_async_host_function_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("execution/async_host_overhead");
    
    group.sample_size(30);
    group.measurement_time(std::time::Duration::from_secs(5));
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    // Component that calls async host function
    let bytes = wat::parse_str(
        r#"
        (component
            (core module $async_caller
                (import "host" "sleep" (func $host_sleep (param i64)))
                
                (func (export "call_sleep") (param i64)
                    local.get 0
                    call $host_sleep
                )
            )
            (core instance $m (instantiate $async_caller
                (with "host" (instance
                    (export "sleep" (func $host_sleep))
                ))
            ))
            (func (export "call_sleep") (param "ms" s64)
                (canon lift (core func $m "call_sleep"))
            )
        )
        "#,
    )
    .expect("WAT compilation should succeed");
    
    let engine = WasmEngine::new().expect("Engine creation should succeed");
    let component_id = ComponentId::new("async-host-caller");
    
    let handle = rt.block_on(async {
        engine
            .load_component(&component_id, &bytes)
            .await
            .expect("Component loading should succeed")
    });
    
    let limits = ComponentResourceLimits::default();
    let context = ExecutionContext::new(ComponentVersion::new(1, 0, 0));
    
    group.bench_function("minimal_async_call", |b| {
        b.to_async(&rt).iter(|| async {
            // Call async host function with 1ms sleep
            let input = ComponentInput::from_params(vec![("ms", 1i64.into())]);
            
            let _output = engine
                .execute(
                    black_box(&handle),
                    black_box("call_sleep"),
                    black_box(input),
                    black_box(&context),
                )
                .await
                .expect("Execution should succeed");
        });
    });
    
    group.finish();
}

criterion_group!(
    execution_benches,
    bench_minimal_function_overhead,
    bench_compute_heavy_throughput,
    bench_memory_intensive_throughput,
    // bench_async_host_function_overhead, // TODO: Enable when async host functions implemented
);

criterion_main!(execution_benches);
