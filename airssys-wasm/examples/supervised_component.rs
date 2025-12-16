//! # Supervised Component Example
//!
//! **Purpose**: Demonstrate SupervisorConfig setup and restart strategies
//! **Demonstrates**: SupervisorConfig, restart strategies, health monitoring concepts
//! **Run**: `cargo run --example supervised_component`
//!
//! This example shows how to configure supervision for components including:
//! - Restart strategies (immediate, delayed, exponential backoff)
//! - Restart limits and time windows
//! - Health monitoring configuration
//! - Crash recovery simulation

// Layer 1: Standard library imports
use std::time::Duration;

// Layer 2: Third-party crate imports
// (none needed)

// Layer 3: Internal module imports
use airssys_rt::supervisor::{ChildHealth, RestartPolicy};
use airssys_wasm::core::ComponentId;

/// Simulate restart with exponential backoff
fn simulate_exponential_backoff(initial_delay_ms: u64, max_delay_ms: u64, multiplier: f64, attempts: u32) {
    println!("\n--- Exponential Backoff Simulation ---");
    let mut delay = Duration::from_millis(initial_delay_ms);
    let max_delay = Duration::from_millis(max_delay_ms);
    
    for attempt in 1..=attempts {
        println!("  Attempt {}: wait {}ms", attempt, delay.as_millis());
        
        // Calculate next delay
        let next_delay_ms = (delay.as_millis() as f64 * multiplier) as u64;
        delay = Duration::from_millis(next_delay_ms.min(max_delay.as_millis() as u64));
    }
}

