//! Capability Check API Performance Benchmarks (Task 3.1)
//!
//! These benchmarks measure the performance of the capability checking system
//! to verify compliance with the <5μs per check performance target.
//!
//! # Performance Targets
//!
//! - **Fast Path (no capabilities)**: <1μs (early deny)
//! - **Typical Check**: <5μs (including ACL evaluation)
//! - **Registration**: <10μs per component
//! - **Unregistration**: <5μs per component
//!
//! # Run Benchmarks
//!
//! ```bash
//! cargo bench --bench capability_check_benchmarks
//! ```

use airssys_wasm::security::enforcement::{CapabilityChecker, register_component};
use airssys_wasm::security::{WasmCapability, WasmCapabilitySet, WasmSecurityContext};
use std::hint::black_box;
use criterion::{ criterion_group, criterion_main, Criterion, BenchmarkId};

/// Benchmark: Component registration.
///
/// Measures time to register a component with a typical capability set.
fn bench_component_registration(c: &mut Criterion) {
    c.bench_function("component_registration", |b| {
        let mut counter = 0;
        b.iter(|| {
            counter += 1;
            let checker = CapabilityChecker::new();
            let capabilities = WasmCapabilitySet::new()
                .grant(WasmCapability::Filesystem {
                    paths: vec!["/app/data/*".to_string()],
                    permissions: vec!["read".to_string(), "write".to_string()],
                });

            let security_ctx = WasmSecurityContext::new(
                format!("bench-comp-{}", counter),
                capabilities,
            );

            black_box(checker.register_component(security_ctx).expect("registration failed"));
        });
    });
}

/// Benchmark: Component unregistration.
///
/// Measures time to unregister a component.
fn bench_component_unregistration(c: &mut Criterion) {
    c.bench_function("component_unregistration", |b| {
        let mut counter = 0;
        b.iter_with_setup(
            || {
                counter += 1;
                let checker = CapabilityChecker::new();
                let security_ctx = WasmSecurityContext::new(
                    format!("bench-unreg-{}", counter),
                    WasmCapabilitySet::new(),
                );
                checker.register_component(security_ctx).expect("registration failed");
                (checker, format!("bench-unreg-{}", counter))
            },
            |(checker, component_id)| {
                black_box(checker.unregister_component(&component_id).expect("unregistration failed"));
            },
        );
    });
}

/// Benchmark: Capability check (fast path - no capabilities).
///
/// Measures time for early deny when component has no capabilities.
/// Target: <1μs
fn bench_check_fast_path_no_capabilities(c: &mut Criterion) {
    let checker = CapabilityChecker::new();
    let security_ctx = WasmSecurityContext::new(
        "bench-fast-path".to_string(),
        WasmCapabilitySet::new(), // Empty capabilities
    );
    checker.register_component(security_ctx).expect("registration failed");

    c.bench_function("check_fast_path_no_capabilities", |b| {
        b.iter(|| {
            black_box(checker.check(
                black_box("bench-fast-path"),
                black_box("/app/data/file.json"),
                black_box("read"),
            ))
        });
    });
}

/// Benchmark: Capability check (single capability - granted).
///
/// Measures time for typical check with single filesystem capability.
/// Target: <5μs
fn bench_check_single_capability_granted(c: &mut Criterion) {
    let checker = CapabilityChecker::new();
    let capabilities = WasmCapabilitySet::new().grant(WasmCapability::Filesystem {
        paths: vec!["/app/data/*".to_string()],
        permissions: vec!["read".to_string()],
    });
    let security_ctx = WasmSecurityContext::new("bench-single-cap".to_string(), capabilities);
    checker.register_component(security_ctx).expect("registration failed");

    c.bench_function("check_single_capability_granted", |b| {
        b.iter(|| {
            black_box(checker.check(
                black_box("bench-single-cap"),
                black_box("/app/data/file.json"),
                black_box("read"),
            ))
        });
    });
}

