//! SupervisorNode Integration Example
//!
//! Demonstrates the integration between ComponentSupervisor (Layer 1) and
//! SupervisorNode (Layer 3) for automatic component restart.

use airssys_wasm::actor::{
    ComponentSupervisor, HealthRestartConfig, SupervisorConfig, SupervisorNodeWrapper,
};
use airssys_wasm::core::ComponentId;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== SupervisorNode Integration Example ===\n");

    // Example 1: Create ComponentSupervisor with bridge
    println!("Example 1: ComponentSupervisor with SupervisorNode Bridge");
    let bridge = Arc::new(RwLock::new(SupervisorNodeWrapper::new()));
    let mut supervisor = ComponentSupervisor::with_bridge(bridge);
    println!("✓ ComponentSupervisor created with SupervisorNode bridge\n");

    // Example 2: Register components with different restart policies
    println!("Example 2: Restart Policies");

    println!("  Permanent Policy (always restart):");
    let permanent_id = ComponentId::new("permanent-component");
    let config_permanent = SupervisorConfig::permanent()
        .with_max_restarts(5)
        .with_time_window(Duration::from_secs(60));
    supervisor.supervise(&permanent_id, config_permanent)?;
    println!("    ✓ Registered component with Permanent policy");

    println!("\n  Transient Policy (restart on error only):");
    let transient_id = ComponentId::new("transient-component");
    let config_transient = SupervisorConfig::transient()
        .with_max_restarts(3)
        .with_time_window(Duration::from_secs(30));
    supervisor.supervise(&transient_id, config_transient)?;
    println!("    ✓ Registered component with Transient policy");

    println!("\n  Temporary Policy (never restart):");
    let temporary_id = ComponentId::new("temporary-component");
    let config_temporary = SupervisorConfig::temporary();
    supervisor.supervise(&temporary_id, config_temporary)?;
    println!("    ✓ Registered component with Temporary policy\n");

    // Example 3: Health-based restart configuration
    println!("Example 3: Health-Based Restart Configuration");
    let health_config = HealthRestartConfig::new(
        Duration::from_secs(30), // Check every 30 seconds
        3,                       // Restart after 3 consecutive failures
        true,                    // Enable health-based restart
    );
    println!(
        "  ✓ Health config: check_interval={:?}, failure_threshold={}",
        health_config.interval(),
        health_config.threshold()
    );
    println!(
        "  ✓ Health-based restart enabled: {}\n",
        health_config.is_enabled()
    );

    // Example 4: Supervision statistics
    println!("Example 4: Supervision Statistics");
    let stats = supervisor.get_statistics();
    println!("  Total supervised: {}", stats.total_supervised);
    println!("  Currently running: {}", stats.currently_running);
    println!("  Failed components: {}", stats.failed_components);
    println!(
        "  Total restart attempts: {}\n",
        stats.total_restart_attempts
    );

    // Example 5: Layer separation demonstration
    println!("Example 5: Architecture - Three-Layer Separation");
    println!("  Layer 1 (WASM Config):   ComponentSupervisor - Policy tracking");
    println!("  Layer 2 (WASM Lifecycle): ComponentActor - WASM execution");
    println!("  Layer 3 (Actor Runtime):  SupervisorNode - Restart execution");
    println!("  Bridge:                   SupervisorNodeBridge - Integration\n");

    println!("=== Example Complete ===");
    Ok(())
}
