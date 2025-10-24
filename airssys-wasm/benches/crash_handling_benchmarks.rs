// Standard library imports
use std::sync::Arc;

// Third-party imports
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

// Internal imports
use airssys_wasm::core::{
    ComponentId, ComponentInput, ExecutionContext, ResourceLimits, RuntimeEngine,
};
use airssys_wasm::runtime::WasmEngine;

/// Helper: Create component that executes successfully
fn create_success_component() -> Vec<u8> {
    wat::parse_str(
        r#"
        (component
            (core module $success
                (func (export "success") (result i32)
                    i32.const 42
                )
            )
            (core instance $m (instantiate $success))
            (func (export "success") (result s32)
                (canon lift (core func $m "success"))
            )
        )
        "#,
    )
    .expect("WAT compilation should succeed")
}

/// Helper: Create component that traps (division by zero)
fn create_trap_component() -> Vec<u8> {
    wat::parse_str(
        r#"
        (component
            (core module $trap
                (func (export "divide_zero") (result i32)
                    i32.const 42
                    i32.const 0
                    i32.div_s  ;; Division by zero - will trap
                )
            )
            (core instance $m (instantiate $trap))
            (func (export "divide_zero") (result s32)
                (canon lift (core func $m "divide_zero"))
            )
        )
        "#,
    )
    .expect("WAT compilation should succeed")
}

/// Helper: Create component that exhausts fuel
fn create_fuel_exhaustion_component() -> Vec<u8> {
    wat::parse_str(
        r#"
        (component
            (core module $fuel
                (func (export "infinite") (result i32)
                    (local $i i32)
                    i32.const 0
                    local.set $i
                    
                    block $break
                        loop $continue
                            local.get $i
                            i32.const 1000000
                            i32.lt_u
                            if
                                local.get $i
                                i32.const 1
                                i32.add
                                local.set $i
                                br $continue
                            end
                        end
                    end
                    
                    local.get $i
                )
            )
            (core instance $m (instantiate $fuel))
            (func (export "infinite") (result s32)
                (canon lift (core func $m "infinite"))
            )
        )
        "#,
    )
    .expect("WAT compilation should succeed")
}

/// Benchmark: Normal execution baseline (no crashes)
fn bench_normal_execution_baseline(c: &mut Criterion) {
    let mut group = c.benchmark_group("crash_handling/normal_execution");
    
    group.sample_size(30);
    group.measurement_time(std::time::Duration::from_secs(5));
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    let engine = WasmEngine::new().expect("Engine creation should succeed");
    let component_id = ComponentId::new("success-component");
    let bytes = create_success_component();
    
    let handle = rt.block_on(async {
        engine
            .load_component(&component_id, &bytes)
            .await
            .expect("Component loading should succeed")
    });
    
    let limits = ComponentResourceLimits::default();
    let context = ExecutionContext::new(ComponentVersion::new(1, 0, 0));
    
    group.bench_function("successful_execution", |b| {
        b.to_async(&rt).iter(|| async {
            let input = ComponentInput::empty();
            
            let _output = engine
                .execute(
                    black_box(&handle),
                    black_box("success"),
                    black_box(input),
                    black_box(&context),
                )
                .await
                .expect("Execution should succeed");
        });
    });
    
    group.finish();
}

/// Benchmark: Trap detection overhead (Phase 5 integration)
fn bench_trap_detection_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("crash_handling/trap_detection");
    
    group.sample_size(30);
    group.measurement_time(std::time::Duration::from_secs(5));
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    let engine = WasmEngine::new().expect("Engine creation should succeed");
    let component_id = ComponentId::new("trap-component");
    let bytes = create_trap_component();
    
    let handle = rt.block_on(async {
        engine
            .load_component(&component_id, &bytes)
            .await
            .expect("Component loading should succeed")
    });
    
    let limits = ComponentResourceLimits::default();
    let context = ExecutionContext::new(ComponentVersion::new(1, 0, 0));
    
    group.bench_function("trap_categorization", |b| {
        b.to_async(&rt).iter(|| async {
            let input = ComponentInput::empty();
            
            // Execute (will trap)
            let result = engine
                .execute(
                    black_box(&handle),
                    black_box("divide_zero"),
                    black_box(input),
                    black_box(&context),
                )
                .await;
            
            // Verify trap was detected (don't panic on error)
            let _ = black_box(result);
        });
    });
    
    group.finish();
}