/// Demonstrate health monitoring logic
fn demonstrate_health_monitoring(error_count: u32, threshold: u32) -> ChildHealth {
    if error_count > threshold {
        ChildHealth::Failed(format!("Error count {} exceeds threshold {}", error_count, threshold))
    } else if error_count > threshold / 2 {
        ChildHealth::Degraded(format!("Elevated errors: {}/{}", error_count, threshold))
    } else {
        ChildHealth::Healthy
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Supervised Component Demo ===\n");
    
    // Step 1: Create component ID
    let component_id = ComponentId::new("sensor-monitor");
    println!("Component ID: {}", component_id.as_str());
    
    // Step 2: Configure restart strategies
    println!("\n--- Available Restart Strategies ---");
    
    println!("\n1. Permanent - Always restart (critical services):");
    let restart_policy_permanent = RestartPolicy::Permanent;
    println!("   RestartPolicy::Permanent");
    println!("   - Restarts on: Any exit (error or normal)");
    println!("   - Use for: Critical services that must always run");
    println!("   - Example: HTTP server, database connection pool");
    
    println!("\n2. Transient - Restart on error only:");
    let restart_policy_transient = RestartPolicy::Transient;
    println!("   RestartPolicy::Transient");
    println!("   - Restarts on: Error exit only");
    println!("   - Use for: Workers that may complete successfully");
    println!("   - Example: Job processor, batch worker");
    
    println!("\n3. Temporary - Never restart:");
    let restart_policy_temporary = RestartPolicy::Temporary;
    println!("   RestartPolicy::Temporary");
    println!("   - Restarts on: Never");
    println!("   - Use for: One-shot tasks, temporary processes");
    println!("   - Example: Migration script, one-time job");
    
    println!("\n✓ Selected strategy for this demo: Permanent");
    
    // Step 3: Test restart decision logic
    println!("\n--- Restart Decision Logic ---");
    let max_restarts = 5u32;
    let time_window = Duration::from_secs(60);
    
    println!("Policy decision for different exit conditions:");
    println!("  Permanent + error exit: {}", restart_policy_permanent.should_restart(true));
    println!("  Permanent + normal exit: {}", restart_policy_permanent.should_restart(false));
    println!("  Transient + error exit: {}", restart_policy_transient.should_restart(true));
    println!("  Transient + normal exit: {}", restart_policy_transient.should_restart(false));
    println!("  Temporary + error exit: {}", restart_policy_temporary.should_restart(true));
    println!("  Temporary + normal exit: {}", restart_policy_temporary.should_restart(false));
    
    println!("\n✓ Restart limits: {} per {} seconds", max_restarts, time_window.as_secs());
    println!("  This prevents infinite restart loops on persistent failures");
    
    // Step 4: Demonstrate health monitoring
    println!("\n--- Configuring Health Monitoring ---");
    let check_interval = Duration::from_secs(5);
    let unhealthy_threshold = 3u32;
    println!("✓ Health check interval: {}s", check_interval.as_secs());
    println!("✓ Unhealthy threshold: {} failed checks", unhealthy_threshold);
    
    println!("\n--- Health Check Examples ---");
    let error_scenarios = vec![
        (0, "Normal operation"),
        (2, "Elevated errors (degraded)"),
        (5, "Critical errors (failed)"),
    ];
    
    for (error_count, description) in error_scenarios {
        let health = demonstrate_health_monitoring(error_count, unhealthy_threshold);
        print!("  Error count {}: {} → ", error_count, description);
        match health {
            ChildHealth::Healthy => println!("✓ Healthy"),
            ChildHealth::Degraded(reason) => println!("⚠️  Degraded ({})", reason),
            ChildHealth::Failed(reason) => println!("❌ Failed ({})", reason),
        }
    }
    
    // Step 5: Demonstrate crash recovery scenario
    println!("\n--- Crash Recovery Scenario ---");
    println!("Using Permanent restart policy:");
    println!("1. Component crashes (panic or error)");
    println!("2. Supervisor detects failure via health_check()");
    println!("3. Applies restart strategy based on RestartPolicy");
    println!("4. Simulating exponential backoff for demonstration:");
    
    simulate_exponential_backoff(100, 30000, 2.0, 5);
    
    println!("\n4. Health check validates component is healthy");
    println!("5. Component resumes normal operation");
    
    // Step 6: Demonstrate restart limit enforcement
    println!("\n--- Restart Limit Enforcement ---");
    println!("Simulating rapid failures:");
    for restart_num in 1..=max_restarts {
        println!("  Restart #{}: Component crashes immediately", restart_num);
    }
    println!("\n  Restart limit reached ({}/{})", max_restarts, max_restarts);
    println!("  → Supervisor stops restart attempts");
    println!("  → Component marked as permanently failed");
    println!("  → Operator notification triggered");
    
    // Step 7: Show supervision strategies
    println!("\n--- Supervision Strategies ---");
    println!("Available in airssys-rt:");
    println!("  1. OneForOne: Restart only failed child");
    println!("     - Use for: Independent components");
    println!("     - Example: Multiple HTTP workers");
    println!();
    println!("  2. OneForAll: Restart all children when one fails");
    println!("     - Use for: Tightly coupled components");
    println!("     - Example: Database pool + connection manager");
    println!();
    println!("  3. RestForOne: Restart failed child + all started after it");
    println!("     - Use for: Pipeline dependencies");
    println!("     - Example: Ingress → Processor → Egress");
    
    // Summary
    println!("\n=== Demo Complete ===");
    println!("\n✅ Supervision configuration demonstrated successfully!");
    println!("   - Restart policies: Permanent, Transient, Temporary");
    println!("   - Restart limits: {} per {}s window", max_restarts, time_window.as_secs());
    println!("   - Health monitoring: {}s interval", check_interval.as_secs());
    println!("   - Exponential backoff simulation");
    
    println!("\n📊 Performance:");
    println!("   - Supervisor overhead: <10µs per health check");
    println!("   - Restart detection: <100µs");
    println!("   - See: airssys-rt/BENCHMARKING.md");
    
    println!("\n📖 Integration:");
    println!("   To use supervision in your component:");
    println!("   1. Implement Child trait (start, stop, health_check)");
    println!("   2. Create SupervisorNode with RestartPolicy");
    println!("   3. Add child with supervisor.add_child()");
    println!("   4. Supervisor automatically handles failures");
    println!("   See: airssys-rt/examples/supervisor_*.rs for full examples");
    
    Ok(())
}
