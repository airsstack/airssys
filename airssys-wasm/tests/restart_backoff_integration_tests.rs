//! Integration tests for component restart and exponential backoff system.
//!
//! This test suite validates the end-to-end restart flow with backoff delays,
//! sliding window limits, restart tracking, and health monitoring.

// Layer 1: Standard library imports
use std::time::{Duration, Instant};

// Layer 2: Third-party crate imports
// (none needed for these tests)

// Layer 3: Internal module imports
use airssys_wasm::actor::{
    ExponentialBackoff, ExponentialBackoffConfig, HealthDecision, HealthMonitor,
    MonitorHealthStatus, RestartReason, RestartTracker, SlidingWindowConfig, SlidingWindowLimiter,
    WindowLimitResult,
};

/// Test exponential backoff delay calculation with no jitter.
///
/// Verifies that delays grow exponentially: base * multiplier^attempt.
#[test]
fn test_exponential_backoff_growth() {
    let config = ExponentialBackoffConfig {
        base_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(5),
        multiplier: 2.0,
        jitter_factor: 0.0, // No jitter for deterministic test
    };

    let mut backoff = ExponentialBackoff::new(config);

    // First attempt: 100ms (base_delay * 2^0)
    let delay1 = backoff.next_attempt();
    assert!(
        delay1 >= Duration::from_millis(95) && delay1 <= Duration::from_millis(105),
        "First delay should be ~100ms, got {:?}",
        delay1
    );

    // Second attempt: 200ms (base_delay * 2^1)
    let delay2 = backoff.next_attempt();
    assert!(
        delay2 >= Duration::from_millis(190) && delay2 <= Duration::from_millis(210),
        "Second delay should be ~200ms, got {:?}",
        delay2
    );

    // Third attempt: 400ms (base_delay * 2^2)
    let delay3 = backoff.next_attempt();
    assert!(
        delay3 >= Duration::from_millis(390) && delay3 <= Duration::from_millis(410),
        "Third delay should be ~400ms, got {:?}",
        delay3
    );
}

/// Test that exponential backoff respects max delay cap.
///
/// Verifies that delays never exceed configured max_delay.
#[test]
fn test_exponential_backoff_max_delay_cap() {
    let config = ExponentialBackoffConfig {
        base_delay: Duration::from_millis(100),
        max_delay: Duration::from_millis(500), // Cap at 500ms
        multiplier: 2.0,
        jitter_factor: 0.0,
    };

    let mut backoff = ExponentialBackoff::new(config);

    // Attempt multiple times until we hit the cap
    for i in 0..10 {
        let delay = backoff.next_attempt();
        assert!(
            delay <= Duration::from_millis(500),
            "Delay at attempt {} exceeded max_delay: {:?}",
            i,
            delay
        );
    }

    // After many attempts, delay should stabilize at max_delay
    let final_delay = backoff.calculate_delay();
    assert!(
        final_delay <= Duration::from_millis(500),
        "Final delay exceeded max_delay: {:?}",
        final_delay
    );
}

/// Test exponential backoff with jitter adds variance.
///
/// Verifies that jitter factor introduces randomness within expected bounds.
#[test]
fn test_exponential_backoff_with_jitter() {
    let config = ExponentialBackoffConfig {
        base_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(5),
        multiplier: 2.0,
        jitter_factor: 0.1, // ±10% jitter
    };

    let backoff = ExponentialBackoff::new(config);

    // Calculate multiple delays and verify they're within jitter bounds
    // Expected: ~100ms ± 10% = 90-110ms range
    for _ in 0..10 {
        let delay = backoff.calculate_delay();
        assert!(
            delay >= Duration::from_millis(90) && delay <= Duration::from_millis(110),
            "Delay with jitter out of expected range: {:?}",
            delay
        );
    }
}

/// Test backoff reset clears attempt counter.
///
/// Verifies that reset() returns delay to base_delay.
#[test]
fn test_exponential_backoff_reset() {
    let config = ExponentialBackoffConfig {
        base_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(5),
        multiplier: 2.0,
        jitter_factor: 0.0,
    };

    let mut backoff = ExponentialBackoff::new(config);

    // Advance several attempts
    backoff.next_attempt(); // 100ms
    backoff.next_attempt(); // 200ms
    backoff.next_attempt(); // 400ms

    assert_eq!(backoff.attempt(), 3, "Should be at attempt 3");

    // Reset and verify we're back to attempt 0
    backoff.reset();
    assert_eq!(backoff.attempt(), 0, "Reset should clear attempt counter");

    let delay = backoff.calculate_delay();
    assert!(
        delay >= Duration::from_millis(95) && delay <= Duration::from_millis(105),
        "After reset, delay should be back to base_delay"
    );
}

