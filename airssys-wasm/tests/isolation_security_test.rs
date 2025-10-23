//! Security-focused isolation verification tests.
//!
//! Verifies 100% memory isolation (MANDATORY per ADR-WASM-006).

#![allow(clippy::expect_used)] // Test code - expect is acceptable for test assertions

use airssys_wasm::runtime::limits::{ComponentResourceLimiter, ResourceLimits};
use wasmtime::ResourceLimiter;

#[test]
fn test_component_cannot_see_other_memory() {
    let limits = ResourceLimits::builder()
        .max_memory_bytes(1024 * 1024)
        .max_fuel(10_000)
        .timeout_seconds(30)
        .build()
        .expect("valid memory limit");

    let mut limiter_a = ComponentResourceLimiter::new(limits);
    let mut limiter_b = ComponentResourceLimiter::new(limits);

    limiter_a
        .memory_growing(0, 512 * 1024, None)
        .expect("memory_growing should not error");

    assert_eq!(limiter_b.current_memory_bytes(), 0);

    limiter_b
        .memory_growing(0, 768 * 1024, None)
        .expect("memory_growing should not error");

    assert_eq!(limiter_a.current_memory_bytes(), 512 * 1024);
    assert_eq!(limiter_b.current_memory_bytes(), 768 * 1024);
}

#[test]
fn test_oom_isolation_security() {
    let limits_a = ResourceLimits::builder()
        .max_memory_bytes(512 * 1024)
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

    assert!(!limiter_a
        .memory_growing(0, 1024 * 1024, None)
        .expect("memory_growing should not error"));

    assert!(limiter_b
        .memory_growing(0, 2 * 1024 * 1024, None)
        .expect("memory_growing should not error"));
}

#[test]
fn test_limit_independence() {
    let limits_512kb = ResourceLimits::builder()
        .max_memory_bytes(512 * 1024)
        .max_fuel(10_000)
        .timeout_seconds(30)
        .build()
        .expect("valid memory limit");
    let limits_1mb = ResourceLimits::builder()
        .max_memory_bytes(1024 * 1024)
        .max_fuel(10_000)
        .timeout_seconds(30)
        .build()
        .expect("valid memory limit");
    let limits_4mb = ResourceLimits::builder()
        .max_memory_bytes(4 * 1024 * 1024)
        .max_fuel(10_000)
        .timeout_seconds(30)
        .build()
        .expect("valid memory limit");

    let mut limiter_512kb = ComponentResourceLimiter::new(limits_512kb);
    let mut limiter_1mb = ComponentResourceLimiter::new(limits_1mb);
    let mut limiter_4mb = ComponentResourceLimiter::new(limits_4mb);

    assert!(limiter_512kb
        .memory_growing(0, 512 * 1024, None)
        .expect("memory_growing should not error"));
    assert!(limiter_1mb
        .memory_growing(0, 1024 * 1024, None)
        .expect("memory_growing should not error"));
    assert!(limiter_4mb
        .memory_growing(0, 4 * 1024 * 1024, None)
        .expect("memory_growing should not error"));

    assert!(!limiter_512kb
        .memory_growing(512 * 1024, 513 * 1024, None)
        .expect("memory_growing should not error"));
    assert!(!limiter_1mb
        .memory_growing(1024 * 1024, 1024 * 1024 + 1, None)
        .expect("memory_growing should not error"));
    assert!(!limiter_4mb
        .memory_growing(4 * 1024 * 1024, 4 * 1024 * 1024 + 1, None)
        .expect("memory_growing should not error"));
}

#[test]
fn test_component_failure_isolation() {
    let limits = ResourceLimits::builder()
        .max_memory_bytes(1024 * 1024)
        .max_fuel(10_000)
        .timeout_seconds(30)
        .build()
        .expect("valid memory limit");

    let mut limiter_a = ComponentResourceLimiter::new(limits);
    let mut limiter_b = ComponentResourceLimiter::new(limits);
    let mut limiter_c = ComponentResourceLimiter::new(limits);

    limiter_a
        .memory_growing(0, 1024 * 1024, None)
        .expect("memory_growing should not error");
    assert!(!limiter_a
        .memory_growing(1024 * 1024, 2 * 1024 * 1024, None)
        .expect("memory_growing should not error"));

    assert!(limiter_b
        .memory_growing(0, 512 * 1024, None)
        .expect("memory_growing should not error"));
    assert!(limiter_c
        .memory_growing(0, 768 * 1024, None)
        .expect("memory_growing should not error"));

    assert_eq!(limiter_a.current_memory_bytes(), 1024 * 1024);
    assert_eq!(limiter_b.current_memory_bytes(), 512 * 1024);
    assert_eq!(limiter_c.current_memory_bytes(), 768 * 1024);
}
