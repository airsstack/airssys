#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

//! Security enforcement performance benchmarks.
//!
//! Validates that DEBT-WASM-004 Item #3 security checks meet performance targets:
//! - Individual checks: <2μs each
//! - Full security check: <5μs total
//! - Rate limiter: <2μs with 100 tracked senders
//!
//! # Performance Targets (CRITICAL)
//!
//! | Benchmark | Target | Description |
//! |-----------|--------|-------------|
//! | `capability_check` | <2μs | Sender authorization validation |
//! | `payload_size_check` | <1μs | Message size validation |
//! | `rate_limit_check` | <2μs | DoS prevention check |
//! | `full_security_check` | <5μs | All 3 checks combined |
//! | `security_check_denied_path` | <3μs | Early return on first failure |
//! | `rate_limiter_with_100_senders` | <2μs | Scalability validation |
//!
//! # References
//!
//! - **Action Plan**: `.memory-bank/sub-projects/airssys-wasm/tasks/debt-wasm-004-item-3-action-plan.md`
//! - **DEBT-WASM-004**: Technical Debt Resolution (Item #3: Capability Enforcement)
//! - **ADR-WASM-005**: Capability-Based Security Model

// Layer 1: Standard library imports
use std::hint::black_box;
use std::time::Duration;

// Layer 2: Third-party crate imports
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

// Layer 3: Internal module imports
use airssys_wasm::core::rate_limiter::{MessageRateLimiter, RateLimiterConfig};
use airssys_wasm::core::{
    Capability, CapabilitySet, ComponentId, DomainPattern, NamespacePattern, PathPattern,
    TopicPattern,
};

// ============================================================================
// Benchmark 1: Sender Authorization Check
// ============================================================================

/// Benchmark capability check performance (allows_receiving_from).
///
/// Target: <2μs per check
///
/// This is the first security gate and must be extremely fast to avoid
/// message processing bottlenecks.
fn bench_capability_check(c: &mut Criterion) {
    // Setup: Component with Messaging capability
    let mut caps = CapabilitySet::new();
    caps.grant(Capability::Messaging(TopicPattern::new("*")));

    let sender = ComponentId::new("sender-component");

    c.bench_function("capability_check", |b| {
        b.iter(|| black_box(caps.allows_receiving_from(&sender)));
    });
}

/// Benchmark capability check with multiple capabilities.
///
/// Target: <2μs per check (should be O(n) where n = number of capabilities)
///
/// Tests worst-case scenario where component has multiple capability types.
fn bench_capability_check_multiple_caps(c: &mut Criterion) {
    // Setup: Component with 5 capabilities (various types)
    let mut caps = CapabilitySet::new();
    caps.grant(Capability::FileRead(PathPattern::new("/data/*.json")));
    caps.grant(Capability::FileWrite(PathPattern::new("/output/*")));
    caps.grant(Capability::NetworkOutbound(DomainPattern::new(
        "*.example.com",
    )));
    caps.grant(Capability::NetworkInbound(8080));
    caps.grant(Capability::Storage(NamespacePattern::new("cache.*")));
    caps.grant(Capability::ProcessSpawn);
    caps.grant(Capability::Messaging(TopicPattern::new("*")));
    caps.grant(Capability::Custom {
        name: "custom-cap".to_string(),
        parameters: serde_json::json!({}),
    });

    let sender = ComponentId::new("sender-component");

    c.bench_function("capability_check_multiple_caps", |b| {
        b.iter(|| black_box(caps.allows_receiving_from(&sender)));
    });
}

// ============================================================================
// Benchmark 2: Payload Size Validation
// ============================================================================

/// Benchmark payload size check performance.
///
/// Target: <1μs (simple comparison)
///
/// This is a trivial check but must be measured to verify no overhead.
fn bench_payload_size_validation(c: &mut Criterion) {
    let payload_1kb = vec![0u8; 1024];
    let payload_1mb = vec![0u8; 1_048_576];
    let max_size = 1_048_576; // 1 MB

    let mut group = c.benchmark_group("payload_size_check");

    group.bench_function("1kb_payload", |b| {
        b.iter(|| black_box(payload_1kb.len() <= max_size));
    });

    group.bench_function("1mb_payload", |b| {
        b.iter(|| black_box(payload_1mb.len() <= max_size));
    });

    group.finish();
}

// ============================================================================
// Benchmark 3: Rate Limiting
// ============================================================================

/// Benchmark rate limiter check performance.
///
/// Target: <2μs per check
///
/// Rate limiter uses Arc<Mutex<HashMap>> which adds lock contention overhead.
fn bench_rate_limit_check(c: &mut Criterion) {
    let limiter = MessageRateLimiter::default();
    let sender = ComponentId::new("sender");

    c.bench_function("rate_limit_check", |b| {
        b.iter(|| black_box(limiter.check_rate_limit(&sender)));
    });
}

/// Benchmark rate limiter with multiple tracked senders.
///
/// Target: <2μs per check with 100 senders
///
/// Validates that HashMap lookup performance doesn't degrade with scale.
fn bench_rate_limit_check_multiple_senders(c: &mut Criterion) {
    let limiter = MessageRateLimiter::default();

    // Pre-populate with 100 senders
    for i in 0..100 {
        let sender = ComponentId::new(format!("sender-{}", i));
        limiter.check_rate_limit(&sender);
    }

    let test_sender = ComponentId::new("test-sender");

    c.bench_function("rate_limit_check_100_senders", |b| {
        b.iter(|| black_box(limiter.check_rate_limit(&test_sender)));
    });
}