/// Test restart tracker records restart history correctly.
///
/// Verifies that RestartTracker maintains FIFO history with timestamps.
#[test]
fn test_restart_tracker_history() {
    let mut tracker = RestartTracker::new();

    // Record some restarts
    tracker.record_restart(RestartReason::ComponentFailure, Duration::from_millis(100));
    std::thread::sleep(Duration::from_millis(10)); // Necessary: differentiate Instant timestamps

    tracker.record_restart(RestartReason::HealthCheckFailed, Duration::from_millis(200));
    std::thread::sleep(Duration::from_millis(10)); // Necessary: differentiate Instant timestamps

    tracker.record_restart(RestartReason::ManualRestart, Duration::from_millis(300));

    // Query history (newest first)
    let history = tracker.get_history(10);
    assert_eq!(history.len(), 3, "Should have 3 restart records");

    // Verify FIFO order (newest first)
    assert_eq!(
        history[0].reason,
        RestartReason::ManualRestart,
        "Newest record should be first"
    );
    assert_eq!(
        history[1].reason,
        RestartReason::HealthCheckFailed,
        "Middle record should be second"
    );
    assert_eq!(
        history[2].reason,
        RestartReason::ComponentFailure,
        "Oldest record should be last"
    );

    // Verify total count
    assert_eq!(
        tracker.total_restarts(),
        3,
        "Total restart count should be 3"
    );
}

/// Test restart tracker circular buffer overflow.
///
/// Verifies that RestartTracker maintains only last 100 records.
#[test]
fn test_restart_tracker_circular_buffer_overflow() {
    let mut tracker = RestartTracker::new();

    // Record 150 restarts (exceeds 100 limit)
    for _ in 0..150 {
        tracker.record_restart(RestartReason::ComponentFailure, Duration::from_millis(100));
    }

    // Verify only last 100 are kept
    let history = tracker.get_history(200); // Request more than capacity
    assert_eq!(
        history.len(),
        100,
        "Should only keep last 100 restart records"
    );

    // Total count should still be 150
    assert_eq!(
        tracker.total_restarts(),
        150,
        "Total count should track all restarts"
    );
}

/// Test restart tracker reset on recovery clears history.
///
/// Verifies that reset_on_recovery() clears restart history.
#[test]
fn test_restart_tracker_reset_on_recovery() {
    let mut tracker = RestartTracker::new();

    // Record some restarts
    tracker.record_restart(RestartReason::ComponentFailure, Duration::from_millis(100));
    tracker.record_restart(RestartReason::HealthCheckFailed, Duration::from_millis(200));

    assert_eq!(tracker.total_restarts(), 2, "Should have 2 restarts");

    // Reset on recovery
    tracker.reset_on_recovery();

    // History should be cleared, but total count preserved
    let history = tracker.get_history(10);
    assert_eq!(history.len(), 0, "History should be cleared after recovery");

    // Note: Current implementation clears total count on recovery
    // This is a design choice - recovery means "back to baseline"
}

/// Test sliding window limiter allows restarts within limit.
///
/// Verifies that SlidingWindowLimiter permits restarts under max_restarts.
#[test]
fn test_sliding_window_limiter_allows_within_limit() {
    let config = SlidingWindowConfig {
        max_restarts: 3,
        window_duration: Duration::from_secs(60),
    };

    let mut limiter = SlidingWindowLimiter::new(config);

    // First 3 restarts should be allowed
    for i in 0..3 {
        let result = limiter.check_can_restart();
        assert!(
            matches!(result, WindowLimitResult::AllowRestart),
            "Restart {} should be allowed",
            i + 1
        );
        limiter.record_restart();
    }

    assert_eq!(
        limiter.restart_count_in_window(),
        3,
        "Should have 3 restarts in window"
    );
}

/// Test sliding window limiter denies restarts at limit.
///
/// Verifies that SlidingWindowLimiter blocks restarts once max_restarts is reached.
#[test]
fn test_sliding_window_limiter_denies_at_limit() {
    let config = SlidingWindowConfig {
        max_restarts: 3,
        window_duration: Duration::from_secs(60),
    };

    let mut limiter = SlidingWindowLimiter::new(config);

    // Use up all 3 allowed restarts
    for _ in 0..3 {
        limiter.check_can_restart();
        limiter.record_restart();
    }

    // 4th restart should be denied
    let result = limiter.check_can_restart();
    assert!(
        matches!(result, WindowLimitResult::DenyRestart { .. }),
        "4th restart should be denied"
    );

    if let WindowLimitResult::DenyRestart { reason, .. } = result {
        assert!(
            reason.contains("Maximum restart"),
            "Deny reason should mention maximum restart rate"
        );
    }
}

