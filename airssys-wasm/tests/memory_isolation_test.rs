#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

//! Cross-component memory isolation tests.
//!
//! Verifies 100% isolation between component instances per ADR-WASM-006.

use airssys_wasm::core::config::ResourceLimits;
use airssys_wasm::runtime::ComponentResourceLimiter;
use wasmtime::ResourceLimiter;

#[test]
fn test_two_components_independent_limits() {
    let limits_a = ResourceLimits::builder()
        .max_memory_bytes(1024 * 1024)
        .max_fuel(10_000)
        .timeout_seconds(30)
        .build()
        .expect("valid memory limit");
    let mut limiter_a = ComponentResourceLimiter::new(limits_a);

    let limits_b = ResourceLimits::builder()
        .max_memory_bytes(2 * 1024 * 1024)
        .max_fuel(10_000)
        .timeout_seconds(30)
        .build()
        .expect("valid memory limit");
    let mut limiter_b = ComponentResourceLimiter::new(limits_b);

    assert!(limiter_a
        .memory_growing(0, 1024 * 1024, None)
        .expect("memory_growing should not error"));
    assert_eq!(limiter_a.current_memory_bytes(), 1024 * 1024);

    assert!(limiter_b
        .memory_growing(0, 1024 * 1024, None)
        .expect("memory_growing should not error"));
    assert_eq!(limiter_b.current_memory_bytes(), 1024 * 1024);

    assert!(!limiter_a
        .memory_growing(1024 * 1024, 2 * 1024 * 1024, None)
        .expect("memory_growing should not error"));

    assert!(limiter_b
        .memory_growing(1024 * 1024, 2 * 1024 * 1024, None)
        .expect("memory_growing should not error"));
}

#[test]
fn test_component_oom_does_not_affect_other() {
    let limits = ResourceLimits::builder()
        .max_memory_bytes(1024 * 1024)
        .max_fuel(10_000)
        .timeout_seconds(30)
        .build()
        .expect("valid memory limit");

    let mut limiter_a = ComponentResourceLimiter::new(limits);
    let mut limiter_b = ComponentResourceLimiter::new(limits);

    limiter_a
        .memory_growing(0, 1024 * 1024, None)
        .expect("memory_growing should not error");
    assert!(!limiter_a
        .memory_growing(1024 * 1024, 2 * 1024 * 1024, None)
        .expect("memory_growing should not error"));

    assert!(limiter_b
        .memory_growing(0, 512 * 1024, None)
        .expect("memory_growing should not error"));
    assert_eq!(limiter_b.current_memory_bytes(), 512 * 1024);
}

#[test]
fn test_multiple_components_concurrent_allocation() {
    let limits = ResourceLimits::builder()
        .max_memory_bytes(1024 * 1024)
        .max_fuel(10_000)
        .timeout_seconds(30)
        .build()
        .expect("valid memory limit");

    let mut limiters: Vec<_> = (0..10)
        .map(|_| ComponentResourceLimiter::new(limits))
        .collect();

    for limiter in &mut limiters {
        assert!(limiter
            .memory_growing(0, 512 * 1024, None)
            .expect("memory_growing should not error"));
        assert_eq!(limiter.current_memory_bytes(), 512 * 1024);
    }

    for limiter in &mut limiters {
        assert!(!limiter
            .memory_growing(512 * 1024, 2 * 1024 * 1024, None)
            .expect("memory_growing should not error"));
    }
}

#[test]
fn test_component_usage_isolation() {
    let limits_a = ResourceLimits::builder()
        .max_memory_bytes(1024 * 1024)
        .max_fuel(10_000)
        .timeout_seconds(30)
        .build()
        .expect("valid memory limit");
    let limits_b = ResourceLimits::builder()
        .max_memory_bytes(2 * 1024 * 1024)
        .max_fuel(10_000)
        .timeout_seconds(30)
        .build()
        .expect("valid memory limit");

    let mut limiter_a = ComponentResourceLimiter::new(limits_a);
    let mut limiter_b = ComponentResourceLimiter::new(limits_b);

    limiter_a
        .memory_growing(0, 512 * 1024, None)
        .expect("memory_growing should not error");

    assert_eq!(limiter_b.current_memory_bytes(), 0);

    limiter_b
        .memory_growing(0, 1024 * 1024, None)
        .expect("memory_growing should not error");

    assert_eq!(limiter_a.current_memory_bytes(), 512 * 1024);
    assert_eq!(limiter_b.current_memory_bytes(), 1024 * 1024);
}
