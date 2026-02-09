//! Integration tests for SupervisorConfig with airssys-rt types.
//!
//! These tests verify that SupervisorConfig correctly integrates with
//! airssys-rt supervisor framework types, producing valid RestartBackoff
//! instances and storing compatible airssys-rt configuration values.

// Layer 1: Standard library imports
use std::thread;
use std::time::Duration;

// Layer 2: Third-party crate imports
use airssys_rt::supervisor::{RestartPolicy, ShutdownPolicy};

// Layer 3: Crate under test
use airssys_wasm::component::supervisor::{
    BackoffConfig, SupervisorConfig, WasmSupervisionStrategy,
};

/// Test 1: Default SupervisorConfig creates a valid RestartBackoff.
///
/// Verifies that the default config creates a RestartBackoff with correct
/// max_restarts (3) and sliding window behavior.
#[test]
fn test_default_config_creates_valid_restart_backoff() {
    let config = SupervisorConfig::default();
    let mut backoff = config.to_restart_backoff();

    // Initially not exceeded
    assert!(!backoff.is_limit_exceeded());
    assert_eq!(backoff.restart_count(), 0);

    // Record restarts up to the default limit (3)
    backoff.record_restart();
    assert_eq!(backoff.restart_count(), 1);
    assert!(!backoff.is_limit_exceeded());

    backoff.record_restart();
    assert_eq!(backoff.restart_count(), 2);
    assert!(!backoff.is_limit_exceeded());

    backoff.record_restart();
    assert_eq!(backoff.restart_count(), 3);
    // At limit (3 restarts with max_restarts=3), should be exceeded
    assert!(backoff.is_limit_exceeded());
}

/// Test 2: Custom config creates correct RestartBackoff with custom delays.
///
/// Build a custom config with non-default values and verify the backoff
/// delay calculation returns expected exponential values.
#[test]
fn test_custom_config_creates_correct_restart_backoff() {
    let result = SupervisorConfig::builder()
        .max_restarts(5)
        .restart_window(Duration::from_secs(120))
        .backoff(BackoffConfig::new(
            Duration::from_millis(200),
            Duration::from_secs(60),
        ))
        .build();

    assert!(result.is_ok());
    if let Ok(config) = result {
        let mut backoff = config.to_restart_backoff();

        // First restart: 200ms * 2^1 = 400ms
        backoff.record_restart();
        assert_eq!(backoff.calculate_delay(), Duration::from_millis(400));

        // Second restart: 200ms * 2^2 = 800ms
        backoff.record_restart();
        assert_eq!(backoff.calculate_delay(), Duration::from_millis(800));

        // Third restart: 200ms * 2^3 = 1600ms
        backoff.record_restart();
        assert_eq!(backoff.calculate_delay(), Duration::from_millis(1600));
    }
}

/// Test 3: RestartPolicy values are airssys-rt compatible.
///
/// Verify that all three RestartPolicy values can be stored and retrieved
/// via the builder/accessor, and that should_restart() works correctly.
#[test]
fn test_config_restart_policy_is_airssys_rt_compatible() {
    // Test Permanent: always restart
    let result = SupervisorConfig::builder()
        .restart_policy(RestartPolicy::Permanent)
        .build();
    assert!(result.is_ok());
    if let Ok(config) = result {
        let policy = config.restart_policy();
        assert!(policy.should_restart(true)); // Restart on error
        assert!(policy.should_restart(false)); // Restart on normal exit
    }

    // Test Transient: restart only on error
    let result = SupervisorConfig::builder()
        .restart_policy(RestartPolicy::Transient)
        .build();
    assert!(result.is_ok());
    if let Ok(config) = result {
        let policy = config.restart_policy();
        assert!(policy.should_restart(true)); // Restart on error
        assert!(!policy.should_restart(false)); // Don't restart on normal exit
    }

    // Test Temporary: never restart
    let result = SupervisorConfig::builder()
        .restart_policy(RestartPolicy::Temporary)
        .build();
    assert!(result.is_ok());
    if let Ok(config) = result {
        let policy = config.restart_policy();
        assert!(!policy.should_restart(true)); // Never restart
        assert!(!policy.should_restart(false)); // Never restart
    }
}