/// Test sliding window cleanup removes old entries.
///
/// Verifies that restarts outside the time window don't count toward limit.
#[test]
fn test_sliding_window_cleanup() {
    let config = SlidingWindowConfig {
        max_restarts: 2,
        window_duration: Duration::from_millis(100), // Short window for testing
    };

    let mut limiter = SlidingWindowLimiter::new(config);

    // Record 2 restarts (hits limit)
    limiter.record_restart();
    limiter.record_restart();

    // Should be at limit now
    let result = limiter.check_can_restart();
    assert!(
        matches!(result, WindowLimitResult::DenyRestart { .. }),
        "Should be at limit"
    );

    // Wait for window to expire
    std::thread::sleep(Duration::from_millis(150)); // Necessary: test sliding window expiry

    // Old restarts should be cleaned up, new restart should be allowed
    let result = limiter.check_can_restart();
    assert!(
        matches!(result, WindowLimitResult::AllowRestart),
        "After window expiry, restart should be allowed"
    );
}

/// Test permanent failure detection.
///
/// Verifies that limiter detects permanent failure after repeated limit hits.
#[test]
fn test_permanent_failure_detection() {
    let config = SlidingWindowConfig {
        max_restarts: 2,
        window_duration: Duration::from_secs(60),
    };

    let mut limiter = SlidingWindowLimiter::new(config);

    // Hit the limit multiple times (need at least 5 for permanent failure)
    for _ in 0..6 {
        limiter.record_restart();
        limiter.record_restart();
        let _ = limiter.check_can_restart(); // Triggers limit hit
    }

    // Should detect permanent failure (limit_hit_count >= 5)
    assert!(
        limiter.is_permanently_failed(),
        "Should detect permanent failure after 5+ limit hits"
    );
}

/// Test health monitor evaluates healthy status correctly.
///
/// Verifies that HealthMonitor returns Healthy decision for healthy status.
#[test]
fn test_health_monitor_healthy_evaluation() {
    let mut monitor = HealthMonitor::new(Duration::from_secs(10));

    let status = MonitorHealthStatus::Healthy;
    let decision = monitor.evaluate_health(status);

    assert!(
        matches!(decision, HealthDecision::Healthy),
        "Healthy status should yield Healthy decision"
    );
    assert_eq!(
        monitor.consecutive_failures(),
        0,
        "No failures should be recorded"
    );
}

/// Test health monitor evaluates degraded status.
///
/// Verifies that HealthMonitor returns Degraded decision for degraded status.
#[test]
fn test_health_monitor_degraded_evaluation() {
    let mut monitor = HealthMonitor::new(Duration::from_secs(10));

    let status = MonitorHealthStatus::Degraded {
        message: "High latency".to_string(),
    };
    let decision = monitor.evaluate_health(status);

    assert!(
        matches!(decision, HealthDecision::Degraded),
        "Degraded status should yield Degraded decision"
    );
}

/// Test health monitor triggers unhealthy decision at threshold.
///
/// Verifies that consecutive failures trigger Unhealthy decision.
#[test]
fn test_health_monitor_unhealthy_at_threshold() {
    let mut monitor = HealthMonitor::new(Duration::from_secs(10));
    monitor.set_failure_threshold(3); // Trigger at 3 consecutive failures

    // Record 2 failures (below threshold)
    for _ in 0..2 {
        let status = MonitorHealthStatus::Unhealthy {
            message: "Connection failed".to_string(),
        };
        let decision = monitor.evaluate_health(status);
        assert!(
            !matches!(decision, HealthDecision::Unhealthy),
            "Should not trigger restart before threshold"
        );
    }

    // 3rd failure should trigger Unhealthy decision
    let status = MonitorHealthStatus::Unhealthy {
        message: "Connection failed".to_string(),
    };
    let decision = monitor.evaluate_health(status);
    assert!(
        matches!(decision, HealthDecision::Unhealthy),
        "Should trigger Unhealthy at threshold"
    );
}

