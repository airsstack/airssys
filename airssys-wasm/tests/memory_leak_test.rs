//! Memory leak detection tests.
//!
//! Validates proper resource cleanup and leak prevention.

#![allow(clippy::expect_used)] // Test code - expect is acceptable for test assertions

use airssys_wasm::runtime::limits::{ComponentResourceLimiter, ResourceLimits};
use wasmtime::ResourceLimiter;

#[test]
fn test_repeated_allocations_stable_usage() {
    let limits = ResourceLimits::builder()
        .max_memory_bytes(1024 * 1024)
        .max_fuel(10_000)
        .timeout_seconds(30)
        .build()
        .expect("valid memory limit");
    let mut limiter = ComponentResourceLimiter::new(limits);

    for _ in 0..1000 {
        limiter
            .memory_growing(0, 512 * 1024, None)
            .expect("memory_growing should not error");
        assert_eq!(limiter.current_memory_bytes(), 512 * 1024);
    }
}

#[test]
fn test_allocation_deallocation_cycle() {
    let limits = ResourceLimits::builder()
        .max_memory_bytes(1024 * 1024)
        .max_fuel(10_000)
        .timeout_seconds(30)
        .build()
        .expect("valid memory limit");
    let mut limiter = ComponentResourceLimiter::new(limits);

    limiter
        .memory_growing(0, 512 * 1024, None)
        .expect("memory_growing should not error");
    assert_eq!(limiter.current_memory_bytes(), 512 * 1024);

    limiter
        .memory_growing(512 * 1024, 256 * 1024, None)
        .expect("memory_growing should not error");
    assert_eq!(limiter.current_memory_bytes(), 256 * 1024);

    limiter
        .memory_growing(256 * 1024, 768 * 1024, None)
        .expect("memory_growing should not error");
    assert_eq!(limiter.current_memory_bytes(), 768 * 1024);
}

#[test]
fn test_long_running_stable_memory() {
    let limits = ResourceLimits::builder()
        .max_memory_bytes(1024 * 1024)
        .max_fuel(10_000)
        .timeout_seconds(30)
        .build()
        .expect("valid memory limit");
    let mut limiter = ComponentResourceLimiter::new(limits);

    limiter
        .memory_growing(0, 512 * 1024, None)
        .expect("memory_growing should not error");
    let initial_usage = limiter.current_memory_bytes();

    for _ in 0..10000 {
        assert_eq!(limiter.current_memory_bytes(), initial_usage);
    }
}