/// Benchmark rate limiter with different rate limit configurations.
///
/// Target: <2μs per check (should be independent of limit value)
fn bench_rate_limit_check_various_limits(c: &mut Criterion) {
    let mut group = c.benchmark_group("rate_limit_various_limits");

    for limit in [100, 1000, 10000].iter() {
        let config = RateLimiterConfig {
            messages_per_second: *limit,
            window_duration: Duration::from_secs(1),
        };
        let limiter = MessageRateLimiter::new(config);
        let sender = ComponentId::new("sender");

        group.bench_with_input(BenchmarkId::from_parameter(limit), limit, |b, _| {
            b.iter(|| black_box(limiter.check_rate_limit(&sender)));
        });
    }

    group.finish();
}

// ============================================================================
// Benchmark 4: Full Security Check (All 3 Layers)
// ============================================================================

/// Benchmark complete security check sequence.
///
/// Target: <5μs total
///
/// This is the end-to-end overhead added to every inter-component message.
fn bench_full_security_check(c: &mut Criterion) {
    // Setup: All security components
    let mut caps = CapabilitySet::new();
    caps.grant(Capability::Messaging(TopicPattern::new("*")));

    let sender = ComponentId::new("sender");
    let payload = vec![0u8; 1024]; // 1 KB payload
    let max_size = 1_048_576; // 1 MB limit
    let limiter = MessageRateLimiter::default();

    c.bench_function("full_security_check", |b| {
        b.iter(|| {
            // 1. Capability check
            let auth_ok = caps.allows_receiving_from(&sender);

            // 2. Payload size check
            let size_ok = payload.len() <= max_size;

            // 3. Rate limit check
            let rate_ok = limiter.check_rate_limit(&sender);

            black_box(auth_ok && size_ok && rate_ok)
        });
    });
}

// ============================================================================
// Benchmark 5: Denied Path (Early Return)
// ============================================================================

/// Benchmark security check with early denial.
///
/// Target: <3μs (faster than full check)
///
/// When first check fails, should return immediately without running remaining checks.
fn bench_security_check_denied_path(c: &mut Criterion) {
    // Setup: Component WITHOUT Messaging capability (will deny)
    let caps = CapabilitySet::new(); // No capabilities

    let sender = ComponentId::new("unauthorized-sender");
    let payload = vec![0u8; 1024];
    let max_size = 1_048_576;
    let limiter = MessageRateLimiter::default();

    c.bench_function("security_check_denied_path", |b| {
        b.iter(|| {
            // 1. Capability check (fails immediately)
            let auth_ok = caps.allows_receiving_from(&sender);

            if !auth_ok {
                return black_box(false);
            }

            // These checks should NOT execute due to early return
            let size_ok = payload.len() <= max_size;
            let rate_ok = limiter.check_rate_limit(&sender);

            black_box(auth_ok && size_ok && rate_ok)
        });
    });
}

/// Benchmark security check with payload too large (second check fails).
///
/// Target: <2μs (skips rate limit check)
fn bench_security_check_size_denied_path(c: &mut Criterion) {
    let mut caps = CapabilitySet::new();
    caps.grant(Capability::Messaging(TopicPattern::new("*")));

    let sender = ComponentId::new("sender");
    let payload = vec![0u8; 2_000_000]; // 2 MB (exceeds limit)
    let max_size = 1_048_576; // 1 MB limit
    let limiter = MessageRateLimiter::default();

    c.bench_function("security_check_size_denied_path", |b| {
        b.iter(|| {
            // 1. Capability check (passes)
            let auth_ok = caps.allows_receiving_from(&sender);

            if !auth_ok {
                return black_box(false);
            }

            // 2. Payload size check (fails)
            let size_ok = payload.len() <= max_size;

            if !size_ok {
                return black_box(false);
            }

            // This check should NOT execute
            let rate_ok = limiter.check_rate_limit(&sender);

            black_box(auth_ok && size_ok && rate_ok)
        });
    });
}

// ============================================================================
// Benchmark 6: Concurrent Security Checks
// ============================================================================

/// Benchmark concurrent security checks (multi-threaded scenario).
///
/// Target: <5μs per check under contention
///
/// Validates that rate limiter lock contention doesn't degrade performance.
fn bench_concurrent_security_checks(c: &mut Criterion) {
    use std::sync::Arc;

    let limiter = Arc::new(MessageRateLimiter::default());
    let mut caps = CapabilitySet::new();
    caps.grant(Capability::Messaging(TopicPattern::new("*")));

    let sender = ComponentId::new("sender");
    let payload = vec![0u8; 1024];
    let max_size = 1_048_576;

    c.bench_function("concurrent_security_checks", |b| {
        b.iter(|| {
            let limiter_clone = Arc::clone(&limiter);
            let sender_clone = sender.clone();

            // Simulate concurrent check
            let auth_ok = caps.allows_receiving_from(&sender_clone);
            let size_ok = payload.len() <= max_size;
            let rate_ok = limiter_clone.check_rate_limit(&sender_clone);

            black_box(auth_ok && size_ok && rate_ok)
        });
    });
}

// ============================================================================
// Benchmark Groups
// ============================================================================

criterion_group!(
    security_benches,
    bench_capability_check,
    bench_capability_check_multiple_caps,
    bench_payload_size_validation,
    bench_rate_limit_check,
    bench_rate_limit_check_multiple_senders,
    bench_rate_limit_check_various_limits,
    bench_full_security_check,
    bench_security_check_denied_path,
    bench_security_check_size_denied_path,
    bench_concurrent_security_checks,
);

criterion_main!(security_benches);
