//! Example: Supervising WASM components with automatic restart
//!
//! Demonstrates:
//! - Creating supervised components
//! - Restart policy configuration (Permanent, Transient, Temporary)
//! - Backoff strategies (Immediate, Linear, Exponential)
//! - Supervision tree visualization
//! - Monitoring component health

use airssys_wasm::actor::{BackoffStrategy, RestartPolicy, SupervisorConfig};
use std::time::Duration;

fn main() {
    println!("=== WASM Component Supervision Examples ===\n");

    // Example 1: Permanent policy (always restart)
    println!("Example 1: Permanent Policy");
    println!("============================");
    let permanent_config = SupervisorConfig::permanent()
        .with_max_restarts(5)
        .with_time_window(Duration::from_secs(60));
    println!("Policy: {}", permanent_config.restart_policy);
    println!("Max restarts: {} in {:?}", permanent_config.max_restarts, permanent_config.time_window);
    println!("Use case: Critical components (databases, API servers, core services)\n");

    // Example 2: Transient policy (restart on error only)
    println!("Example 2: Transient Policy");
    println!("===========================");
    let transient_config = SupervisorConfig::transient()
        .with_backoff(BackoffStrategy::Exponential {
            base_delay: Duration::from_millis(100),
            multiplier: 1.5,
            max_delay: Duration::from_secs(30),
        });
    println!("Policy: {}", transient_config.restart_policy);
    println!("Backoff: Exponential (1.5x multiplier, 30s cap)");
    println!("Use case: Workers that may complete successfully\n");

    // Example 3: Temporary policy (never restart)
    println!("Example 3: Temporary Policy");
    println!("===========================");
    let temporary_config = SupervisorConfig::temporary();
    println!("Policy: {}", temporary_config.restart_policy);
    println!("Use case: One-shot tasks, temporary processes\n");

    // Example 4: Restart policies in action
    println!("Example 4: Restart Decision Logic");
    println!("=================================");

    let permanent = RestartPolicy::Permanent;
    let transient = RestartPolicy::Transient;
    let temporary = RestartPolicy::Temporary;

    println!("\nError exit (abnormal termination):");
    println!("  Permanent:  should_restart(true) = {}", permanent.should_restart(true));
    println!("  Transient:  should_restart(true) = {}", transient.should_restart(true));
    println!("  Temporary:  should_restart(true) = {}", temporary.should_restart(true));

    println!("\nNormal exit (graceful shutdown):");
    println!("  Permanent:  should_restart(false) = {}", permanent.should_restart(false));
    println!("  Transient:  should_restart(false) = {}", transient.should_restart(false));
    println!("  Temporary:  should_restart(false) = {}", temporary.should_restart(false));

    // Example 5: Backoff strategies
    println!("\n\nExample 5: Backoff Strategies");
    println!("=============================\n");

    let immediate = BackoffStrategy::Immediate;
    let linear = BackoffStrategy::Linear {
        base_delay: Duration::from_millis(100),
    };
    let exponential = BackoffStrategy::Exponential {
        base_delay: Duration::from_millis(100),
        multiplier: 1.5,
        max_delay: Duration::from_secs(30),
    };

    println!("Restart delay for attempts (in milliseconds):\n");
    println!("Attempt  Immediate   Linear      Exponential");
    println!("--------  ---------   ------      -----------");
    for attempt in 0..10 {
        let imm_delay = immediate.calculate_delay(attempt);
        let lin_delay = linear.calculate_delay(attempt);
        let exp_delay = exponential.calculate_delay(attempt);

        println!(
            "   {}      {:>4}ms    {:>4}ms      {:>7}ms",
            attempt,
            imm_delay.as_millis(),
            lin_delay.as_millis(),
            exp_delay.as_millis()
        );
    }

    // Example 6: Supervision tree visualization
    println!("\n\nExample 6: Supervision Tree Hierarchy");
    println!("=====================================\n");
    println!("Root Supervisor");
    println!("├─ Database Component [Permanent]");
    println!("│  ├─ Connection Pool [Permanent]");
    println!("│  └─ Query Executor [Transient]");
    println!("├─ API Component [Permanent]");
    println!("│  ├─ Request Handler [Transient]");
    println!("│  └─ Response Builder [Transient]");
    println!("└─ Worker Pool [Permanent]");
    println!("   ├─ Worker 1 [Permanent]");
    println!("   ├─ Worker 2 [Permanent]");
    println!("   └─ Worker 3 [Permanent]");

    // Example 7: Restart limit scenarios
    println!("\n\nExample 7: Restart Limit Configuration");
    println!("======================================\n");

    let strict_config = SupervisorConfig::permanent()
        .with_max_restarts(1)
        .with_time_window(Duration::from_secs(10));
    println!("Strict (1 restart per 10s): {:?}", strict_config.restart_policy);

    let moderate_config = SupervisorConfig::permanent()
        .with_max_restarts(3)
        .with_time_window(Duration::from_secs(60));
    println!("Moderate (3 restarts per 60s): {:?}", moderate_config.restart_policy);

    let lenient_config = SupervisorConfig::permanent()
        .with_max_restarts(10)
        .with_time_window(Duration::from_secs(300));
    println!("Lenient (10 restarts per 300s): {:?}", lenient_config.restart_policy);

    // Example 8: Builder pattern composition
    println!("\n\nExample 8: Builder Pattern Configuration");
    println!("========================================\n");

    let complex_config = SupervisorConfig::transient()
        .with_max_restarts(5)
        .with_time_window(Duration::from_secs(120))
        .with_backoff(BackoffStrategy::Linear {
            base_delay: Duration::from_millis(200),
        })
        .with_shutdown_timeout(Duration::from_secs(10))
        .with_startup_timeout(Duration::from_secs(15));

    println!("Complex configuration created:");
    println!("  Policy: {:?}", complex_config.restart_policy);
    println!("  Max restarts: {}", complex_config.max_restarts);
    println!("  Time window: {:?}", complex_config.time_window);
    println!("  Shutdown timeout: {:?}", complex_config.shutdown_timeout);
    println!("  Startup timeout: {:?}", complex_config.startup_timeout);

    println!("\n✓ All examples completed successfully");
}