/// Test 4: ShutdownPolicy values are airssys-rt compatible.
///
/// Verify that all ShutdownPolicy variants can be stored and retrieved,
/// and that timeout() returns the correct value.
#[test]
fn test_config_shutdown_policy_is_airssys_rt_compatible() {
    // Test Graceful with custom timeout
    let result = SupervisorConfig::builder()
        .shutdown_policy(ShutdownPolicy::Graceful(Duration::from_secs(10)))
        .build();
    assert!(result.is_ok());
    if let Ok(config) = result {
        let policy = config.shutdown_policy();
        assert_eq!(policy.timeout(), Some(Duration::from_secs(10)));
        assert!(policy.is_graceful());
    }

    // Test Immediate
    let result = SupervisorConfig::builder()
        .shutdown_policy(ShutdownPolicy::Immediate)
        .build();
    assert!(result.is_ok());
    if let Ok(config) = result {
        let policy = config.shutdown_policy();
        assert_eq!(policy.timeout(), Some(Duration::ZERO));
        assert!(!policy.is_graceful());
    }

    // Test Infinity
    let result = SupervisorConfig::builder()
        .shutdown_policy(ShutdownPolicy::Infinity)
        .build();
    assert!(result.is_ok());
    if let Ok(config) = result {
        let policy = config.shutdown_policy();
        assert_eq!(policy.timeout(), None);
        assert!(policy.is_graceful());
    }
}

/// Test 5: Builder produces validated config that integrates with airssys-rt.
///
/// Use the builder to construct a fully customized config, verify it passes
/// validation, convert to RestartBackoff, and verify behavior.
#[test]
fn test_builder_produces_validated_config_for_airssys_rt() {
    let result = SupervisorConfig::builder()
        .strategy(WasmSupervisionStrategy::OneForAll)
        .restart_policy(RestartPolicy::Permanent)
        .shutdown_policy(ShutdownPolicy::Graceful(Duration::from_secs(10)))
        .max_restarts(5)
        .restart_window(Duration::from_secs(120))
        .backoff(BackoffConfig::new(
            Duration::from_millis(500),
            Duration::from_secs(30),
        ))
        .build();

    assert!(result.is_ok());
    if let Ok(config) = result {
        // Validation passed (build() validates)
        assert!(config.validate().is_ok());

        // Convert to RestartBackoff and verify
        let mut backoff = config.to_restart_backoff();

        // With base_delay=500ms, first restart: 500ms * 2^1 = 1000ms
        backoff.record_restart();
        assert_eq!(backoff.calculate_delay(), Duration::from_millis(1000));

        // Should not be exceeded at 1 restart (max is 5)
        assert!(!backoff.is_limit_exceeded());

        // Fill up to the limit
        backoff.record_restart();
        backoff.record_restart();
        backoff.record_restart();
        backoff.record_restart();

        // Now at 5 restarts with max_restarts=5, should be exceeded
        assert!(backoff.is_limit_exceeded());
    }
}

/// Test 6: RestartBackoff respects the sliding window with real time.
///
/// Create a config with a very short restart window, fill it, then sleep
/// past the window and verify the limit is no longer exceeded.
///
/// Note: This test uses real time-based sleep. The margins are generous
/// (200ms window, 300ms sleep) to avoid flakiness on slow CI runners.
#[test]
fn test_restart_backoff_respects_sliding_window() {
    let result = SupervisorConfig::builder()
        .max_restarts(2)
        .restart_window(Duration::from_millis(200))
        .build();

    assert!(result.is_ok());
    if let Ok(config) = result {
        let mut backoff = config.to_restart_backoff();

        // Fill to the limit
        backoff.record_restart();
        backoff.record_restart();
        assert!(backoff.is_limit_exceeded());

        // Wait generously past the sliding window to avoid CI flakiness
        thread::sleep(Duration::from_millis(300));

        // Restarts should have expired from the window
        assert!(!backoff.is_limit_exceeded());
        assert_eq!(backoff.restart_count(), 0);
    }
}