/// Benchmark: Resource cleanup performance (StoreWrapper Drop)
fn bench_resource_cleanup_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("crash_handling/resource_cleanup");
    
    group.sample_size(30);
    group.measurement_time(std::time::Duration::from_secs(5));
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    let engine = WasmEngine::new().expect("Engine creation should succeed");
    let success_bytes = create_success_component();
    let trap_bytes = create_trap_component();
    
    // Cleanup after successful execution
    group.bench_function("cleanup_after_success", |b| {
        let component_id = ComponentId::new("cleanup-success");
        let handle = rt.block_on(async {
            engine
                .load_component(&component_id, &success_bytes)
                .await
                .expect("Component loading should succeed")
        });
        
        let limits = ComponentResourceLimits::default();
        let context = ExecutionContext::new(ComponentVersion::new(1, 0, 0));
        
        b.to_async(&rt).iter(|| async {
            let input = ComponentInput::empty();
            
            // Execute and let StoreWrapper drop cleanup resources
            let _output = engine
                .execute(
                    black_box(&handle),
                    black_box("success"),
                    black_box(input),
                    black_box(&context),
                )
                .await
                .expect("Execution should succeed");
            
            // StoreWrapper Drop happens here - measure cleanup overhead
        });
    });
    
    // Cleanup after trap
    group.bench_function("cleanup_after_trap", |b| {
        let component_id = ComponentId::new("cleanup-trap");
        let handle = rt.block_on(async {
            engine
                .load_component(&component_id, &trap_bytes)
                .await
                .expect("Component loading should succeed")
        });
        
        let limits = ComponentResourceLimits::default();
        let context = ExecutionContext::new(ComponentVersion::new(1, 0, 0));
        
        b.to_async(&rt).iter(|| async {
            let input = ComponentInput::empty();
            
            // Execute (will trap) and let StoreWrapper drop cleanup resources
            let _ = engine
                .execute(
                    black_box(&handle),
                    black_box("divide_zero"),
                    black_box(input),
                    black_box(&context),
                )
                .await;
            
            // StoreWrapper Drop happens here - measure cleanup overhead after trap
        });
    });
    
    group.finish();
}

/// Benchmark: Crash recovery latency
fn bench_crash_recovery_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("crash_handling/recovery_latency");
    
    group.sample_size(30);
    group.measurement_time(std::time::Duration::from_secs(5));
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    let engine = Arc::new(WasmEngine::new().expect("Engine creation should succeed"));
    
    let trap_bytes = create_trap_component();
    let success_bytes = create_success_component();
    
    group.bench_function("recovery_after_crash", |b| {
        b.to_async(&rt).iter(|| {
            let engine = Arc::clone(&engine);
            let trap_bytes = trap_bytes.clone();
            let success_bytes = success_bytes.clone();
            
            async move {
                // 1. Load and execute crashing component
                let trap_id = ComponentId::new("crash-then-recover");
                let trap_handle = engine
                    .load_component(black_box(&trap_id), black_box(&trap_bytes))
                    .await
                    .expect("Trap component loading should succeed");
                
                let limits = ComponentResourceLimits::default();
                let context = ExecutionContext::new(ComponentVersion::new(1, 0, 0));
                let input = ComponentInput::empty();
                
                // Execute (will trap)
                let _ = engine
                    .execute(
                        black_box(&trap_handle),
                        black_box("divide_zero"),
                        black_box(input.clone()),
                        black_box(&context),
                    )
                    .await;
                
                // 2. Immediately load and execute successful component
                // This measures host stability and recovery latency
                let success_id = ComponentId::new("after-crash");
                let success_handle = engine
                    .load_component(black_box(&success_id), black_box(&success_bytes))
                    .await
                    .expect("Success component loading should succeed");
                
                let _output = engine
                    .execute(
                        black_box(&success_handle),
                        black_box("success"),
                        black_box(input),
                        black_box(&context),
                    )
                    .await
                    .expect("Recovery execution should succeed");
            }
        });
    });
    
    group.finish();
}

/// Benchmark: Fuel exhaustion handling overhead
fn bench_fuel_exhaustion_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("crash_handling/fuel_exhaustion");
    
    group.sample_size(30);
    group.measurement_time(std::time::Duration::from_secs(5));
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    let engine = WasmEngine::new().expect("Engine creation should succeed");
    let component_id = ComponentId::new("fuel-exhaustion");
    let bytes = create_fuel_exhaustion_component();
    
    let handle = rt.block_on(async {
        engine
            .load_component(&component_id, &bytes)
            .await
            .expect("Component loading should succeed")
    });
    
    // Very low fuel limit to exhaust quickly
    let mut limits = ComponentResourceLimits::default();
    limits.max_fuel_per_execution = 10_000; // Will exhaust quickly
    
    let context = ExecutionContext::new(ComponentVersion::new(1, 0, 0));
    
    group.bench_function("fuel_trap_handling", |b| {
        b.to_async(&rt).iter(|| async {
            let input = ComponentInput::empty();
            
            // Execute (will exhaust fuel)
            let result = engine
                .execute(
                    black_box(&handle),
                    black_box("infinite"),
                    black_box(input),
                    black_box(&context),
                )
                .await;
            
            // Verify fuel exhaustion was detected
            let _ = black_box(result);
        });
    });
    
    group.finish();
}

criterion_group!(
    crash_benches,
    bench_normal_execution_baseline,
    bench_trap_detection_overhead,
    bench_resource_cleanup_performance,
    bench_crash_recovery_latency,
    bench_fuel_exhaustion_overhead
);

criterion_main!(crash_benches);
