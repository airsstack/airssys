#![allow(clippy::unwrap_used)] // Test code is allowed to use unwrap for clarity

//! Component memory boundary tests.
//!
//! Verifies single-component memory limit enforcement per ADR-WASM-002.

#![allow(clippy::expect_used)] // Test code - expect is acceptable for test assertions

use airssys_wasm::runtime::limits::{ComponentResourceLimiter, ResourceLimits};
use wasmtime::ResourceLimiter;

#[test]
fn test_single_component_respects_limit() {
    let limits = ResourceLimits::builder()
        .max_memory_bytes(1024 * 1024)
        .max_fuel(10_000)
        .timeout_seconds(30)
        .build()
        .expect("valid memory limit");
    let mut limiter = ComponentResourceLimiter::new(limits);

    assert!(limiter
        .memory_growing(0, 512 * 1024, None)
        .expect("memory_growing should not error"));

    assert!(limiter
        .memory_growing(512 * 1024, 1024 * 1024, None)
        .expect("memory_growing should not error"));

    assert!(!limiter
        .memory_growing(1024 * 1024, 2 * 1024 * 1024, None)
        .expect("memory_growing should not error"));
}

#[test]
fn test_oom_at_maximum_allocation() {
    let limits = ResourceLimits::builder()
        .max_memory_bytes(ResourceLimits::MAX_MEMORY_BYTES)
        .max_fuel(10_000)
        .timeout_seconds(30)
        .build()
        .expect("valid memory limit");
    let mut limiter = ComponentResourceLimiter::new(limits);

    assert!(limiter
        .memory_growing(0, 4 * 1024 * 1024, None)
        .expect("memory_growing should not error"));

    assert!(!limiter
        .memory_growing(0, (4 * 1024 * 1024) + 1, None)
        .expect("memory_growing should not error"));
}

#[test]
fn test_oom_at_minimum_allocation() {
    let limits = ResourceLimits::builder()
        .max_memory_bytes(ResourceLimits::MIN_MEMORY_BYTES)
        .max_fuel(10_000)
        .timeout_seconds(30)
        .build()
        .expect("valid memory limit");
    let mut limiter = ComponentResourceLimiter::new(limits);

    assert!(limiter
        .memory_growing(0, 512 * 1024, None)
        .expect("memory_growing should not error"));

    assert!(!limiter
        .memory_growing(0, (512 * 1024) + 1, None)
        .expect("memory_growing should not error"));
}

#[test]
fn test_gradual_memory_growth() {
    let limits = ResourceLimits::builder()
        .max_memory_bytes(1024 * 1024)
        .max_fuel(10_000)
        .timeout_seconds(30)
        .build()
        .expect("valid memory limit");
    let mut limiter = ComponentResourceLimiter::new(limits);

    assert!(limiter
        .memory_growing(0, 256 * 1024, None)
        .expect("memory_growing should not error"));
    assert_eq!(limiter.current_memory_bytes(), 256 * 1024);

    assert!(limiter
        .memory_growing(256 * 1024, 512 * 1024, None)
        .expect("memory_growing should not error"));
    assert_eq!(limiter.current_memory_bytes(), 512 * 1024);

    assert!(limiter
        .memory_growing(512 * 1024, 768 * 1024, None)
        .expect("memory_growing should not error"));
    assert_eq!(limiter.current_memory_bytes(), 768 * 1024);

    assert!(limiter
        .memory_growing(768 * 1024, 1024 * 1024, None)
        .expect("memory_growing should not error"));

    assert!(!limiter
        .memory_growing(1024 * 1024, 1024 * 1024 + 1, None)
        .expect("memory_growing should not error"));
}

#[test]
fn test_memory_usage_tracking() {
    let limits = ResourceLimits::builder()
        .max_memory_bytes(1024 * 1024)
        .max_fuel(10_000)
        .timeout_seconds(30)
        .build()
        .expect("valid memory limit");
    let mut limiter = ComponentResourceLimiter::new(limits);

    assert_eq!(limiter.current_memory_bytes(), 0);

    limiter
        .memory_growing(0, 512 * 1024, None)
        .expect("memory_growing should not error");
    assert_eq!(limiter.current_memory_bytes(), 512 * 1024);

    let metrics = limiter.metrics();
    assert_eq!(metrics.usage_percentage(), 50.0);
}
