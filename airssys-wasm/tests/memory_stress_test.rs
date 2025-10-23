//! Memory stress tests under high load.
//!
//! Verifies system behavior under high-frequency allocations and concurrent components.

#![allow(clippy::expect_used)] // Test code - expect is acceptable for test assertions

use airssys_wasm::runtime::limits::{ComponentResourceLimiter, ResourceLimits};
use wasmtime::ResourceLimiter;

#[test]
fn test_high_frequency_allocations() {
    let limits = ResourceLimits::builder()
        .max_memory_bytes(1024 * 1024)
        .max_fuel(10_000)
        .timeout_seconds(30)
        .build()
        .expect("valid memory limit");
    let mut limiter = ComponentResourceLimiter::new(limits);

    for i in 0..10000 {
        let size = 256 * 1024 + (i % 10) * 1024;
        limiter
            .memory_growing(0, size.min(1024 * 1024), None)
            .expect("memory_growing should not error");
    }
}

#[test]
fn test_concurrent_components_high_load() {
    let limits = ResourceLimits::builder()
        .max_memory_bytes(1024 * 1024)
        .max_fuel(10_000)
        .timeout_seconds(30)
        .build()
        .expect("valid memory limit");

    let mut limiters: Vec<_> = (0..100)
        .map(|_| ComponentResourceLimiter::new(limits))
        .collect();

    for limiter in &mut limiters {
        assert!(limiter
            .memory_growing(0, 512 * 1024, None)
            .expect("memory_growing should not error"));
    }

    for (i, limiter) in limiters.iter().enumerate() {
        assert_eq!(
            limiter.current_memory_bytes(),
            512 * 1024,
            "Component {i} has incorrect usage"
        );
    }
}

#[test]
fn test_oom_recovery_stress() {
    let limits = ResourceLimits::builder()
        .max_memory_bytes(1024 * 1024)
        .max_fuel(10_000)
        .timeout_seconds(30)
        .build()
        .expect("valid memory limit");
    let mut limiter = ComponentResourceLimiter::new(limits);

    for _ in 0..1000 {
        assert!(!limiter
            .memory_growing(0, 2 * 1024 * 1024, None)
            .expect("memory_growing should not error"));

        assert!(limiter
            .memory_growing(0, 512 * 1024, None)
            .expect("memory_growing should not error"));
    }
}

#[test]
fn test_edge_case_allocations() {
    let limits = ResourceLimits::builder()
        .max_memory_bytes(1024 * 1024)
        .max_fuel(10_000)
        .timeout_seconds(30)
        .build()
        .expect("valid memory limit");
    let mut limiter = ComponentResourceLimiter::new(limits);

    assert!(limiter
        .memory_growing(0, 0, None)
        .expect("memory_growing should not error"));

    assert!(limiter
        .memory_growing(0, 1, None)
        .expect("memory_growing should not error"));

    assert!(limiter
        .memory_growing(0, 1024 * 1024, None)
        .expect("memory_growing should not error"));

    assert!(!limiter
        .memory_growing(0, 1024 * 1024 + 1, None)
        .expect("memory_growing should not error"));
}