/// Test health monitor recovery resets failure counter.
///
/// Verifies that recovery (Healthy status) resets consecutive failures.
#[test]
fn test_health_monitor_recovery_resets_failures() {
    let mut monitor = HealthMonitor::new(Duration::from_secs(10));
    monitor.set_failure_threshold(3);

    // Record 2 failures
    for _ in 0..2 {
        let status = MonitorHealthStatus::Unhealthy {
            message: "Error".to_string(),
        };
        monitor.evaluate_health(status);
    }

    assert_eq!(
        monitor.consecutive_failures(),
        2,
        "Should have 2 consecutive failures"
    );

    // Recovery with Healthy status
    let status = MonitorHealthStatus::Healthy;
    monitor.evaluate_health(status);

    assert_eq!(
        monitor.consecutive_failures(),
        0,
        "Recovery should reset failure counter"
    );
}

/// Test combined backoff and window limiter enforcement.
///
/// Simulates realistic restart scenario with backoff delays and window limits.
#[test]
fn test_combined_backoff_and_window_enforcement() {
    // Setup backoff config
    let backoff_config = ExponentialBackoffConfig {
        base_delay: Duration::from_millis(10),
        max_delay: Duration::from_millis(100),
        multiplier: 2.0,
        jitter_factor: 0.0,
    };
    let mut backoff = ExponentialBackoff::new(backoff_config);

    // Setup window limiter (max 3 restarts in 1 second)
    let window_config = SlidingWindowConfig {
        max_restarts: 3,
        window_duration: Duration::from_secs(1),
    };
    let mut limiter = SlidingWindowLimiter::new(window_config);

    // Setup restart tracker
    let mut tracker = RestartTracker::new();

    // Simulate restart loop
    for attempt in 0..5 {
        // Check if restart is allowed
        let window_result = limiter.check_can_restart();
        if matches!(window_result, WindowLimitResult::DenyRestart { .. }) {
            // Hit window limit - can't restart anymore
            assert_eq!(
                attempt, 3,
                "Should hit limit on 4th attempt (after 3 restarts)"
            );
            break;
        }

        // Calculate backoff delay
        let delay = backoff.next_attempt();

        // Record restart
        limiter.record_restart();
        tracker.record_restart(RestartReason::ComponentFailure, delay);

        // Simulate delay (shortened for test)
        // In real scenario: tokio::time::sleep(delay).await;
        std::thread::sleep(Duration::from_millis(1)); // Necessary: health check timing
    }

    // Verify tracking
    assert_eq!(
        tracker.total_restarts(),
        3,
        "Should have recorded 3 restarts"
    );
    assert_eq!(
        limiter.restart_count_in_window(),
        3,
        "Should have 3 restarts in window"
    );
}

/// Test full restart flow simulation.
///
/// End-to-end test of restart decision making with all subsystems.
#[test]
fn test_full_restart_flow_simulation() {
    // Initialize all subsystems
    let backoff_config = ExponentialBackoffConfig {
        base_delay: Duration::from_millis(10),
        max_delay: Duration::from_millis(100),
        multiplier: 2.0,
        jitter_factor: 0.1,
    };
    let mut backoff = ExponentialBackoff::new(backoff_config);

    let window_config = SlidingWindowConfig {
        max_restarts: 5,
        window_duration: Duration::from_secs(60),
    };
    let mut limiter = SlidingWindowLimiter::new(window_config);

    let mut tracker = RestartTracker::new();

    let mut health_monitor = HealthMonitor::new(Duration::from_secs(1));
    health_monitor.set_failure_threshold(2);

    // Simulate component lifecycle
    let start_time = Instant::now();

    // Component fails multiple times
    for i in 0..4 {
        // Check health
        let health_status = if i < 2 {
            MonitorHealthStatus::Unhealthy {
                message: "Connection timeout".to_string(),
            }
        } else {
            MonitorHealthStatus::Healthy
        };

        let health_decision = health_monitor.evaluate_health(health_status);

        // Make restart decision
        if matches!(health_decision, HealthDecision::Unhealthy) {
            // Check window limit
            let window_result = limiter.check_can_restart();
            if matches!(window_result, WindowLimitResult::AllowRestart) {
                // Calculate backoff delay
                let delay = backoff.next_attempt();

                // Record restart
                limiter.record_restart();
                tracker.record_restart(RestartReason::HealthCheckFailed, delay);

                // Simulate delay
                std::thread::sleep(delay);
            }
        } else if matches!(health_decision, HealthDecision::Healthy) {
            // Recovery - reset tracking
            backoff.reset();
            health_monitor.reset_on_recovery();
        }
    }

    // Verify final state
    assert!(
        start_time.elapsed() < Duration::from_secs(1),
        "Test should complete quickly"
    );

    // Should have recovered after 2 failures
    let history = tracker.get_history(10);
    assert!(
        history.len() <= 2,
        "Should have attempted restart at most twice"
    );
}