/// Benchmark: Capability check (single capability - denied pattern mismatch).
///
/// Measures time for denied check due to pattern mismatch.
fn bench_check_single_capability_denied_pattern(c: &mut Criterion) {
    let checker = CapabilityChecker::new();
    let capabilities = WasmCapabilitySet::new().grant(WasmCapability::Filesystem {
        paths: vec!["/app/data/*".to_string()],
        permissions: vec!["read".to_string()],
    });
    let security_ctx = WasmSecurityContext::new("bench-single-denied".to_string(), capabilities);
    checker.register_component(security_ctx).expect("registration failed");

    c.bench_function("check_single_capability_denied_pattern", |b| {
        b.iter(|| {
            black_box(checker.check(
                black_box("bench-single-denied"),
                black_box("/etc/passwd"), // Outside pattern
                black_box("read"),
            ))
        });
    });
}

/// Benchmark: Capability check (multiple capabilities - 10 capabilities).
///
/// Measures time for check with multiple declared capabilities.
fn bench_check_multiple_capabilities(c: &mut Criterion) {
    let checker = CapabilityChecker::new();
    let mut capabilities = WasmCapabilitySet::new();

    // Add 10 filesystem capabilities
    for i in 0..10 {
        capabilities = capabilities.grant(WasmCapability::Filesystem {
            paths: vec![format!("/app/data-{}/", i)],
            permissions: vec!["read".to_string()],
        });
    }

    let security_ctx = WasmSecurityContext::new("bench-multi-cap".to_string(), capabilities);
    checker.register_component(security_ctx).expect("registration failed");

    c.bench_function("check_multiple_capabilities_10", |b| {
        b.iter(|| {
            black_box(checker.check(
                black_box("bench-multi-cap"),
                black_box("/app/data-5/file.json"), // Match middle capability
                black_box("read"),
            ))
        });
    });
}

