// Standard library imports
use std::sync::Arc;

// Third-party imports
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

// Internal imports
use airssys_wasm::core::{
    ComponentId, ComponentInput, ExecutionContext, ResourceLimits, RuntimeEngine,
};
use airssys_wasm::runtime::{ComponentResourceLimiter, WasmEngine};

/// Helper: Create minimal component for memory baseline
fn create_minimal_component() -> Vec<u8> {
    wat::parse_str(
        r#"
        (component
            (core module $minimal
                (memory (export "memory") 1)  ;; 1 page = 64KB
                
                (func (export "noop") (result i32)
                    i32.const 42
                )
            )
            (core instance $m (instantiate $minimal))
            (func (export "noop") (result s32)
                (canon lift (core func $m "noop"))
            )
        )
        "#,
    )
    .expect("WAT compilation should succeed")
}

/// Helper: Create component with specified memory size
fn create_sized_component(pages: u32) -> Vec<u8> {
    let wat = format!(
        r#"
        (component
            (core module $sized
                (memory (export "memory") {pages})  ;; {pages} pages = {kb}KB
                
                (func (export "noop") (result i32)
                    i32.const 42
                )
            )
            (core instance $m (instantiate $sized))
            (func (export "noop") (result s32)
                (canon lift (core func $m "noop"))
            )
        )
        "#,
        pages = pages,
        kb = pages * 64
    );
    
    wat::parse_str(&wat).expect("WAT compilation should succeed")
}

/// Benchmark: Per-component memory footprint baseline
fn bench_component_memory_footprint(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory/footprint");
    
    group.sample_size(30);
    group.measurement_time(std::time::Duration::from_secs(5));
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    let engine = Arc::new(WasmEngine::new().expect("Engine creation should succeed"));
    
    // Test different memory configurations
    let configs = vec![
        (1, "64kb"),    // Minimal
        (8, "512kb"),   // Default target
        (16, "1mb"),    // Medium
        (32, "2mb"),    // Large
    ];
    
    for (pages, label) in configs {
        group.bench_with_input(BenchmarkId::from_parameter(label), &pages, |b, &pages| {
            let bytes = create_sized_component(pages);
            let engine = Arc::clone(&engine);
            
            b.to_async(&rt).iter(|| {
                let engine = Arc::clone(&engine);
                let bytes = bytes.clone();
                async move {
                    let component_id = ComponentId::new(&format!("mem-{pages}"));
                    
                    // Load and instantiate (allocates memory)
                    let handle = engine
                        .load_component(black_box(&component_id), black_box(&bytes))
                        .await
                        .expect("Component loading should succeed");
                    
                    // Execute to ensure full instantiation
                    let limits = ComponentResourceLimits::default();
                    let context = ExecutionContext::new(ComponentVersion::new(1, 0, 0));
                    let input = ComponentInput::empty();
                    
                    let _output = engine
                        .execute(
                            black_box(&handle),
                            black_box("noop"),
                            black_box(input),
                            black_box(&context),
                        )
                        .await
                        .expect("Execution should succeed");
                }
            });
        });
    }
    
    group.finish();
}

/// Benchmark: Memory scaling with concurrent components
fn bench_memory_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory/scaling");
    
    group.sample_size(30);
    group.measurement_time(std::time::Duration::from_secs(5));
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    let engine = Arc::new(WasmEngine::new().expect("Engine creation should succeed"));
    let bytes = create_minimal_component();
    
    // Test scaling: 1, 10, 50 concurrent components
    for count in &[1, 10, 50] {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{count}_components")),
            count,
            |b, &count| {
                b.to_async(&rt).iter(|| {
                    let engine = Arc::clone(&engine);
                    let bytes = bytes.clone();
                    
                    async move {
                        let mut handles = Vec::new();
                        
                        // Load multiple components concurrently
                        for i in 0..count {
                            let engine = Arc::clone(&engine);
                            let bytes = bytes.clone();
                            let component_id = ComponentId::new(&format!("scale-{i}"));
                            
                            let handle = tokio::spawn(async move {
                                engine
                                    .load_component(&component_id, &bytes)
                                    .await
                                    .expect("Component loading should succeed")
                            });
                            
                            handles.push(handle);
                        }
                        
                        // Wait for all to complete
                        for handle in handles {
                            let _ = handle.await;
                        }
                    }
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark: Memory limit enforcement overhead (Phase 2 integration)
fn bench_memory_limit_enforcement(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory/limit_enforcement");
    
    group.sample_size(30);
    group.measurement_time(std::time::Duration::from_secs(5));
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    let engine = WasmEngine::new().expect("Engine creation should succeed");
    
    // Component that allocates memory up to limit
    let bytes = wat::parse_str(
        r#"
        (component
            (core module $alloc
                (memory (export "memory") 8 16)  ;; Initial 8, max 16 pages
                
                (func (export "grow") (param i32) (result i32)
                    local.get 0
                    memory.grow
                )
            )
            (core instance $m (instantiate $alloc))
            (func (export "grow") (param "pages" s32) (result s32)
                (canon lift (core func $m "grow"))
            )
        )
        "#,
    )
    .expect("WAT compilation should succeed");
    
    let component_id = ComponentId::new("memory-grow");
    let handle = rt.block_on(async {
        engine
            .load_component(&component_id, &bytes)
            .await
            .expect("Component loading should succeed")
    });
    
    let limits = ComponentResourceLimiter::default();
    let context = ExecutionContext::new(ComponentVersion::new(1, 0, 0));
    
    group.bench_function("memory_grow_within_limit", |b| {
        b.to_async(&rt).iter(|| async {
            // Grow by 1 page (within limit)
            let input = ComponentInput::from_params(vec![("pages", 1.into())]);
            
            let _output = engine
                .execute(
                    black_box(&handle),
                    black_box("grow"),
                    black_box(input),
                    black_box(&context),
                )
                .await
                .expect("Execution should succeed");
        });
    });
    
    group.finish();
}

/// Benchmark: Linear scaling validation
fn bench_memory_linear_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory/linear_scaling");
    
    group.sample_size(30);
    group.measurement_time(std::time::Duration::from_secs(5));
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    let engine = Arc::new(WasmEngine::new().expect("Engine creation should succeed"));
    
    // Sequential component loading (measure overhead per component)
    for count in &[1, 5, 10, 25, 50] {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{count}_sequential")),
            count,
            |b, &count| {
                let bytes = create_minimal_component();
                let engine = Arc::clone(&engine);
                
                b.to_async(&rt).iter(|| {
                    let engine = Arc::clone(&engine);
                    let bytes = bytes.clone();
                    
                    async move {
                        // Load components sequentially
                        for i in 0..count {
                            let component_id = ComponentId::new(&format!("seq-{i}"));
                            let _handle = engine
                                .load_component(black_box(&component_id), black_box(&bytes))
                                .await
                                .expect("Component loading should succeed");
                        }
                    }
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    memory_benches,
    bench_component_memory_footprint,
    bench_memory_scaling,
    bench_memory_limit_enforcement,
    bench_memory_linear_scaling
);

criterion_main!(memory_benches);