/// Benchmark: Capability check scaling by capability count.
///
/// Measures how check performance scales with number of capabilities.
fn bench_check_scaling_by_capability_count(c: &mut Criterion) {
    let mut group = c.benchmark_group("check_scaling");

    for cap_count in [1, 5, 10, 20, 50, 100].iter() {
        let checker = CapabilityChecker::new();
        let mut capabilities = WasmCapabilitySet::new();

        // Add N filesystem capabilities
        for i in 0..*cap_count {
            capabilities = capabilities.grant(WasmCapability::Filesystem {
                paths: vec![format!("/app/data-{}/", i)],
                permissions: vec!["read".to_string()],
            });
        }

        let component_id = format!("bench-scale-{}", cap_count);
        let security_ctx = WasmSecurityContext::new(component_id.clone(), capabilities);
        checker.register_component(security_ctx).expect("registration failed");

        group.bench_with_input(
            BenchmarkId::from_parameter(cap_count),
            cap_count,
            |b, &cap_count| {
                b.iter(|| {
                    black_box(checker.check(
                        black_box(&format!("bench-scale-{}", cap_count)),
                        black_box(&format!("/app/data-{}/file.json", cap_count / 2)),
                        black_box("read"),
                    ))
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Capability check with complex glob patterns.
///
/// Measures time for checks with complex glob patterns (**, *, ?).
fn bench_check_complex_glob_patterns(c: &mut Criterion) {
    let checker = CapabilityChecker::new();
    let capabilities = WasmCapabilitySet::new().grant(WasmCapability::Filesystem {
        paths: vec![
            "/app/**/*.log".to_string(),       // Recursive glob
            "/app/data-?/*.json".to_string(),  // Single character wildcard
            "/app/config/*.{toml,json}".to_string(), // Alternatives
        ],
        permissions: vec!["read".to_string()],
    });
    let security_ctx = WasmSecurityContext::new("bench-glob".to_string(), capabilities);
    checker.register_component(security_ctx).expect("registration failed");

    c.bench_function("check_complex_glob_patterns", |b| {
        b.iter(|| {
            black_box(checker.check(
                black_box("bench-glob"),
                black_box("/app/logs/subdir/app.log"), // Match recursive glob
                black_box("read"),
            ))
        });
    });
}

/// Benchmark: Capability check for network capabilities.
///
/// Measures time for network capability checks (different resource type).
fn bench_check_network_capability(c: &mut Criterion) {
    let checker = CapabilityChecker::new();
    let capabilities = WasmCapabilitySet::new().grant(WasmCapability::Network {
        endpoints: vec!["api.example.com:443".to_string()],
        permissions: vec!["connect".to_string()],
    });
    let security_ctx = WasmSecurityContext::new("bench-network".to_string(), capabilities);
    checker.register_component(security_ctx).expect("registration failed");

    c.bench_function("check_network_capability", |b| {
        b.iter(|| {
            black_box(checker.check(
                black_box("bench-network"),
                black_box("api.example.com:443"),
                black_box("connect"),
            ))
        });
    });
}

/// Benchmark: Capability check for storage capabilities.
///
/// Measures time for storage capability checks (namespace-based).
fn bench_check_storage_capability(c: &mut Criterion) {
    let checker = CapabilityChecker::new();
    let capabilities = WasmCapabilitySet::new().grant(WasmCapability::Storage {
        namespaces: vec!["component:test:data:*".to_string()],
        permissions: vec!["read".to_string(), "write".to_string()],
    });
    let security_ctx = WasmSecurityContext::new("bench-storage".to_string(), capabilities);
    checker.register_component(security_ctx).expect("registration failed");

    c.bench_function("check_storage_capability", |b| {
        b.iter(|| {
            black_box(checker.check(
                black_box("bench-storage"),
                black_box("component:test:data:config"),
                black_box("read"),
            ))
        });
    });
}

/// Benchmark: Concurrent capability checks.
///
/// Measures throughput for concurrent checks from multiple threads.
fn bench_concurrent_checks(c: &mut Criterion) {
    use std::sync::Arc;
    use std::thread;

    let checker = Arc::new(CapabilityChecker::new());
    let capabilities = WasmCapabilitySet::new().grant(WasmCapability::Filesystem {
        paths: vec!["/app/data/*".to_string()],
        permissions: vec!["read".to_string()],
    });
    let security_ctx = WasmSecurityContext::new("bench-concurrent".to_string(), capabilities);
    checker.register_component(security_ctx).expect("registration failed");

    c.bench_function("concurrent_checks_4_threads", |b| {
        b.iter(|| {
            let mut handles = vec![];

            for i in 0..4 {
                let checker_clone = Arc::clone(&checker);
                let handle = thread::spawn(move || {
                    for j in 0..100 {
                        black_box(checker_clone.check(
                            "bench-concurrent",
                            &format!("/app/data/file-{}-{}.json", i, j),
                            "read",
                        ));
                    }
                });
                handles.push(handle);
            }

            for handle in handles {
                handle.join().unwrap();
            }
        });
    });
}

/// Benchmark: Global check_capability() function.
///
/// Measures performance of the global convenience API.
fn bench_global_check_capability(c: &mut Criterion) {
    let component_id = "bench-global-check";
    let capabilities = WasmCapabilitySet::new().grant(WasmCapability::Filesystem {
        paths: vec!["/app/data/*".to_string()],
        permissions: vec!["read".to_string()],
    });
    let security_ctx = WasmSecurityContext::new(component_id.to_string(), capabilities);
    register_component(security_ctx).expect("registration failed");

    c.bench_function("global_check_capability", |b| {
        b.iter(|| {
            black_box(
                airssys_wasm::security::check_capability(
                    black_box(component_id),
                    black_box("/app/data/file.json"),
                    black_box("read"),
                )
                .expect("check failed")
            )
        });
    });
}

criterion_group!(
    benches,
    bench_component_registration,
    bench_component_unregistration,
    bench_check_fast_path_no_capabilities,
    bench_check_single_capability_granted,
    bench_check_single_capability_denied_pattern,
    bench_check_multiple_capabilities,
    bench_check_scaling_by_capability_count,
    bench_check_complex_glob_patterns,
    bench_check_network_capability,
    bench_check_storage_capability,
    bench_concurrent_checks,
    bench_global_check_capability,
);

criterion_main!(benches);
